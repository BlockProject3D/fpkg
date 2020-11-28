// Copyright (c) 2020, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::collections::HashMap;
use std::string::String;
use std::path::Path;
use std::path::PathBuf;
use std::fs;
use std::io;
use crate::command;

#[cfg(windows)]
use winapi::um::fileapi;

pub struct Profile
{
    path: PathBuf,
    data: HashMap<String, String>
}

fn read_profile(path: &Path, map: &mut HashMap<String, String>)
{
    let res = fs::read_to_string(path);
    if res.is_err() {
        return;
    }
    let jres = json::parse(&res.unwrap());
    if jres.is_err() {
        return;
    }
    let json = jres.unwrap();
    for v in json.entries()
    {
        let (f, f1) = v;
        map.insert(String::from(f), f1.to_string());
    }
}

fn find_compiler_info() -> io::Result<(String, String)>
{
    println!("Reading compiler information...");
    let dir = tempfile::tempdir()?;
    let content =
    "
        cmake_minimum_required(VERSION 3.10)
        project(DetectCompiler)

        function(FixedMessage)
            execute_process(COMMAND ${CMAKE_COMMAND} -E echo \"${ARGN}\")
        endfunction()

        FixedMessage(${CMAKE_CXX_COMPILER_ID})
        FixedMessage(${CMAKE_CXX_COMPILER_VERSION})
    ";
    fs::write(&dir.path().join("CMakeLists.txt"), content)?;
    let s = command::run_command_with_output("cmake", &["-S", &dir.path().to_string_lossy(), "-B", &dir.path().to_string_lossy()])?;
    let mut compiler = "";
    let mut version = "";
    let lines = s.split("\n");
    for l in lines
    {
        if l.starts_with("--")
        {
            continue;
        }
        if compiler == ""
        {
            compiler = l;
        }
        else if version == ""
        {
            version = l;
        }
        else
        {
            break;
        }
    }
    if compiler == "" || version == ""
    {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Unable to read compiler information"));
    }
    println!("Found compiler {} ({})", compiler, version);
    return Ok((String::from(compiler), String::from(version)));
}

impl Profile
{
    pub fn mkdir(path: &Path) -> io::Result<()>
    {
        let p = path.join(Path::new(".fpkg"));

        if !p.exists() {
            fs::create_dir(&p)?;
            #[cfg(windows)]
            fileapi::SetFileAttributesA(&p, fileapi::FILE_ATTRIBUTE_HIDDEN);
        }
        return Ok(());
    }

    pub fn new(path: &Path) -> Profile
    {
        let mut map = HashMap::new();
        let p = path.join(Path::new(".fpkg/profile"));

        if p.exists() {
            read_profile(&p, &mut map);
        }
        return Profile
        {
            path: p,
            data: map
        };
    }

    pub fn exists(&self) -> bool
    {
        return self.path.exists();
    }

    pub fn get(&self, name: &str) -> Option<&String>
    {
        return self.data.get(&String::from(name));
    }

    pub fn regenerate_cross(&mut self, name: &str) -> Result<(), String> //Regenerate profile for cross-compile platform
    {
        return Err(format!("Platform name {} does not exist", name));
    }

    pub fn write(&self) -> io::Result<()>
    {
        let mut json = json::JsonValue::new_object();

        for (k, v) in &self.data
        {
            json[k] = json::JsonValue::String(v.to_string());
        }
        fs::write(&self.path, json.dump())?;
        return Ok(());
    }

    pub fn fill_table(&self, table: &mut rlua::Table) -> rlua::Result<()>
    {
        for (k, v) in self.data.iter()
        {
            table.set(k.as_str(), v.as_str())?; //Annoying peace of shit RLua is unable to take Strings!
        }
        return Ok(());
    }

    pub fn regenerate_self(&mut self) -> Result<(), String> //Regenerate profile for current platform
    {
        if cfg!(target_os = "windows")
        {
            self.data.insert(String::from("Platform"), String::from("Windows"));
        }
        else if cfg!(target_os = "macos")
        {
            self.data.insert(String::from("Platform"), String::from("OSX"));
        }
        else if cfg!(target_os = "linux")
        {
            self.data.insert(String::from("Platform"), String::from("Linux"));
        }
        else if cfg!(target_os = "android")
        {
            self.data.insert(String::from("Platform"), String::from("Android"));
        }
        if cfg!(target_arch = "x86")
        {
            self.data.insert(String::from("Arch"), String::from("x86"));
        }
        else if cfg!(target_arch = "x86_64")
        {
            self.data.insert(String::from("Arch"), String::from("x86_64"));
        }
        else if cfg!(target_arch = "arm")
        {
            self.data.insert(String::from("Arch"), String::from("arm"));
        }
        else if cfg!(target_arch = "aarch64")
        {
            self.data.insert(String::from("Arch"), String::from("aarch64"));
        }
        println!("Identified platform as {} {}", self.data.get("Platform").unwrap(), self.data.get("Arch").unwrap());
        match find_compiler_info()
        {
            Ok((name, version)) =>
            {
                self.data.insert(String::from("CompilerName"), name);
                self.data.insert(String::from("CompilerVersion"), version);
            },
            Err(e) => return Err(format!("{}", e))
        }
        return Ok(());
    }
}

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
use crate::common::Result;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::common::read_property_map;

#[cfg(windows)]
use winapi::um::fileapi;

const CROSS_PLATFORMS: [&'static str; 1] = ["android"];

pub struct Profile
{
    path: PathBuf,
    data: HashMap<String, String>,
    platform: String
}

fn find_compiler_info() -> Result<(String, String)>
{
    println!("Reading compiler information...");
    let dir = match tempfile::tempdir()
    {
        Ok(v) => v,
        Err(e) => return Err(Error::Io(ErrorDomain::Profile, e))
    };
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
    if let Err(e) = fs::write(&dir.path().join("CMakeLists.txt"), content)
    {
        return Err(Error::Io(ErrorDomain::Profile, e));
    }
    let s = match command::run_command_with_output("cmake", &["-S", &dir.path().to_string_lossy(), "-B", &dir.path().to_string_lossy()])
    {
        Ok(v) => v,
        Err(e) => return Err(Error::Io(ErrorDomain::Profile, e))
    };
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
        return Err(Error::Generic(ErrorDomain::Profile, String::from("Unable to read compiler information")));
    }
    println!("Found compiler {} ({})", compiler, version);
    return Ok((String::from(compiler), String::from(version)));
}

impl Profile
{
    fn mkdir(&self) -> io::Result<()>
    {
        let toolchain = self.path.join(Path::new(&self.platform));

        if !self.path.exists()
        {
            fs::create_dir(&self.path)?;
            #[cfg(windows)]
            fileapi::SetFileAttributesA(&self.path, fileapi::FILE_ATTRIBUTE_HIDDEN);
        }
        if !toolchain.exists()
        {
            fs::create_dir(&toolchain)?;
        }
        return Ok(());
    }

    pub fn new(path: &Path) -> Result<Profile>
    {
        let mut map = HashMap::new();
        let p = path.join(Path::new(".fpkg/host/profile"));

        if p.exists() {
            read_property_map(&p, &mut map)?;
        }
        return Ok(Profile
        {
            path: path.join(".fpkg"),
            data: map,
            platform: String::from("host")
        });
    }

    pub fn get_platform_path(&self) -> PathBuf
    {
        return self.path.join(Path::new(&self.platform));
    }

    pub fn get_path(&self) -> &Path
    {
        return &self.path;
    }

    pub fn get_platform(&self) -> &str
    {
        return &self.platform;
    }

    pub fn exists(&self) -> bool
    {
        return self.get_platform_path().join("profile").exists();
    }

    pub fn get(&self, name: &str) -> Option<&String>
    {
        return self.data.get(&String::from(name));
    }

    pub fn write(&self) -> io::Result<()>
    {
        let mut json = json::JsonValue::new_object();

        for (k, v) in &self.data
        {
            json[k] = json::JsonValue::String(v.to_string());
        }
        fs::write(&self.get_platform_path().join("profile"), json.dump())?;
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

    pub fn fill_structured_data(&self, obj: &mut bpx::sd::Object)
    {
        for (k, v) in self.data.iter()
        {
            obj.set(k, bpx::sd::Value::String(v.clone()));
        }
    }

    //Sets the name of the current platform
    pub fn set_platform(&mut self, name: &str) -> Result<()>
    {
        for v in &CROSS_PLATFORMS
        {
            if &name == v
            {
                self.platform = String::from(name);
                let path = self.get_platform_path().join("profile");
                let mut map = HashMap::new();
                read_property_map(&path, &mut map)?;
                self.data = map;
                return Ok(());
            }
        }
        return Err(Error::Generic(ErrorDomain::Profile, format!("Unknown cross-compile target platform: {}", name)));
    }

    pub fn install(&mut self) -> Result<()>
    {
        if !self.exists()
        {
            if let Err(e) = self.mkdir()
            {
                return Err(Error::Io(ErrorDomain::Profile, e));
            }
            if &self.platform == "host"
            {
                self.regenerate_self()?;
            }
            else
            {
                self.regenerate_cross()?;
            }
            if let Err(e) = self.write()
            {
                return Err(Error::Io(ErrorDomain::Profile, e));
            }
        }
        return Ok(());
    }

    fn regenerate_cross(&mut self) -> Result<()> //Regenerate profile for cross-compiled target platform
    {
        return Err(Error::Generic(ErrorDomain::Profile, String::from("Cross-compiled target platforms are currently not supported")));
    }

    fn regenerate_self(&mut self) -> Result<()> //Regenerate profile for host target platform
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
        let (name, version) = find_compiler_info()?;
        self.data.insert(String::from("CompilerName"), name);
        self.data.insert(String::from("CompilerVersion"), version);
        return Ok(());
    }
}

// Copyright (c) 2021, BlockProject 3D
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

use std::string::String;
use std::collections::HashMap;
use std::boxed::Box;
use std::fs;

use crate::command;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::common::Result;
use crate::profile::Profile;
use crate::hosttoolchain::HostToolchainProvider;

pub trait Toolchain
{
    fn create_profile(&mut self) -> Result<Profile>;
    fn generate(&mut self) -> Result<()>;
}

pub trait ToolchainProvider
{
    fn new(&self, props: Option<HashMap<String, String>>) -> Result<Box<dyn Toolchain>>;
}

pub fn create_toolchain(toolchain_name: &str, props: Option<HashMap<String, String>>) -> Result<Option<Box<dyn Toolchain>>>
{
    let mut v: HashMap<&str, Box<dyn ToolchainProvider>> = HashMap::new();
    v.insert("host", Box::new(HostToolchainProvider {}));
    if let Some(t) = v.remove(toolchain_name)
    {
        let toolchain = t.new(props)?;
        return Ok(Some(toolchain));
    }
    return Ok(None);
}

pub fn find_compiler_info(toolchain_file: Option<&str>) -> Result<(String, String)>
{
    println!("Reading compiler information using CMake...");
    let dir = match tempfile::tempdir()
    {
        Ok(v) => v,
        Err(e) => return Err(Error::Io(ErrorDomain::Toolchain, e))
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
        return Err(Error::Io(ErrorDomain::Toolchain, e));
    }
    let useless = match toolchain_file
    {
        Some(file) => command::run_command_with_output("cmake", &[&format!("-DCMAKE_TOOLCHAIN_FILE={}", file),
                                                                "-S", &dir.path().to_string_lossy(),
                                                                "-B", &dir.path().to_string_lossy()]),
        None => command::run_command_with_output("cmake", &["-S", &dir.path().to_string_lossy(),
                                                            "-B", &dir.path().to_string_lossy()])
    };
    let s = match useless
    {
        Ok(v) => v,
        Err(e) => return Err(Error::Io(ErrorDomain::Toolchain, e))
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
        return Err(Error::Generic(ErrorDomain::Toolchain, String::from("Unable to read compiler information")));
    }
    return Ok((String::from(compiler), String::from(version)));
}

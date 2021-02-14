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

use std::path::Path;
use std::string::String;

use crate::command::run_command;
use crate::common::Result;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::builder::Builder;
use crate::builder::check_build_configuration;

pub struct CMakeBuilder {}

impl Builder for CMakeBuilder
{
    fn can_build(&self, path: &Path) -> bool
    {
        return path.join("CMakeLists.txt").exists();
    }

    fn run_build(&self, configuration: &str, path: &Path) -> Result<i32>
    {
        let mut builddir = String::from("build-");
        let mut buildtype = String::from("-DCMAKE_BUILD_TYPE=");
        let config = check_build_configuration(&configuration, &Some(vec!(String::from("Debug"), String::from("Release"))))?;

        builddir.push_str(&config.to_lowercase());
        buildtype.push_str(&config);
        match run_command("cmake", &["-S", &path.to_string_lossy(), "-B", &builddir, &buildtype])
        {
            Ok(status) =>
            {
                if !status.success()
                {
                    if let Some(v) = status.code()
                    {
                        return Ok(v);
                    }
                    eprintln!("The target application has crashed!");
                    return Ok(11);    
                }
            }
            Err(e) => return Err(Error::Io(ErrorDomain::Builder, e))
        };
        match run_command("cmake", &["--build", &builddir, "--config", &config])
        {
            Ok(status) =>
            {
                if let Some(v) = status.code()
                {
                    return Ok(v);
                }
                eprintln!("The target application has crashed!");
                return Ok(11);
            }
            Err(e) => return Err(Error::Io(ErrorDomain::Builder, e))
        };
    }
}

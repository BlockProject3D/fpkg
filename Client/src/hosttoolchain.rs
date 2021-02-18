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

use crate::profile::Profile;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::common::Result;
use crate::toolchain::Toolchain;
use crate::toolchain::ToolchainProvider;
use crate::toolchain::find_compiler_info;

struct HostToolchain {}

impl Toolchain for HostToolchain
{
    fn create_profile(&mut self) -> Result<Profile>
    {
        let mut platform = "";
        let mut architecture = "";

        if cfg!(target_os = "windows")
        {
            platform = "Windows";
        }
        else if cfg!(target_os = "macos")
        {
            platform = "OSX";
        }
        else if cfg!(target_os = "linux")
        {
            platform = "Linux";
        }
        else if cfg!(target_os = "android")
        {
            platform = "Android";
        }
        if cfg!(target_arch = "x86")
        {
            architecture = "x86";
        }
        else if cfg!(target_arch = "x86_64")
        {
            architecture = "x86_64";
        }
        else if cfg!(target_arch = "arm")
        {
            architecture = "arm";
        }
        else if cfg!(target_arch = "aarch64")
        {
            architecture = "aarch64";
        }
        if platform == "" || architecture == ""
        {
            return Err(Error::Generic(ErrorDomain::Toolchain, String::from("HostToolchain failure: impossible to obtain the current running platform and/or architecture!")));
        }
        let (compiler, version) = find_compiler_info(None)?;
        return Ok(Profile
        {
            compiler_name: compiler,
            compiler_version: version,
            architecture: String::from(architecture),
            platform: String::from(platform)
        });
    }

    fn generate(&mut self) -> Result<()>
    {
        return Ok(());
    }
}

pub struct HostToolchainProvider {}

impl ToolchainProvider for HostToolchainProvider
{
    fn new(&self, props: Option<HashMap<String, String>>) -> Result<Box<dyn Toolchain>>
    {
        if props.is_some()
        {
            return Err(Error::Generic(ErrorDomain::Toolchain, String::from("The host toolchain does not support properties")));
        }
        return Ok(Box::new(HostToolchain {}));
    }
}

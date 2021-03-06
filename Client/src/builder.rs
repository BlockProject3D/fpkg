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

use std::path::Path;
use std::boxed::Box;
use std::string::String;

use crate::common::Result;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::luabuilder::LuaBuilder;
use crate::cmakebuilder::CMakeBuilder;

pub trait Builder
{
    fn can_build(&self, path: &Path) -> bool;
    fn run_build(&self, config: &str, path: &Path, toolchain: Option<&str>) -> Result<i32>;
}

pub fn find_builder(path: &Path) -> Option<Box<dyn Builder>>
{
    let mut builders: Vec<Box<dyn Builder>> = vec!(
        Box::new(LuaBuilder {}),
        Box::new(CMakeBuilder {})
    );

    for i in 0..builders.len()
    {
        if builders[i].can_build(path) {
            return Some(builders.remove(i));
        }
    }
    return None;
}

pub fn check_build_configuration(config: &str, configs: &Option<Vec<String>>) -> Result<String>
{
    match configs
    {
        None => return Ok(String::from(config)),
        Some(v) =>
        {
            let cfg = v.iter().find(|v| v == &config || v.to_lowercase() == config);
            match cfg
            {
                None => return Err(Error::Generic(ErrorDomain::Builder, format!("Could not find configuration named {}", config))),
                Some(v) => return Ok(String::from(v))
            }
        }
    }
}

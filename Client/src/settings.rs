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

use dirs::config_dir;
use std::collections::HashMap;
use std::path::Path;
use std::fs;

use crate::common::Error;
use crate::common::Result;
use crate::common::ErrorDomain;

pub struct RegistryInfo
{
    base_url: String,
    access_token: String
}

pub struct Settings
{
    default_registry: String,
    registries: HashMap<String, String>
}

fn read_settings(path: &Path) -> Result<Settings, Error>
{
    let res = match fs::read_to_string(path)
    {
        Ok(v) => v,
        Err(e) => return Err(Error::Io(ErrorDomain::Settings, e))
    };
    let json = match json::parse(&res)
    {
        Ok(v) => v,
        Err(e) => return Err(Error::Generic(ErrorDomain::Settings, format!("Error parsing json: {}", e)))
    };
    for v in json.entries()
    {
        let (f, f1) = v;
        map.insert(String::from(f), f1.to_string());
    }
}

impl Settings
{
    pub fn new() -> Result<Settings, Error>
    {
        let mut path = match config_dir()
        {
            Some(v) => v,
            None => return Err(Error::Generic(ErrorDomain::Settings, String::from("Unable to obtain a valid config directory, is your system sane?!")))
        };
        path.push(Path::new("fpkg-settings.json"));

    }
}

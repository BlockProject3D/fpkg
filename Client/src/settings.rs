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
use json::JsonValue;
use std::vec::Vec;

use crate::common::Error;
use crate::common::Result;
use crate::common::ErrorDomain;

#[cfg(unix)]
const PATH_LOCAL_REG: &str = "local:///opt/fpkg/";

#[cfg(windows)]
const PATH_LOCAL_REG: &str = "local://C:/fpkg/";

#[derive(Clone)]
pub struct RegistryInfo
{
    pub priority: i32,
    pub base_url: String,
    pub access_token: Option<String>
}

pub struct Settings
{
    default_registry: String,
    registry_map: HashMap<String, RegistryInfo>,
    registry_list: Vec<String>
}

fn read_settings(path: &Path) -> Result<Settings>
{
    let mut map = HashMap::new();
    let mut list = Vec::new();
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
    match &json["Registries"]
    {
        JsonValue::Object(v) =>
        {
            for entry in v.iter()
            {
                let (key, val) = entry;
                let url = match val["BaseUrl"].as_str()
                {
                    Some(v) => v,
                    None => return Err(Error::Generic(ErrorDomain::Settings, String::from("Invalid type for 'BaseUrl' key")))
                };
                let token = match val["AccessToken"].as_str()
                {
                    Some(v) => Some(String::from(v)),
                    None => None
                };
                let priority = match val["Priority"].as_number()
                {
                    Some(v) =>
                    {
                        let useless: f64 = v.into();
                        useless as i32
                    },
                    None => 0
                };
                let mut key = String::from(key);
                if map.contains_key(&key)
                {
                    key.push('-');
                    key.push_str(&(map.len() + 1).to_string());
                }
                map.insert(key.clone(), RegistryInfo
                {
                    priority: priority,
                    base_url: String::from(url),
                    access_token: token
                });
                list.push(key);
            }
        },
        _ => return Err(Error::Generic(ErrorDomain::Settings, String::from("Invalid type for 'Registries' key")))
    };
    let default = match &json["DefaultRegistry"]
    {
        JsonValue::Null =>
        {
            let mut val = None;
            for (k, _) in &map
            {
                val = Some(String::from(k));
                break;
            }
            val
        }
        JsonValue::String(v) =>
        {
            let mut val = None;
            if map.contains_key(v)
            {
                val = Some(String::from(v));
            }
            val
        },
        _ => return Err(Error::Generic(ErrorDomain::Settings, String::from("Invalid type for 'DefaultRegistry' key")))
    };
    if default.is_none()
    {
        return Err(Error::Generic(ErrorDomain::Settings, String::from("Default registry does not exist")));
    }
    list.sort_by(|a, b|
    {
        let v = map[a].priority;
        let v1 = map[b].priority;
        return v1.cmp(&v);
    });
    return Ok(Settings
    {
        default_registry: default.unwrap(),
        registry_map: map,
        registry_list: list
    });
}

impl Settings
{
    pub fn new() -> Result<Settings>
    {
        let mut path = match config_dir()
        {
            Some(v) => v,
            None => return Err(Error::Generic(ErrorDomain::Settings, String::from("Unable to obtain a valid config directory, is your system sane?!")))
        };
        path.push(Path::new("fpkg-settings.json"));
        if path.exists()
        {
            return read_settings(&path);
        }
        let mut map = HashMap::new();
        map.insert(String::from("LocalSystem"), RegistryInfo
        {
            priority: 0,
            base_url: String::from(PATH_LOCAL_REG),
            access_token: None
        });
        return Ok(Settings
        {
            default_registry: String::from("LocalSystem"),
            registry_map: map,
            registry_list: vec!(String::from("LocalSystem"))
        });
    }

    pub fn get_registries(&self) -> Vec<&RegistryInfo>
    {
        let mut res = Vec::new();

        for v in &self.registry_list
        {
            res.push(&self.registry_map[v]);
        }
        return res;
    }

    pub fn get_registry(&self, name: Option<&str>) -> Result<&RegistryInfo>
    {
        match name
        {
            None => return Ok(&self.registry_map[&self.default_registry]),
            Some(v) =>
            {
                match &self.registry_map.get(&String::from(v))
                {
                    None => return Err(Error::Generic(ErrorDomain::Settings, format!("Registry {} does not exist", v))),
                    Some(v) => return Ok(v)
                };
            }
        };
    }
}

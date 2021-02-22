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

use std::collections::HashMap;
use std::string::String;
use std::path::Path;
use std::path::PathBuf;
use std::fs;
use std::io;
use std::boxed::Box;

use crate::common::Result;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::common::read_property_map;
use crate::toolchain::create_toolchain;
use crate::toolchain::Toolchain;

#[cfg(windows)]
use winapi::um::fileapi;
#[cfg(windows)]
use winapi::um::winnt::FILE_ATTRIBUTE_HIDDEN;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

fn hash_check_key(map: &mut HashMap<String, String>, key_name: &str) -> Result<String>
{
    return match map.remove(key_name)
    {
        None => Err(Error::Generic(ErrorDomain::Profile, format!("Missing key {} from profile", key_name))),
        Some(v) => Ok(v)
    };
}

fn bpx_check_key(map: &bpx::sd::Object, key_name: &str) -> Result<String>
{
    return match map.get(key_name)
    {
        None => Err(Error::Generic(ErrorDomain::Profile, format!("Missing key {} from profile", key_name))),
        Some(v) =>
        {
            return match v
            {
                bpx::sd::Value::String(v) => Ok(v.clone()),
                _ => Err(Error::Generic(ErrorDomain::Profile, format!("Bad type for key {}", key_name)))
            };
        }
    };
}

pub struct Profile
{
    pub compiler_name: String,
    pub compiler_version: String,
    pub platform: String,
    pub architecture: String
}

impl Profile
{
    pub fn from_bpxsd(obj: &bpx::sd::Object) -> Result<Profile>
    {
        let p = Profile
        {
            compiler_name: bpx_check_key(obj, "CompilerName")?,
            compiler_version: bpx_check_key(obj, "CompilerVersion")?,
            platform: bpx_check_key(obj, "Platform")?,
            architecture: bpx_check_key(obj, "Arch")?
        };
        return Ok(p);
    }

    pub fn from_file(path: &Path) -> Result<Profile>
    {
        let mut map = HashMap::new();
        read_property_map(&path, &mut map)?;
        let p = Profile
        {
            compiler_name: hash_check_key(&mut map, "CompilerName")?,
            compiler_version: hash_check_key(&mut map, "CompilerVersion")?,
            platform: hash_check_key(&mut map, "Platform")?,
            architecture: hash_check_key(&mut map, "Arch")?
        };
        return Ok(p);
    }

    pub fn to_file(&self, path: &Path) -> io::Result<()>
    {
        let mut json = json::JsonValue::new_object();
        json["CompilerName"] = json::JsonValue::String(self.compiler_name.clone());
        json["CompilerVersion"] = json::JsonValue::String(self.compiler_version.clone());
        json["Platform"] = json::JsonValue::String(self.platform.clone());
        json["Arch"] = json::JsonValue::String(self.architecture.clone());
        fs::write(path, json.dump())?;
        return Ok(());
    }

    pub fn fill_structured_data(&self, obj: &mut bpx::sd::Object)
    {
        obj.set("CompilerName", bpx::sd::Value::String(self.compiler_name.clone()));
        obj.set("CompilerVersion", bpx::sd::Value::String(self.compiler_version.clone()));
        obj.set("Platform", bpx::sd::Value::String(self.platform.clone()));
        obj.set("Arch", bpx::sd::Value::String(self.architecture.clone()));
    }

    pub fn fill_table(&self, table: &mut rlua::Table) -> rlua::Result<()>
    {
        //Annoying peace of shit RLua is unable to take Strings!
        table.set("compilerName", self.compiler_name.as_str())?;
        table.set("compilerVersion", self.compiler_version.as_str())?;
        table.set("platform", self.platform.as_str())?;
        table.set("architecture", self.architecture.as_str())?;
        return Ok(());
    }
}

pub struct ProfileManager
{
    path: PathBuf,
    toolchain_name: String,
    current_profile: Option<Profile>
}

impl ProfileManager
{
    fn mkdir(&self) -> io::Result<()>
    {
        let toolchain = self.path.join(Path::new(&self.toolchain_name));

        if !self.path.exists()
        {
            fs::create_dir(&self.path)?;
            #[cfg(windows)]
            unsafe
            {
                let result: Vec<u16> = self.path.as_os_str().encode_wide().collect();
                fileapi::SetFileAttributesW(result.as_ptr(), FILE_ATTRIBUTE_HIDDEN);
            }
        }
        if !toolchain.exists()
        {
            fs::create_dir(&toolchain)?;
        }
        return Ok(());
    }

    pub fn new(path: &Path) -> Result<ProfileManager>
    {
        let p = path.join(Path::new(".fpkg/host/profile"));
        let mut profile = None;

        if p.exists()
        {
            profile = Some(Profile::from_file(&p)?);
        }
        return Ok(ProfileManager
        {
            path: path.join(".fpkg"),
            toolchain_name: String::from("host"),
            current_profile: profile
        });
    }

    pub fn get_current(&self) -> Result<&Profile>
    {
        match &self.current_profile
        {
            None => return Err(Error::Generic(ErrorDomain::Profile, String::from("The current profile is not initialized"))),
            Some(v) => return Ok(&v)
        };
    }

    pub fn get_toolchain_path(&self) -> PathBuf
    {
        return self.path.join(Path::new(&self.toolchain_name));
    }

    pub fn get_base_path(&self) -> &Path
    {
        return &self.path;
    }

    pub fn get_toolchain(&self) -> &str
    {
        return &self.toolchain_name;
    }

    pub fn exists(&self) -> bool
    {
        return self.get_toolchain_path().join("profile").exists();
    }

    pub fn write(&self) -> io::Result<()>
    {
        if let Some(profile) = &self.current_profile
        {
            return profile.to_file(&self.get_toolchain_path().join("profile"));
        }
        return Ok(());
    }

    pub fn load(&mut self, toolchain_name: &str) -> Result<()>
    {
        let old_toolchain = std::mem::replace(&mut self.toolchain_name, String::from(toolchain_name));
        if !self.exists()
        {
            self.toolchain_name = old_toolchain;
            return Err(Error::Generic(ErrorDomain::Profile, format!("No toolchain named {} has been found in the ProfileManager", toolchain_name)));
        }
        let profile = match Profile::from_file(&self.get_toolchain_path().join("profile"))
        {
            Err(e) =>
            {
                self.toolchain_name = old_toolchain;
                return Err(e);
            },
            Ok(v) => v
        };
        self.current_profile = Some(profile);
        return Ok(());
    }

    pub fn install(&mut self, toolchain: &str, toolchain_props: Option<HashMap<String, String>>) -> Result<Box<dyn Toolchain>>
    {
        if self.exists()
        {
            return Err(Error::Generic(ErrorDomain::Profile, String::from("The current profile has already been installed")));
        }
        if let Err(e) = self.mkdir()
        {
            return Err(Error::Io(ErrorDomain::Profile, e));
        }
        let mut toolchain = match create_toolchain(toolchain, toolchain_props)?
        {
            None => return Err(Error::Generic(ErrorDomain::Profile, format!("Unknown toolchain name: {}", toolchain))),
            Some(v) => v
        };
        let profile = toolchain.create_profile()?;
        println!("Identified platform as {} {}", profile.platform, profile.architecture);
        println!("Found compiler {} ({})", profile.compiler_name, profile.compiler_version);
        self.current_profile = Some(profile);
        if let Err(e) = self.write()
        {
            return Err(Error::Io(ErrorDomain::Profile, e));
        }
        return Ok(toolchain);
    }
}

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
use std::path::PathBuf;
use std::string::String;

use crate::luaengine::LuaFile;
use crate::profile::ProfileManager;
use crate::settings::Settings;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::common::Result;
use crate::registry::open_package_registry;
use crate::registry::Package;
use crate::packager::get_pk_file;

pub fn publish(path: &Path, registry: Option<&str>) -> Result<i32>
{
    let settings = Settings::new()?;
    let profilemgr = ProfileManager::new(path)?;
    if !profilemgr.exists()
    {
        return Err(Error::Generic(ErrorDomain::Publisher, String::from("Unable to load project profile; did you forget to run fpkg install?")));
    }
    let profile = profilemgr.get_current()?;
    let p: PathBuf = [path, Path::new("fpkg.lua")].iter().collect();
    let mut lua = LuaFile::new();
    lua.open_libs()?;
    lua.open(&p)?;
    let package = lua.read_table()?;
    let pkg = Package::new(&package.name, &package.version);
    let file_name = get_pk_file(&profile);
    println!("Uploading package {} {} - {}...", &package.name, &package.version, &file_name);
    let registry_info = settings.get_registry(registry)?;
    let mut registry = open_package_registry(&registry_info)?;
    registry.ensure_valid_package(&pkg)?;
    registry.publish(&pkg, &file_name, Path::new(&file_name))?;
    println!("Uploaded package build {} to {}", &file_name, &registry_info.base_url);
    return Ok(0);
}

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

use std::string::String;
use std::path::Path;
use std::fs;

use crate::profile::Profile;
use crate::builder;
use crate::common::Result;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::luaengine::LuaFile;
use crate::luaengine::Dependency;
use crate::settings::Settings;
use crate::settings::RegistryInfo;
use crate::registry::open_package_registry;

fn run_lua_install(file: &mut LuaFile, profile: &Profile) -> Result<Option<Vec<String>>>
{
    if file.has_func_install()
    {
        return file.func_install(&profile);
    }
    else
    {
        return Ok(None);
    }
}

fn check_file_name_match(profile: &Profile, file_name: &str) -> bool
{
    //File format: build-Platform-Arch-CompilerName-CompilerVersion.bpx
    let components = &file_name[0..file_name.len() - 4].split('-').collect::<Vec<&str>>();
    let platform = profile.get("Platform").unwrap();
    let arch = profile.get("Arch").unwrap();
    let compiler = profile.get("CompilerName").unwrap();
    let version = profile.get("CompilerVersion").unwrap();

    if components.len() != 5 || &components[0] != &"build" || &components[1] != platform
        || &components[2] != arch || &components[3] != compiler
    {
        return false;
    }
    if &components[4] == version
    {
        return true;
    }
    else
    {
        let rep1 = version.replace('.', "");
        let rep2 = &components[4].replace('.', "");
        if let Ok(value1) = rep1.parse::<usize>()
        {
            if let Ok(value2) = rep2.parse::<usize>()
            {
                if value2 >= value1
                {
                    return true;
                }
                else
                {
                    return false;
                }
            }
        }
    }
    return false;
}

fn install_dependency(dep: &Dependency, profile: &Profile, registries: &Vec<RegistryInfo>) -> Result<()>
{
    println!("Installing dependency {} - {}...", &dep.name, &dep.version);
    for registry_info in registries
    {
        let mut registry = open_package_registry(&registry_info)?;
        if dep.version == "latest"
        {
            if let Some(pkg) = registry.find_latest(&dep.name)?
            {
                for file_name in &pkg.files
                {
                    if check_file_name_match(&profile, &file_name)
                    {
                        let folder = profile.get_platform_path().join(Path::new(&dep.name));
                        if !folder.exists()
                        {
                            if let Err(e) = fs::create_dir(&folder)
                            {
                                return Err(Error::Io(ErrorDomain::Installer, e));
                            }
                        }
                        registry.download(&folder, &pkg, &file_name)?;
                        //TODO: Add unpack system here
                        return Ok(());
                    }
                }
                return Err(Error::Generic(ErrorDomain::Installer, format!("The dependency ({} - {}) is not compatible with your system", &dep.name, &dep.version)));
            }
            continue;
        }
    }
    return Err(Error::Generic(ErrorDomain::Installer, format!("Could not find dependency {} in any registry", &dep.name)));
}

fn install_depenedencies(file: &mut LuaFile, profile: &Profile, registries: &Vec<RegistryInfo>) -> Result<()>
{
    let package = file.read_table()?;
    if let Some(deps) = package.dependencies
    {
        for dep in deps
        {
            install_dependency(&dep, &profile, &registries)?;
        }
    }
    return Ok(());
}

fn install_sub_directory(path: &Path, platform: Option<&str>) -> Result<Vec<String>>
{
    let settings = Settings::new()?;
    let mut res = Vec::new();
    let mut profile = Profile::new(path)?;
    let registries = settings.get_registries();

    if let Some(p) = platform
    {
        profile.set_platform(p)?;
    }
    profile.install()?;
    let path = Path::new(path).join("fpkg.lua");
    if path.exists()
    {
        let mut file = LuaFile::new();
        file.open_libs()?;
        file.open(&path)?;
        //TODO: Implement dependency/framework downloader/installer and connect it right here
        install_depenedencies(&mut file, &profile, &registries)?;
        if let Some(vc) = run_lua_install(&mut file, &profile)?
        {
            for path in vc
            {
                res.push(path);
            }
        }
    }
    return Ok(res);
}

fn check_is_valid_project_dir(path: &Path) -> Result<()>
{
    let builder = builder::find_builder(&path);
    if builder.is_none() {
        return Err(Error::Generic(ErrorDomain::Installer, String::from("Project directory does not contain a valid project file")));
    }
    return Ok(());
}

pub fn install(platform: Option<&str>) -> Result<()>
{
    let mut directories: Vec<String> = Vec::new();
    directories.push(String::from("."));
    while let Some(dir) = directories.pop()
    {
        check_is_valid_project_dir(Path::new(&dir))?;
        let subdirs = install_sub_directory(Path::new(&dir), platform)?;
        for v in subdirs
        {
            directories.push(v);
        }
    }
    return Ok(());
}

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
use std::collections::HashMap;
use bpx::bpxp;
use bpx::sd;
use std::io;

use crate::generator::create_generator;
use crate::generator::BuildGenerator;
use crate::generator::Target;
use crate::generator::Library;
use crate::profile::Profile;
use crate::profile::ProfileManager;
use crate::builder;
use crate::common::Result;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::luaengine::LuaFile;
use crate::luaengine::Dependency;
use crate::settings::Settings;
use crate::settings::RegistryInfo;
use crate::registry::open_package_registry;
use crate::common::read_property_map;

fn check_file_name_match(profile: &Profile, file_name: &str) -> bool
{
    //File format: build-Platform-Arch-CompilerName-CompilerVersion.bpx
    let components = &file_name[0..file_name.len() - 4].split('-').collect::<Vec<&str>>();
    let platform = &profile.platform;
    let arch = &profile.architecture;
    let compiler = &profile.compiler_name;
    let version = &profile.compiler_version;

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

fn build_package_info(obj: &sd::Object) -> Result<json::object::Object>
{
    let profile = Profile::from_bpxsd(&obj)?;
    let mut j = json::object::Object::new();
    j.insert("Platform", json::JsonValue::String(profile.platform.clone()));
    j.insert("Architecture", json::JsonValue::String(profile.architecture.clone()));
    j.insert("CompilerName", json::JsonValue::String(profile.compiler_name.clone()));
    j.insert("CompilerVersion", json::JsonValue::String(profile.compiler_version.clone()));
    if let (Some(name), Some(version), Some(typeafs), Some(desc)) = (obj.get("Name"), obj.get("Version"), obj.get("Type"), obj.get("Description"))
    {
        match name
        {
            sd::Value::String(s) => j.insert("Name", json::JsonValue::String(s.clone())),
            _ => return Err(Error::Generic(ErrorDomain::Installer, format!("Incorrect type for key 'Name'")))
        };
        match version
        {
            sd::Value::String(s) => j.insert("Version", json::JsonValue::String(s.clone())),
            _ => return Err(Error::Generic(ErrorDomain::Installer, format!("Incorrect type for key 'Version'")))
        };
        match typeafs
        {
            sd::Value::String(s) => j.insert("Type", json::JsonValue::String(s.clone())),
            _ => return Err(Error::Generic(ErrorDomain::Installer, format!("Incorrect type for key 'Type'")))
        };
        match desc
        {
            sd::Value::String(s) => j.insert("Description", json::JsonValue::String(s.clone())),
            _ => return Err(Error::Generic(ErrorDomain::Installer, format!("Incorrect type for key 'Description'")))
        };
        return Ok(j);
    }
    return Err(Error::Generic(ErrorDomain::Installer, String::from("BPX Error: missing package header metadata (name, type, version and description)")));
}

fn unpack_bpx(file: &Path, folder: &Path) -> Result<()>
{
    let mut decoder = match bpxp::Decoder::new(&file)
    {
        Ok(v) => v,
        Err(e) => return Err(Error::Io(ErrorDomain::Installer, e))
    };
    let obj = match decoder.open_metadata()
    {
        Ok(v) => v,
        Err(e) => return Err(Error::Io(ErrorDomain::Installer, e))
    };
    let json = build_package_info(&obj)?;
    if let Err(e) = fs::write(&folder.join("package-info.json"), json::stringify(json))
    {
        return Err(Error::Io(ErrorDomain::Installer, e));
    }
    if let Err(e) = decoder.unpack(&folder)
    {
        return Err(Error::Io(ErrorDomain::Installer, e));
    }
    return Ok(());
}

fn install_dependency(dep: &Dependency, profilemgr: &ProfileManager, registries: &Vec<RegistryInfo>) -> Result<()>
{
    let profile = profilemgr.get_current()?;
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
                        //TODO: Implement profile based verification to ensure package compatibility
                        let folder = profilemgr.get_toolchain_path().join(Path::new(&dep.name));
                        if !folder.exists()
                        {
                            if let Err(e) = fs::create_dir(&folder)
                            {
                                return Err(Error::Io(ErrorDomain::Installer, e));
                            }
                        }
                        registry.download(&folder, &pkg, &file_name)?;
                        unpack_bpx(&folder.join(Path::new(&file_name)), &folder)?;
                        println!("Installed dependency {} - {}", &dep.name, &dep.version);
                        return Ok(());
                    }
                }
                return Err(Error::Generic(ErrorDomain::Installer, format!("The dependency {} - {} is not compatible with your system", &dep.name, &dep.version)));
            }
            continue;
        }
    }
    return Err(Error::Generic(ErrorDomain::Installer, format!("Could not find dependency {} in any registry", &dep.name)));
}

fn is_dependency_installed(dep: &Dependency, profilemgr: &ProfileManager) -> Result<bool>
{
    let profile = profilemgr.get_current()?;
    let package_dir = profilemgr.get_toolchain_path().join(Path::new(&dep.name));
    let path = package_dir.join("package-info.json");
    let mut map = HashMap::new();
    if !path.exists()
    {
        return Ok(false);
    }
    read_property_map(&path, &mut map)?;
    if &map["Platform"] != &profile.platform
        || &map["Architecture"] != &profile.architecture
        || &map["CompilerName"] != &profile.compiler_name
        || &map["Name"] != &dep.name
    {
        //Package is corrupted clear directory
        if let Err(e) = fs::remove_dir_all(&package_dir)
        {
            return Err(Error::Io(ErrorDomain::Installer, e));
        }
        return Ok(false);
    }
    if dep.version != "latest" && &map["Version"] != &dep.version
    {
        //Package is corrupted clear directory
        if let Err(e) = fs::remove_dir_all(&package_dir)
        {
            return Err(Error::Io(ErrorDomain::Installer, e));
        }
        return Ok(false);
    }
    return Ok(true);
}

fn list_configurations(path: &Path) -> io::Result<Vec<String>>
{
    let mut configs = Vec::new();
    let paths = fs::read_dir(&path)?;
    for path in paths
    {
        let motehrfucker = path?;
        if motehrfucker.file_type()?.is_dir()
        {
            let config = motehrfucker.file_name();
            //This will in all cases always return a valid UTF-8 string as BPXP does not support non UTF-8 encoded paths
            //See https://github.com/BlockProject3D/BPX/blob/master/BPX_Format.pdf Section 10.1.1 for more information
            configs.push(config.to_string_lossy().into_owned());
        }
    }
    return Ok(configs);
}

fn list_items(path: &Path) -> io::Result<Vec<String>>
{
    let mut items = Vec::new();
    let paths = fs::read_dir(&path)?;
    for path in paths
    {
        let item = path?.file_name();
        //This will in all cases always return a valid UTF-8 string as BPXP does not support non UTF-8 encoded paths
        //See https://github.com/BlockProject3D/BPX/blob/master/BPX_Format.pdf Section 10.1.1 for more information
        items.push(item.to_string_lossy().into_owned());
    }
    return Ok(items);
}

fn call_generator_lib(package_dir: &Path, package_name: &str, generator: &mut Box<dyn BuildGenerator>) -> Result<()>
{
    let mut lib = Library
    {
        binaries: Vec::new(),
        include_dirs: Vec::new()
    };
    let configs = match list_configurations(&package_dir)
    {
        Ok(v) => v,
        Err(e) => return Err(Error::Io(ErrorDomain::Installer, e))
    };
    for cfg in configs
    {
        let items = match list_items(&package_dir.join(Path::new(&cfg)).join(Path::new("bin")))
        {
            Ok(v) => v,
            Err(e) => return Err(Error::Io(ErrorDomain::Installer, e))
        };
        for item in items
        {
            let mut s = cfg.clone();
            s.push_str("/bin/");
            s.push_str(&item);
            lib.binaries.push(Target
            {
                relative_path: s,
                configuration: cfg.clone()
            });
        }
        let mut s1 = cfg.clone();
        s1.push_str("/include");
        lib.include_dirs.push(Target
        {
            relative_path: s1,
            configuration: cfg.clone()
        })
    }
    generator.add_library(&package_name, lib)?;
    return Ok(());
}

fn call_generator(profilemgr: &ProfileManager, dep: &Dependency, generator: &mut Box<dyn BuildGenerator>) -> Result<()>
{
    let package_dir = profilemgr.get_toolchain_path().join(Path::new(&dep.name));
    let path = package_dir.join("package-info.json");
    let mut map = HashMap::new();
    read_property_map(&path, &mut map)?;
    if map["Type"] == "Library"
    {
        call_generator_lib(&package_dir, &dep.name, generator)?;
    }
    else if map["Type"] == "Framework"
    {
        generator.add_framework(&dep.name)?;
    }
    return Ok(());
}

fn install_sub_directory(path: &Path, toolchain: Option<&str>) -> Result<Vec<String>>
{
    let settings = Settings::new()?;
    let mut res = Vec::new();
    let mut profilemgr = ProfileManager::new(path)?;
    let registries = settings.get_registries();
    let mut file = LuaFile::new();
    let toolchain = match toolchain
    {
        Some(v) => v,
        None => "host"
    };
    file.open_libs(path)?;
    let path = Path::new(path).join("fpkg.lua");
    file.open(&path)?;
    if !profilemgr.exists() || profilemgr.get_toolchain() != toolchain
    {
        let mut props = None;
        if file.has_func_configure()
        {
            props = Some(file.func_configure(toolchain)?);
        }
        profilemgr.install(toolchain, props)?;
    }
    let profile = profilemgr.get_current()?;
    if file.has_func_install()
    {
        let (subprojects, deps, generator_name) = file.func_install(&profile)?;
        for path in subprojects
        {
            res.push(path);
        }
        let mut generator = match generator_name
        {
            None =>
            {
                match create_generator("noop", profilemgr.get_base_path(), profilemgr.get_toolchain())?
                {
                    Some(v) => v,
                    None => return Err(Error::Generic(ErrorDomain::Installer, String::from("Missing base noop generator")))
                }
            },
            Some(name) =>
            {
                match create_generator(&name, profilemgr.get_base_path(), profilemgr.get_toolchain())?
                {
                    Some(v) => v,
                    None => return Err(Error::Generic(ErrorDomain::Installer, format!("No generator named {} found", name)))
                }
            }
        };
        for dep in deps
        {
            if !is_dependency_installed(&dep, &profilemgr)?
            {
                install_dependency(&dep, &profilemgr, &registries)?;
            }
            call_generator(&profilemgr, &dep, &mut generator)?;
            if file.has_func_dep_installed()
            {
                file.func_dep_installed(&dep, &profile)?;
            }
        }
        generator.generate()?;
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

pub fn install(toolchain: Option<&str>) -> Result<()>
{
    let mut directories: Vec<String> = Vec::new();
    directories.push(String::from("."));
    while let Some(dir) = directories.pop()
    {
        check_is_valid_project_dir(Path::new(&dir))?;
        let subdirs = install_sub_directory(Path::new(&dir), toolchain)?;
        for v in subdirs
        {
            directories.push(v);
        }
    }
    return Ok(());
}

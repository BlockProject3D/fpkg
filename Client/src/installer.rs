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

fn build_package_info(obj: &sd::Object) -> Result<json::object::Object>
{
    let mut j = json::object::Object::new();
    if let (Some(platform), Some(arch), Some(cname),
            Some(cversion), Some(name), Some(version),
            Some(typeafs), Some(desc))
            = (obj.get("Platform"), obj.get("Arch"), obj.get("CompilerName"),
               obj.get("CompilerVersion"), obj.get("Name"), obj.get("Version"),
               obj.get("Type"), obj.get("Description"))
    {
        match platform
        {
            sd::Value::String(s) => j.insert("Platform", json::JsonValue::String(s.clone())),
            _ => return Err(Error::Generic(ErrorDomain::Installer, format!("Incorrect type for key 'Platform'")))
        };
        match arch
        {
            sd::Value::String(s) => j.insert("Arch", json::JsonValue::String(s.clone())),
            _ => return Err(Error::Generic(ErrorDomain::Installer, format!("Incorrect type for key 'Arch'")))
        };
        match cname
        {
            sd::Value::String(s) => j.insert("CompilerName", json::JsonValue::String(s.clone())),
            _ => return Err(Error::Generic(ErrorDomain::Installer, format!("Incorrect type for key 'CompilerName'")))
        };
        match cversion
        {
            sd::Value::String(s) => j.insert("CompilerVersion", json::JsonValue::String(s.clone())),
            _ => return Err(Error::Generic(ErrorDomain::Installer, format!("Incorrect type for key 'CompilerVersion'")))
        };
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
    return Err(Error::Generic(ErrorDomain::Installer, String::from("Missing some required properties in BPX package")));
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

fn is_dependency_installed(dep: &Dependency, profile: &Profile) -> Result<bool>
{
    let package_dir = profile.get_platform_path().join(Path::new(&dep.name));
    let path = package_dir.join("package-info.json");
    let mut map = HashMap::new();
    if !path.exists()
    {
        return Ok(false);
    }
    read_property_map(&path, &mut map)?;
    if &map["Platform"] != profile.get("Platform").unwrap()
        || &map["Arch"] != profile.get("Arch").unwrap()
        || &map["CompilerName"] != profile.get("CompilerName").unwrap()
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
        name: String::from(package_name),
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
            s.push('/');
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

fn call_generator(profile: &Profile, dep: &Dependency, generator: &mut Box<dyn BuildGenerator>) -> Result<()>
{
    let package_dir = profile.get_platform_path().join(Path::new(&dep.name));
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

fn install_depenedencies(file: &mut LuaFile, profile: &Profile, registries: &Vec<RegistryInfo>) -> Result<()>
{
    let mut generator = (create_generator("cmake", profile.get_path(), profile.get_platform())?).unwrap(); //TODO: Allow using different generators
    let package = file.read_table()?;
    if let Some(deps) = package.dependencies
    {
        for dep in deps
        {
            if !is_dependency_installed(&dep, &profile)?
            {
                install_dependency(&dep, &profile, &registries)?;
            }
            call_generator(&profile, &dep, &mut generator)?;
        }
    }
    generator.generate()?;
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

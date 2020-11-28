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
use std::path::PathBuf;
use rlua::Lua;
use std::fs;
use std::string::String;
use std::vec::Vec;
use rlua::FromLua;

use crate::builder::Builder;
use crate::builder::Error;
use crate::profile::Profile;

pub struct Compiler
{
    name: String,
    minimum_version: Option<String>,
    versions: Option<Vec<String>>
}

impl FromLua<'_> for Compiler
{
    fn from_lua(val: rlua::Value<'_>, _: rlua::Context<'_>) -> std::result::Result<Self, rlua::Error>
    {
        if let rlua::Value::Table(table) = val
        {
            let name: String = table.get("Name")?;
            let minimum_version: Option<String> = table.get("MinVersion")?;
            let versions: Option<Vec<String>> = table.get("Versions")?;
            return Ok(Compiler
            {
                name: name,
                minimum_version: minimum_version,
                versions: versions
            });
        }
        return Err(rlua::Error::FromLuaConversionError
        {
            from: "Compiler",
            to: "Compiler",
            message: Some(String::from("Could not load table"))
        });
    }
}

pub struct Dependency
{
    name: String,
    version: String
}

impl FromLua<'_> for Dependency
{
    fn from_lua(val: rlua::Value<'_>, _: rlua::Context<'_>) -> std::result::Result<Self, rlua::Error>
    {
        if let rlua::Value::Table(table) = val
        {
            let name: String = table.get("Name")?;
            let version: String = table.get("Version")?;
            return Ok(Dependency
            {
                name: name,
                version: version
            });
        }
        return Err(rlua::Error::FromLuaConversionError
        {
            from: "Dependency",
            to: "Dependency",
            message: Some(String::from("Could not load table"))
        });
    }
}

pub struct PackageTable
{
    name: String,
    description: String,
    version: String,
    configurations: Option<Vec<String>>,
    systems: Option<Vec<String>>,
    architectures: Option<Vec<String>>,
    compilers: Option<Vec<Compiler>>,
    dependencies: Option<Vec<Dependency>>
}

pub struct LuaFile
{
    state: Lua
}

impl LuaFile
{
    pub fn new() -> LuaFile
    {
        return LuaFile
        {
            state: Lua::new()
        };
    }

    pub fn open(&mut self, path: &Path) -> Result<(), Error>
    {
        match fs::read_to_string(path)
        {
            Ok(s) =>
            {
                let res = self.state.context(|ctx|
                {
                    ctx.load(&s).set_name("fpkg.lua")?.exec()?;
                    return Ok(());
                });
                match res
                {
                    Ok(()) => return Ok(()),
                    Err(e) => return Err(Error::Lua(e))
                }
            },
            Err(e) => return Err(Error::Io(e))
        }
    }

    pub fn read_table(&mut self) -> Result<PackageTable, Error>
    {
        let res: rlua::Result<PackageTable> = self.state.context(|ctx|
        {
            let globals = ctx.globals();
            let table: rlua::Table = globals.get("Package")?;
            let name: String = table.get("Name")?;
            let desc: String = table.get("Description")?;
            let version: String = table.get("Version")?;
            let configs: Option<Vec<String>> = table.get("Configurations")?;
            let systems: Option<Vec<String>> = table.get("Platforms")?;
            let archs: Option<Vec<String>> = table.get("Archs")?;
            let compilers: Option<Vec<Compiler>> = table.get("Compilers")?;
            let deps: Option<Vec<Dependency>> = table.get("Dependencies")?;

            return Ok(PackageTable
            {
                name: name,
                description: desc,
                version: version,
                configurations: configs,
                systems: systems,
                architectures: archs,
                compilers: compilers,
                dependencies: deps
            });
        });

        match res
        {
            Ok(v) => return Ok(v),
            Err(e) => return Err(Error::Lua(e))
        }
    }

    pub fn func_build(&mut self, cfg: &str, profile: &Profile) -> Result<i32, Error>
    {
        let res = self.state.context(|ctx|
        {
            let mut tbl = ctx.create_table()?;
            tbl.set("Configuration", cfg)?;
            profile.fill_table(&mut tbl)?;
            let func: rlua::Function = ctx.globals().get("Build")?;
            let res: i32 = func.call(tbl)?;
            return Ok(res);
        });
        match res
        {
            Ok(v) => return Ok(v),
            Err(e) => return Err(Error::Lua(e))
        }
    }
}

fn check_build_configuration(config: &str, configs: &Option<Vec<String>>) -> Result<String, Error>
{
    match configs
    {
        None => return Ok(String::from(config)),
        Some(v) =>
        {
            let cfg = v.iter().find(|v| v == &config || v.to_lowercase() == config);
            match cfg
            {
                None => return Err(Error::Generic(format!("Could not find configuration named {}", config))),
                Some(v) => return Ok(String::from(v))
            }
        }
    }
}

fn check_system(profile: &Profile, systems: &Option<Vec<String>>) -> Result<(), Error>
{
    match systems
    {
        None => return Ok(()),
        Some(v) =>
        {
            let platform = profile.get("Platform").unwrap();
            if !v.iter().any(|e| e == platform)
            {
                return Err(Error::Generic(format!("Unsupported platform {}", platform)));
            }
            return Ok(());
        }
    }
}

fn check_arch(profile: &Profile, archs: &Option<Vec<String>>) -> Result<(), Error>
{
    match archs
    {
        None => return Ok(()),
        Some(v) =>
        {
            let arch = profile.get("Arch").unwrap();
            if !v.iter().any(|e| e == arch)
            {
                return Err(Error::Generic(format!("Unsupported acrhitecture {}", arch)));
            }
            return Ok(());
        }
    }
}

fn check_compiler_version(version: &String, compiler: &Compiler) -> Result<(), Error>
{
    if let Some(minver) = &compiler.minimum_version
    {
        let rep1 = minver.replace('.', "");
        let rep2 = version.replace('.', "");
        if let Ok(value1) = rep1.parse::<usize>()
        {
            if let Ok(value2) = rep2.parse::<usize>()
            {
                if value2 >= value1
                {
                    return Ok(());
                }
                else
                {
                    return Err(Error::Generic(format!("Unsuported compiler version {}", version)));
                }
            }
        }
    }
    if let Some(versions) = &compiler.versions
    {
        if !versions.iter().any(|e| e == version)
        {
            return Err(Error::Generic(format!("Unsuported compiler version {}", version)));
        }
    }
    return Ok(());
}

fn check_compiler(profile: &Profile, compilers: &Option<Vec<Compiler>>) -> Result<(), Error>
{
    match compilers
    {
        None => return Ok(()),
        Some(v) =>
        {
            let compiler = profile.get("CompilerName").unwrap();
            let version = profile.get("CompilerVersion").unwrap();
            match v.iter().find(|v| &v.name == compiler)
            {
                None => return Err(Error::Generic(format!("Unsupported compiler"))),
                Some(cfg) => return check_compiler_version(version, cfg)
            }
        }
    }
}

pub struct LuaBuilder {}

impl Builder for LuaBuilder
{
    fn can_build(&self, path: &Path) -> bool
    {
        let path: PathBuf = [path, Path::new("fpkg.lua")].iter().collect();
        return path.exists();
    }

    fn run_build(&self, config: &str, path: &Path) -> Result<i32, Error>
    {
        let profile = Profile::new(path);
        if !profile.exists()
        {
            return Err(Error::Generic(String::from("Unable to load project profile; did you forget to run fpkg install?")))
        }
        let path: PathBuf = [path, Path::new("fpkg.lua")].iter().collect();
        let mut lua = LuaFile::new();
        lua.open(&path)?;
        let package = lua.read_table()?;
        let acfg = check_build_configuration(config, &package.configurations)?;

        println!("Building {} - {} ({})...", package.name, package.version, package.description);
        check_system(&profile, &package.systems)?;
        check_arch(&profile, &package.architectures)?;
        check_compiler(&profile, &package.compilers)?;
        let res = lua.func_build(&acfg, &profile)?;
        if res != 0
        {
            eprintln!("Build finished with non-zero exit code ({})", res);
        }
        return Ok(res);
    }
}

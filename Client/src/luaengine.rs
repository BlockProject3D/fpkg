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

use std::collections::HashMap;
use std::path::Path;
use rlua::Lua;
use rlua::UserData;
use rlua::UserDataMethods;
use std::fs;
use std::string::String;
use std::vec::Vec;
use rlua::FromLua;

use crate::command;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::common::Result;
use crate::profile::Profile;
use crate::profile::ProfileManager;

pub struct Compiler
{
    pub name: String,
    pub minimum_version: Option<String>,
    pub versions: Option<Vec<String>>
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
    pub name: String,
    pub version: String
}

impl FromLua<'_> for Dependency
{
    fn from_lua(val: rlua::Value<'_>, _: rlua::Context<'_>) -> std::result::Result<Self, rlua::Error>
    {
        if let rlua::Value::Table(table) = val
        {
            let name: String = table.get("Name")?;
            let version: Option<String> = table.get("Version")?;
            return Ok(Dependency
            {
                name: name,
                version: match version
                {
                    Some(v) => v,
                    None => String::from("latest")
                }
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

pub struct ConfiguredTarget
{
    pub path: String,
    pub configuration: String
}

impl FromLua<'_> for ConfiguredTarget
{
    fn from_lua(val: rlua::Value<'_>, _: rlua::Context<'_>) -> std::result::Result<Self, rlua::Error>
    {
        if let rlua::Value::Table(table) = val
        {
            let path: String = table.get(1)?;
            let config: String = table.get(2)?;

            return Ok(ConfiguredTarget
            {
                path: path,
                configuration: config
            });
        }
        return Err(rlua::Error::FromLuaConversionError
        {
            from: "ConfiguredTarget",
            to: "ConfiguredTarget",
            message: Some(String::from("Could not load table"))
        });        
    }
}

pub struct PackageTable
{
    pub name: String,
    pub description: String,
    pub version: String,
    pub configurations: Option<Vec<String>>,
    pub systems: Option<Vec<String>>,
    pub architectures: Option<Vec<String>>,
    pub compilers: Option<Vec<Compiler>>,
    pub dependencies: Option<Vec<Dependency>>
}

pub struct Target
{
    pub typefkjh: String,
    pub includes: Option<Vec<ConfiguredTarget>>,
    pub binaries: Option<Vec<ConfiguredTarget>>,
    pub content: Option<Vec<String>>
}

pub struct PackageConfig
{
    toolchain_name: String,
    pub subprojects: Vec<String>,
    pub props: HashMap<String, String>
}

impl PackageConfig
{
    pub fn new(profilemgr: &ProfileManager) -> PackageConfig
    {
        return PackageConfig
        {
            toolchain_name: String::from(profilemgr.get_toolchain()),
            subprojects: Vec::new(),
            props: HashMap::new()
        };
    }

    pub fn empty() -> PackageConfig
    {
        return PackageConfig
        {
            toolchain_name: String::new(),
            subprojects: Vec::new(),
            props: HashMap::new()
        };
    }
}

impl UserData for PackageConfig
{
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M)
    {
        methods.add_method_mut("SubProject", |_, this, path: String|
        {
            this.subprojects.push(path);
            return Ok(());
        });
        methods.add_method_mut("SetProp", |_, this, (key, val): (String, String)|
        {
            this.props.insert(key, val);
            return Ok(());
        });
        methods.add_method("GetProp", |_, this, key: String|
        {
            return match this.props.get(&key)
            {
                Some(v) => Ok(Some(v.clone())),
                None => Ok(None)
            };
        });
        methods.add_method("GetToolchain", |_, this, ()|
        {
            return Ok(this.toolchain_name.clone());
        });
    }
}

pub struct LuaFile
{
    state: Lua
}

fn run_command_lua(_: rlua::Context<'_>, (exe, args): (String, Vec<String>)) -> rlua::Result<(bool, Option<i32>)>
{
    match command::run_command(&exe, args)
    {
        Ok(v) => return Ok((v.success(), v.code())),
        Err(e) => return Err(rlua::Error::RuntimeError(format!("{}", e)))
    }
}

fn run_command_with_output_lua(_: rlua::Context<'_>, (exe, args): (String, Vec<String>)) -> rlua::Result<String>
{
    match command::run_command_with_output(&exe, args)
    {
        Ok(v) => return Ok(v),
        Err(e) => return Err(rlua::Error::RuntimeError(format!("{}", e)))
    }
}

fn io_isdir(_: rlua::Context<'_>, file: String) -> rlua::Result<bool>
{
    let path = Path::new(&file);
    return Ok(path.is_dir());
}

fn io_list(_: rlua::Context<'_>, file: String) -> rlua::Result<Vec<String>>
{
    let mut v: Vec<String> = Vec::new();
    let path = Path::new(&file);

    match path.read_dir()
    {
        Ok(entries) =>
        {
            for entry in entries
            {
                match entry
                {
                    Ok(vv) => v.push(String::from(vv.path().to_string_lossy().to_owned())),
                    Err(e) => return Err(rlua::Error::RuntimeError(format!("{}", e)))
                }
            }
            return Ok(v);
        },
        Err(e) => return Err(rlua::Error::RuntimeError(format!("{}", e)))
    }
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

    pub fn open(&mut self, path: &Path) -> Result<()>
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
                    Err(e) => return Err(Error::Lua(ErrorDomain::LuaEngine, e))
                }
            },
            Err(e) => return Err(Error::Io(ErrorDomain::LuaEngine, e))
        }
    }

    pub fn open_libs(&mut self) -> Result<()>
    {
        let res = self.state.context(|ctx|
        {
            let tbl = ctx.create_table()?;
            tbl.set("Run", ctx.create_function(run_command_lua)?)?;
            tbl.set("RunWithOutput", ctx.create_function(run_command_with_output_lua)?)?;
            ctx.globals().set("command", tbl)?;
            let io = ctx.create_table()?;
            io.set("IsDirectory", ctx.create_function(io_isdir)?)?;
            io.set("List", ctx.create_function(io_list)?)?;
            ctx.globals().set("file", io)?;
            return Ok(());
        });
        match res
        {
            Ok(()) => return Ok(()),
            Err(e) => return Err(Error::Lua(ErrorDomain::LuaEngine, e))
        }
    }

    pub fn read_table(&mut self) -> Result<PackageTable>
    {
        let res: rlua::Result<PackageTable> = self.state.context(|ctx|
        {
            let globals = ctx.globals();
            let table: rlua::Table = globals.get("PackageInfo")?;
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
            Err(e) => return Err(Error::Lua(ErrorDomain::LuaEngine, e))
        }
    }

    pub fn has_func_install(&self) -> bool
    {
        let res = self.state.context(|ctx|
        {
            return ctx.globals().contains_key("Install");
        });
        match res
        {
            Ok(v) => return v,
            Err(_) => return false
        }
    }

    pub fn has_func_configure(&self) -> bool
    {
        let res = self.state.context(|ctx|
        {
            return ctx.globals().contains_key("Configure");
        });
        match res
        {
            Ok(v) => return v,
            Err(_) => return false
        }
    }

    pub fn has_func_build(&self) -> bool
    {
        let res = self.state.context(|ctx|
        {
            return ctx.globals().contains_key("Build");
        });
        match res
        {
            Ok(v) => return v,
            Err(_) => return false
        }
    }

    pub fn func_install(&mut self, profile: &Profile) -> Result<Option<HashMap<String, String>>>
    {
        let res = self.state.context(|ctx|
        {
            let mut tbl = ctx.create_table()?;
            profile.fill_table(&mut tbl)?;
            let func: rlua::Function = ctx.globals().get("Install")?;
            let res: Option<HashMap<String, String>> = func.call(tbl)?;

            match res
            {
                Some(v) => return Ok(Some(v)),
                None => return Ok(None)
            }
        });
        match res
        {
            Ok(v) => return Ok(v),
            Err(e) => return Err(Error::Lua(ErrorDomain::LuaEngine, e))
        }
    }

    pub fn func_configure(&mut self, profilemgr: &ProfileManager) -> Result<PackageConfig>
    {
        let res = self.state.context(|ctx|
        {
            let userdata = ctx.scope(|scope|
            {
                let userdata = scope.create_static_userdata(PackageConfig::new(profilemgr))?;
                let func: rlua::Function = ctx.globals().get("Configure")?;
                func.call(userdata)?;
                return Ok(userdata);
            })?;
            let data = std::mem::replace(&mut userdata.borrow_mut()?, PackageConfig::empty());
            return Ok(data);
        });
        match res
        {
            Ok(v) => return Ok(v),
            Err(e) => return Err(Error::Lua(ErrorDomain::LuaEngine, e))
        }
    }

    pub fn func_build(&mut self, cfg: &str, profile: &Profile) -> Result<i32>
    {
        let res = self.state.context(|ctx|
        {
            let mut tbl = ctx.create_table()?;
            tbl.set("Configuration", cfg)?;
            profile.fill_table(&mut tbl)?;
            let func: rlua::Function = ctx.globals().get("Build")?;
            let res: Option<i32> = func.call(tbl)?;
            match res
            {
                Some(v) => return Ok(v),
                None => return Ok(0)
            }
        });
        match res
        {
            Ok(v) => return Ok(v),
            Err(e) => return Err(Error::Lua(ErrorDomain::LuaEngine, e))
        }
    }

    pub fn func_package(&mut self, profile: &Profile) -> Result<Option<Target>>
    {
        let res = self.state.context(|ctx|
        {
            let mut tbl = ctx.create_table()?;
            profile.fill_table(&mut tbl)?;
            let func: rlua::Function = ctx.globals().get("Package")?;
            let res: Option<rlua::Table> = func.call(tbl)?;

            match res
            {
                Some(v) =>
                {
                    let typedas: String = v.get("Type")?;
                    let bins: Option<Vec<ConfiguredTarget>> = v.get("Binaries")?;
                    let incs: Option<Vec<ConfiguredTarget>> = v.get("Includes")?;
                    let cnt: Option<Vec<String>> = v.get("Content")?;

                    return Ok(Some(Target
                    {
                        typefkjh: typedas,
                        binaries: bins,
                        includes: incs,
                        content: cnt
                    }));
                },
                None => return Ok(None)
            }
        });
        match res
        {
            Ok(v) => return Ok(v),
            Err(e) => return Err(Error::Lua(ErrorDomain::LuaEngine, e))
        }
    }
}

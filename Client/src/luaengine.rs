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
use std::str;
use std::string::String;
use std::vec::Vec;
use rlua::FromLua;
use core::cell::RefMut;

use crate::common::Error;
use crate::common::ErrorDomain;
use crate::common::Result;
use crate::profile::Profile;
use crate::lualibfile::open_libfile;
use crate::lualibcommand::open_libcommand;
use crate::lualibstring::open_libstring;

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
    pub compilers: Option<Vec<Compiler>>
}

pub struct Target
{
    pub typefkjh: String,
    pub includes: Option<Vec<ConfiguredTarget>>,
    pub binaries: Option<Vec<ConfiguredTarget>>,
    pub content: Option<Vec<String>>
}

struct ToolchainConfig
{
    toolchain_name: String,
    props: HashMap<String, String>
}

impl ToolchainConfig
{
    pub fn new(toolchain_name: &str) -> ToolchainConfig
    {
        return ToolchainConfig
        {
            toolchain_name: String::from(toolchain_name),
            props: HashMap::new()
        };
    }
}

impl UserData for &mut ToolchainConfig
{
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M)
    {
        methods.add_method_mut("Set", |_, this, (key, val): (String, String)|
        {
            this.props.insert(key, val);
            return Ok(());
        });
        methods.add_method("Get", |_, this, key: String|
        {
            return match this.props.get(&key)
            {
                Some(v) => Ok(Some(v.clone())),
                None => Ok(None)
            };
        });
        methods.add_method("GetName", |_, this, ()|
        {
            return Ok(this.toolchain_name.clone());
        });
    }
}

struct InstallTool
{
    subprojects: Vec<String>,
    dependencies: Vec<Dependency>,
    generator: Option<String>
}

impl InstallTool
{
    pub fn new() -> InstallTool
    {
        return InstallTool
        {
            subprojects: Vec::new(),
            dependencies: Vec::new(),
            generator: None
        };
    }
}

impl UserData for &mut InstallTool
{
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M)
    {
        methods.add_method_mut("AddSubproject", |_, this, path: String|
        {
            this.subprojects.push(path);
            return Ok(());
        });
        methods.add_method_mut("AddDependency", |_, this, (name, version): (String, String)|
        {
            this.dependencies.push(Dependency
            {
                name: name,
                version: version
            });
            return Ok(());
        });
        methods.add_method_mut("SetGenerator", |_, this, name: String|
        {
            this.generator = Some(name);
            return Ok(());
        });
    }
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
            let globals = ctx.globals();
            //Blacklist start
            let os = globals.get::<_, rlua::Table>("os")?;
            os.set("execute", rlua::Value::Nil)?;
            os.set("exit", rlua::Value::Nil)?;
            os.set("remove", rlua::Value::Nil)?;
            os.set("rename", rlua::Value::Nil)?;
            os.set("setlocale", rlua::Value::Nil)?;
            os.set("tmpname", rlua::Value::Nil)?;
            //Remove the io lib as the library contains APIs that could conflict with Rust or FPKG
            globals.set("io", rlua::Value::Nil)?;
            //Remove the debug lib I don't like the following "Several of its functions violate basic assumptions about Lua code" on https://www.lua.org/manual/5.4/manual.html#6.8
            //I don't things that violate safety.
            globals.set("debug", rlua::Value::Nil)?;
            //Apparently dofile can read from stdin which may cause some conflicts so get rid of it
            globals.set("dofile", rlua::Value::Nil)?;
            //"running maliciously crafted bytecode can crash the interpreter" no thanks I'm not a fan of segfaults, etc... Why would I use Rust otherwise?
            globals.set("load", rlua::Value::Nil)?;
            //Nice it goes with the previous one and even better it wants to conflict with stdin, well then you know where to go crappy unsafe function
            globals.set("loadfile", rlua::Value::Nil)?;
            //Will replace this one with a log library in order to integrate properly with Rust
            globals.set("warn", rlua::Value::Nil)?;
            //The package lib smells. The package.loadlib function screams "Hey, I'm here hijack me! Inject me! I'm a crack-ny-***!"
            globals.set("package", rlua::Value::Nil)?;
            //This function goes with package lib so get rid of it as well. Don't worry this one will have a replacement
            globals.set("require", rlua::Value::Nil)?;
            //Blacklist end
            open_libfile(ctx)?;
            open_libcommand(ctx)?;
            open_libstring(ctx)?;
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

            return Ok(PackageTable
            {
                name: name,
                description: desc,
                version: version,
                configurations: configs,
                systems: systems,
                architectures: archs,
                compilers: compilers
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

    fn func_install_internal(&mut self, tool: &mut InstallTool, profile: &Profile) -> rlua::Result<()>
    {
        return self.state.context(|ctx|
        {
            ctx.scope(|scope|
            {
                let userdata = scope.create_nonstatic_userdata(tool)?;
                let mut tbl = ctx.create_table()?;
                profile.fill_table(&mut tbl)?;
                let func: rlua::Function = ctx.globals().get("Install")?;
                func.call((tbl, userdata))?;
                return Ok(());
            })?;
            return Ok(());
        });
    }

    pub fn func_install(&mut self, profile: &Profile) -> Result<(Vec<String>, Vec<Dependency>, Option<String>)>
    {
        let mut tool = InstallTool::new();
        if let Err(e) = self.func_install_internal(&mut tool, profile)
        {
            return Err(Error::Lua(ErrorDomain::LuaEngine, e));
        }
        return Ok((tool.subprojects, tool.dependencies, tool.generator));
    }

    fn func_configure_internal(&mut self, toolchain: &mut ToolchainConfig) -> rlua::Result<()>
    {
        return self.state.context(|ctx|
        {
            ctx.scope(|scope|
            {
                let userdata = scope.create_nonstatic_userdata(toolchain)?;
                let func: rlua::Function = ctx.globals().get("Configure")?;
                func.call(userdata)?;
                return Ok(());
            })?;
            return Ok(());
        });
    }

    pub fn func_configure(&mut self, toolchain_name: &str) -> Result<HashMap<String, String>>
    {
        let mut toolchain = ToolchainConfig::new(toolchain_name);
        if let Err(e) = self.func_configure_internal(&mut toolchain)
        {
            return Err(Error::Lua(ErrorDomain::LuaEngine, e));
        }
        return Ok(toolchain.props);
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

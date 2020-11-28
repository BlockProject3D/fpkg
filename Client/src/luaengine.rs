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
use rlua::Lua;
use std::fs;
use std::string::String;
use std::vec::Vec;
use rlua::FromLua;

use crate::command;
use crate::builder::Error;
use crate::profile::Profile;

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
    pub name: String,
    pub description: String,
    pub version: String,
    pub configurations: Option<Vec<String>>,
    pub systems: Option<Vec<String>>,
    pub architectures: Option<Vec<String>>,
    pub compilers: Option<Vec<Compiler>>,
    pub dependencies: Option<Vec<Dependency>>
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

    pub fn open_libs(&mut self) -> Result<(), Error>
    {
        let res = self.state.context(|ctx|
        {
            let tbl = ctx.create_table()?;
            tbl.set("Run", ctx.create_function(run_command_lua)?)?;
            tbl.set("RunOutput", ctx.create_function(run_command_with_output_lua)?)?;
            ctx.globals().set("command", tbl)?;
            return Ok(());
        });
        match res
        {
            Ok(()) => return Ok(()),
            Err(e) => return Err(Error::Lua(e))
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

    pub fn func_build(&mut self, cfg: &str, profile: &Profile) -> Result<i32, Error>
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
            Err(e) => return Err(Error::Lua(e))
        }
    }
}

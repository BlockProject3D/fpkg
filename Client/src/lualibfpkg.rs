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

use rlua::Context;
use rlua::Result;
use rlua::Error;
use rlua::Table;
use rlua::Value;
use std::path::Path;
use std::fs::read_to_string;

use crate::luaengine::Compiler;

fn fpkg_project(ctx: Context<'_>, table: Table) -> Result<()>
{
    if ctx.globals().contains_key("Project")?
    {
        return Err(Error::RuntimeError(String::from("Attempt to re-define project table")))
    }
    let name: String = table.get("name")?;
    let desc: String = table.get("description")?;
    let version: String = table.get("version")?;
    let configs: Option<Vec<String>> = table.get("configurations")?;
    let systems: Option<Vec<String>> = table.get("platforms")?;
    let archs: Option<Vec<String>> = table.get("architectures")?;
    let compilers: Option<Vec<Compiler>> = table.get("compilers")?;
    let meta = ctx.create_table()?;
    let copy = ctx.create_table()?;
    let project_home: String = ctx.named_registry_value("PROJECT_HOME")?;
    meta.set("home", project_home)?;
    meta.set("name", name.clone())?;
    meta.set("description", desc.clone())?;
    meta.set("version", version.clone())?;
    meta.set("configurations", configs.clone())?;
    meta.set("platforms", systems.clone())?;
    meta.set("architectures", archs.clone())?;
    meta.set("compilers", compilers.clone())?;
    copy.set("name", name)?;
    copy.set("description", desc)?;
    copy.set("version", version)?;
    copy.set("configurations", configs)?;
    copy.set("platforms", systems)?;
    copy.set("architectures", archs)?;
    copy.set("compilers", compilers)?;
    ctx.globals().set("Project", meta)?;
    ctx.set_named_registry_value("Project", copy)?;
    return Ok(());
}

fn fpkg_add_lua_path(ctx: Context<'_>, asbdh: String) -> Result<()>
{
    let path: Table = ctx.named_registry_value("LUA_PATH")?;
    path.set(path.len()? + 1, asbdh)?;
    return Ok(());
}

fn fpkg_get_lua_path(ctx: Context<'_>, (): ()) -> Result<Table>
{
    let path: Table = ctx.named_registry_value("LUA_PATH")?;
    return Ok(path.clone());
}

fn dofile(ctx: Context<'_>, file: String) -> Result<Value>
{
    match read_to_string(&file)
    {
        Ok(content) =>
        {
            let v = ctx.load(&content).set_name(file.as_bytes())?.call(())?;
            return Ok(v);
        },
        Err(e) => return Err(Error::RuntimeError(format!("IO error: {}", e)))
    };
}

fn base_require(ctx: Context<'_>, module_name: String) -> Result<Value>
{
    if module_name.ends_with(".lua") //Load a standalone Lua file
    {
        return dofile(ctx, module_name);
    }
    let project_home: String = ctx.named_registry_value("PROJECT_HOME")?;
    let path: Table = ctx.named_registry_value("LUA_PATH")?;
    for i in 1..path.len()? + 1  //Search the path for a corresponding Lua file
    {
        let s: String = path.get(i)?;
        let mut file: String = [&s, "/", &module_name, ".lua"].concat().replace("%PROJECT_HOME%", &project_home);
        if let Some(dir) = dirs::home_dir()
        {
            file = file.replace("%USER_HOME%", &dir.to_string_lossy());
        }
        if Path::new(&file).exists()
        {
            return dofile(ctx, file);
        }
    }
    return Err(Error::RuntimeError(format!("Could not find module {} in search path", module_name)));
}

pub fn open_libfpkg(ctx: Context<'_>) -> Result<()>
{
    let path = ctx.create_table()?;
    path.set(1, "%PROJECT_HOME%/.fpkg/lua")?;
    ctx.set_named_registry_value("LUA_PATH", path)?;
    let tbl = ctx.create_table()?;
    tbl.set("project", ctx.create_function(fpkg_project)?)?;
    tbl.set("addLuaPath", ctx.create_function(fpkg_add_lua_path)?)?;
    tbl.set("getLuaPath", ctx.create_function(fpkg_get_lua_path)?)?;
    ctx.globals().set("fpkg", tbl)?;
    ctx.globals().set("require", ctx.create_function(base_require)?)?;
    return Ok(());
}

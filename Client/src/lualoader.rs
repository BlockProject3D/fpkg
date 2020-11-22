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

use crate::builder::Builder;
use crate::builder::Error;

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

    pub fn func_build(&mut self) -> Result<i32, Error>
    {
        let res = self.state.context(|ctx|
        {
            let func: rlua::Function = ctx.globals().get("Build")?;
            let res: i32 = func.call(())?;
            return Ok(res);
        });
        match res
        {
            Ok(v) => return Ok(v),
            Err(e) => return Err(Error::Lua(e))
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
        let path: PathBuf = [path, Path::new("fpkg.lua")].iter().collect();
        let mut lua = LuaFile::new();

        lua.open(&path)?;
        return lua.func_build();
    }
}

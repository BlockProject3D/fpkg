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
use hlua::Lua;
use hlua::LuaFunction;
use std::fs;

use crate::builder::Builder;
use crate::builder::Error;

pub struct LuaFile<'a>
{
    state: Lua<'a>
}

impl <'a> LuaFile<'a>
{
    pub fn new() -> LuaFile<'a>
    {
        let mut a = LuaFile
        {
            state: Lua::new()
        };

        a.state.openlibs();
        return a;
    }

    pub fn open(&mut self, path: &Path) -> Result<(), Error>
    {
        match fs::read_to_string(path)
        {
            Ok(s) =>
            {
                match self.state.execute::<()>(&s)
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
        let build: Option<LuaFunction<_>> = self.state.get("Build");

        match build
        {
            Some(mut func) =>
            {
                match func.call::<i32>()
                {
                    Ok(v) => return Ok(v),
                    Err(e) => return Err(Error::Lua(e))
                }
            },
            None => return Err(Error::Generic(String::from("No function named Build found in Lua script")))
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

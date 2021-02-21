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

use std::vec::Vec;
use rlua::Context;
use rlua::Result;
use rlua::Error;

use crate::command;

fn run_command_lua(_: Context<'_>, (exe, args): (String, Vec<String>)) -> Result<(bool, Option<i32>)>
{
    match command::run_command(&exe, args)
    {
        Ok(v) => return Ok((v.success(), v.code())),
        Err(e) => return Err(Error::RuntimeError(format!("{}", e)))
    }
}

fn run_command_with_output_lua(_: Context<'_>, (exe, args): (String, Vec<String>)) -> Result<String>
{
    match command::run_command_with_output(&exe, args)
    {
        Ok(v) => return Ok(v),
        Err(e) => return Err(Error::RuntimeError(format!("{}", e)))
    }
}

pub fn open_libcommand(ctx: Context<'_>) -> Result<()>
{
    let tbl = ctx.create_table()?;
    tbl.set("run", ctx.create_function(run_command_lua)?)?;
    tbl.set("runPiped", ctx.create_function(run_command_with_output_lua)?)?;
    ctx.globals().set("command", tbl)?;
    return Ok(());
}

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
use rlua::Variadic;
use rlua::Value;
use rlua::Function;
use std::string::String;
use std::vec::Vec;

fn lua_format(format: String, args: Vec<String>) -> String
{
    let mut idx: usize = 0;
    let mut prevc = '\0';
    let mut ignore = false;
    let mut finalstr = String::with_capacity(format.len());
    for c in format.chars()
    {
        if c == '\\'
        {
            ignore = true;
        }
        else
        {
            if ignore
            {
                finalstr.push(c);
                ignore = false;
            }
            else
            {
                if c == '}' && prevc == '{'
                {
                    //take str at idx
                    finalstr.push_str(&args[idx]);
                    idx += 1;
                }
                else if c != '{'
                {
                    finalstr.push(c);
                }
                prevc = c;
            }
        }
    }
    return finalstr;
}

pub fn open_liblog(ctx: Context<'_>) -> Result<()>
{
    let info = ctx.create_function(|ctx, (format, args): (String, Variadic<Value>)|
    {
        let tostring: Function = ctx.globals().get("tostring")?;
        let mut v: Vec<String> = Vec::new();
        for val in args
        {
            let s: String = tostring.call(val)?;
            v.push(s);
        }
        let text = lua_format(format, v);
        println!("[Lua] {}", text);
        return Ok(());
    })?;
    let warn = ctx.create_function(|ctx, (format, args): (String, Variadic<Value>)|
    {
        let tostring: Function = ctx.globals().get("tostring")?;
        let mut v: Vec<String> = Vec::new();
        for val in args
        {
            let s: String = tostring.call(val)?;
            v.push(s);
        }
        let text = lua_format(format, v);
        println!("[Lua] Warning: {}", text);
        return Ok(());
    })?;
    let err = ctx.create_function(|ctx, (format, args): (String, Variadic<Value>)|
    {
        let tostring: Function = ctx.globals().get("tostring")?;
        let mut v: Vec<String> = Vec::new();
        for val in args
        {
            let s: String = tostring.call(val)?;
            v.push(s);
        }
        let text = lua_format(format, v);
        eprintln!("[Lua] Error: {}", text);
        return Ok(());
    })?;
    let print = ctx.create_function(|ctx, args: Variadic<Value>|
    {
        let tostring: Function = ctx.globals().get("tostring")?;
        let mut v = String::new();
        for val in args
        {
            let s: String = tostring.call(val)?;
            v.push_str(&s);
        }
        println!("[Lua] {}", v);
        return Ok(());
    })?;
    //Replace the print function in order to redirect lua stdout to rust stdout
    ctx.globals().set("print", print)?;
    let log = ctx.create_table()?;
    log.set("info", info)?;
    log.set("warning", warn)?;
    log.set("error", err)?;
    ctx.globals().set("log", log)?;
    return Ok(());
}
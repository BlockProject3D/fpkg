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

use std::str;
use rlua::Context;
use rlua::Result;
use rlua::Error;

fn string_frombytes(_: Context<'_>, (buf, offset, size): (Vec<u8>, usize, usize)) -> Result<String>
{
    let s = match str::from_utf8(&buf[offset..size])
    {
        Ok(v) => v,
        Err(e) => return Err(Error::RuntimeError(format!("Invalid UTF-8 sequence: {}", e)))
    };
    return Ok(String::from(s));
}

fn string_tobytes(_: Context<'_>, (s, offset, size): (String, usize, usize)) -> Result<Vec<u8>>
{
    let bytes = (&s[offset..size]).as_bytes().to_vec();
    return Ok(bytes);
}

pub fn open_libstring(ctx: Context<'_>) -> Result<()>
{
    let string = ctx.globals().get::<_, rlua::Table>("string")?;
    string.set("FromBytes", ctx.create_function(string_frombytes)?)?;
    string.set("ToBytes", ctx.create_function(string_tobytes)?)?;
    return Ok(());
}

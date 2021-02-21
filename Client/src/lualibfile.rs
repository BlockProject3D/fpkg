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

use std::fs;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::string::String;
use std::vec::Vec;
use rlua::Context;
use rlua::Result;
use rlua::Error;
use rlua::UserData;
use rlua::AnyUserData;
use rlua::UserDataMethods;

struct WrappedFile
{
    file: Option<File>
}

impl UserData for WrappedFile
{
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M)
    {
        methods.add_method_mut("read", |_, this, len: usize|
        {
            if let Some(file) = &mut this.file
            {
                let mut v: Vec<u8> = Vec::with_capacity(len);
                let bytes = match file.read(&mut v)
                {
                    Err(e) => return Err(Error::RuntimeError(format!("IO error: {}", e))),
                    Ok(v) => v
                };
                return Ok((v, bytes));
            }
            return Err(Error::RuntimeError(String::from("Attempt to use a closed file")));
        });
        methods.add_method_mut("write", |_, this, data: Vec<u8>|
        {
            if let Some(file) = &mut this.file
            {
                let len = match file.write(&data)
                {
                    Err(e) => return Err(Error::RuntimeError(format!("IO error: {}", e))),
                    Ok(v) => v
                };
                return Ok(len);
            }
            return Err(Error::RuntimeError(String::from("Attempt to use a closed file")));
        });
        methods.add_method_mut("close", |_, this, ()|
        {
            let fileres = std::mem::replace(&mut this.file, None);
            if let Some(file) = fileres
            {
                drop(file);
                return Ok(());
            }
            return Err(Error::RuntimeError(String::from("Attempt to use a closed file")));
        });
    }
}

fn file_isdir(_: Context<'_>, file: String) -> Result<bool>
{
    let path = Path::new(&file);
    return Ok(path.is_dir());
}

fn file_list(_: Context<'_>, file: String) -> Result<Vec<String>>
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
                    Err(e) => return Err(Error::RuntimeError(format!("IO error: {}", e)))
                }
            }
            return Ok(v);
        },
        Err(e) => return Err(Error::RuntimeError(format!("IO error: {}", e)))
    }
}

fn file_size(_: Context<'_>, path: String) -> Result<u64>
{
    let metadata = match fs::metadata(Path::new(&path))
    {
        Err(e) => return Err(Error::RuntimeError(format!("IO error: {}", e))),
        Ok(v) => v
    };
    return Ok(metadata.len());
}

fn file_rename(_: Context<'_>, (old_path, new_path): (String, String)) -> Result<()>
{
    match fs::rename(Path::new(&old_path), Path::new(&new_path))
    {
        Err(e) => return Err(Error::RuntimeError(format!("IO error: {}", e))),
        Ok(()) => return Ok(())
    };
}

fn file_open(ctx: Context<'_>, (path, mode): (String, String)) -> Result<AnyUserData<'_>>
{
    let mut useless = OpenOptions::new();
    let options = useless
        .read(mode.find('r').is_some())
        .write(mode.find('w').is_some())
        .create(mode.find('w').is_some())
        .append(mode.find('a').is_some())
        .truncate(mode.find('t').is_some());
    let f = match options.open(Path::new(&path))
    {
        Err(e) => return Err(Error::RuntimeError(format!("IO error: {}", e))),
        Ok(v) => v
    };
    let userdata = WrappedFile
    {
        file: Some(f)
    };
    return ctx.create_userdata(userdata);
}

pub fn open_libfile(ctx: Context<'_>) -> Result<()>
{
    let file = ctx.create_table()?;
    file.set("isDirectory", ctx.create_function(file_isdir)?)?;
    file.set("list", ctx.create_function(file_list)?)?;
    file.set("rename", ctx.create_function(file_rename)?)?;
    file.set("size", ctx.create_function(file_size)?)?;
    file.set("open", ctx.create_function(file_open)?)?;
    ctx.globals().set("file", file)?;
    return Ok(());
}

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

use std::string::String;
use std::path::Path;

use crate::profile::Profile;
use crate::builder;
use crate::common::Result;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::luaengine::LuaFile;

fn run_lua_engine(path: &Path, profile: &Profile) -> Result<Option<Vec<String>>>
{
    let mut file = LuaFile::new();
    file.open_libs()?;
    file.open(&path)?;
    if file.has_func_install()
    {
        return file.func_install(&profile);
    }
    else
    {
        return Ok(None);
    }
}

fn install_sub_directory(path: &Path, platform: Option<&str>) -> Result<Vec<String>>
{
    let mut res = Vec::new();
    let mut profile = Profile::new(path)?;

    match Profile::mkdir(path)
    {
        Err(e) => return Err(Error::Io(ErrorDomain::Installer, e)),
        _ => ()
    }
    if !profile.exists()
    {
        match platform
        {
            Some(name) => profile.regenerate_cross(name)?,
            None => profile.regenerate_self()?
        }
    }
    match profile.write()
    {
        Err(e) => return Err(Error::Io(ErrorDomain::Installer, e)),
        _ => ()
    }
    let path = Path::new(path).join("fpkg.lua");
    if path.exists()
    {
        if let Some(vc) = run_lua_engine(&path, &profile)?
        {
            for path in vc
            {
                res.push(path);
            }
        }
    }
    //TODO: Implement dependency/framework downloader/installer and connect it right here
    return Ok(res);
}

fn check_is_valid_project_dir(path: &Path) -> Result<()>
{
    let builder = builder::find_builder(&path);
    if builder.is_none() {
        return Err(Error::Generic(ErrorDomain::Installer, String::from("Project directory does not contain a valid project file")));
    }
    return Ok(());
}

pub fn install(platform: Option<&str>) -> Result<()>
{
    let mut directories: Vec<String> = Vec::new();
    directories.push(String::from("."));
    while let Some(dir) = directories.pop()
    {
        check_is_valid_project_dir(Path::new(&dir))?;
        let subdirs = install_sub_directory(Path::new(&dir), platform)?;
        for v in subdirs
        {
            directories.push(v);
        }
    }
    return Ok(());
}

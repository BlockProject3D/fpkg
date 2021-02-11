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

use std::path::Path;
use std::path::PathBuf;
use std::string::String;

use crate::luaengine::LuaFile;
use crate::luaengine::PackageTable;
use crate::profile::Profile;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::common::Result;

fn get_pk_file(profile: &Profile) -> String
{
    let mut s = String::from("build-");

    s.push_str(profile.get("Platform").unwrap());
    s.push('-');
    s.push_str(profile.get("Arch").unwrap());
    s.push('-');
    s.push_str(profile.get("CompilerName").unwrap());
    s.push_str(profile.get("CompilerVersion").unwrap());
    s.push_str(".bpx");
    return s;
}

pub fn publish(path: &Path, registry: Option<&str>) -> Result<i32>
{
    let profile = Profile::new(path)?;
    if !profile.exists()
    {
        return Err(Error::Generic(ErrorDomain::Publisher, String::from("Unable to load project profile; did you forget to run fpkg install?")));
    }
    let p: PathBuf = [path, Path::new("fpkg.lua")].iter().collect();
    let mut lua = LuaFile::new();
    lua.open_libs()?;
    lua.open(&p)?;
    let package = lua.read_table()?;
    let file_name = get_pk_file(&profile);
    
    return Ok(0);
}
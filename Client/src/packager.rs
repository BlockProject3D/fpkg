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
use bpx::bpxp;
use bpx::sd;
use std::io;
use std::fs::metadata;

use crate::luaengine::LuaFile;
use crate::luaengine::PackageTable;
use crate::luaengine::Target;
use crate::common::Error;
use crate::common::Result;
use crate::profile::Profile;
use crate::builder::check_build_configuration;

fn new_pk(profile: &Profile) -> io::Result<bpxp::Encoder>
{
    //Format = build-<name of platform>-<architecture>-<compiler id><compiler version>.bpx
    let mut s = String::from("build-");

    s.push_str(profile.get("Platform").unwrap());
    s.push('-');
    s.push_str(profile.get("Arch").unwrap());
    s.push('-');
    s.push_str(profile.get("CompilerName").unwrap());
    s.push_str(profile.get("CompilerVersion").unwrap());
    s.push_str(".bpx");
    return bpxp::Encoder::new(Path::new(&s));
}

fn get_vname(cfg: String, subdir: &str, path: &Path) -> io::Result<String>
{
    let md = metadata(path)?;
    let mut init = cfg;

    init.push('/');
    init.push_str(subdir);
    if md.is_file()
    {
        init.push('/');
        init.push_str(&bpx::strings::get_name_from_path(path)?);
        return Ok(init);
    }
    else
    {
        return Ok(init);
    }
}

fn pack_lib(bpx: &mut bpxp::Encoder, target: &Target, package: &PackageTable) -> Result<()>
{
    if let Some(incs) = &target.includes
    {
        for inc in incs
        {
            let cfg = check_build_configuration(&inc.configuration, &package.configurations)?;
            let path = Path::new(&inc.path);
            let vname = match get_vname(cfg, "include", &path)
            {
                Ok(vname) => vname,
                Err(e) => return Err(Error::Io(e))
            };
            if let Err(e) = bpx.pack_vname(path, &vname)
            {
                return Err(Error::Io(e));
            }
        }
    }
    if let Some(bins) = &target.binaries
    {
        for bin in bins
        {
            let cfg = check_build_configuration(&bin.configuration, &package.configurations)?;
            let path = Path::new(&bin.path);
            let vname = match get_vname(cfg, "bin", &path)
            {
                Ok(vname) => vname,
                Err(e) => return Err(Error::Io(e))
            };
            if let Err(e) = bpx.pack_vname(path, &vname)
            {
                return Err(Error::Io(e));
            }
        }
    }
    return Ok(());
}

fn pack_framework(bpx: &mut bpxp::Encoder, target: &Target) -> Result<()>
{
    if let Some(files) = &target.content
    {
        for file in files
        {
            if let Err(e) = bpx.pack(Path::new(&file))
            {
                return Err(Error::Io(e));
            }
        }
    }
    return Ok(());
}

fn set_type_ext(bpx: &mut bpxp::Encoder, profile: &Profile)
{
    let platform = profile.get("Platform").unwrap();
    let arch = profile.get("Arch").unwrap();

    if platform == "Linux"
    {
        bpx.platform = bpxp::Platform::Linux;
    }
    else if platform == "OSX"
    {
        bpx.platform = bpxp::Platform::Mac;
    }
    else if platform == "Windows"
    {
        bpx.platform = bpxp::Platform::Windows;
    }
    else if platform == "Android"
    {
        bpx.platform = bpxp::Platform::Android;
    }
    if arch == "x86"
    {
        bpx.architecture = bpxp::Architecture::X86;
    }
    else if arch == "x86_64"
    {
        bpx.architecture = bpxp::Architecture::X86_64;
    }
    else if arch == "arm"
    {
        bpx.architecture = bpxp::Architecture::Armv7hl;
    }
    else if arch == "aarch64"
    {
        bpx.architecture = bpxp::Architecture::Aarch64;
    }
}

pub fn package(path: &Path) -> Result<i32>
{
    let profile = Profile::new(path)?;
    if !profile.exists()
    {
        return Err(Error::Generic(String::from("Unable to load project profile; did you forget to run fpkg install?")));
    }
    let p: PathBuf = [path, Path::new("fpkg.lua")].iter().collect();
    let mut lua = LuaFile::new();
    lua.open_libs()?;
    lua.open(&p)?;
    let package = lua.read_table()?;
    println!("Packaging {} - {} ({}) with Lua Engine...", package.name, package.version, package.description);
    if let Some(target) = lua.func_package(&profile)?
    {
        let mut pk = match new_pk(&profile)
        {
            Ok(v) => v,
            Err(e) => return Err(Error::Io(e))
        };
        let mut obj = sd::Object::new();
        set_type_ext(&mut pk, &profile);
        obj.set("Name", sd::Value::String(package.name.clone()));
        obj.set("Version", sd::Value::String(package.version.clone()));
        obj.set("Description", sd::Value::String(package.description.clone()));
        profile.fill_structured_data(&mut obj);
        obj.add_debug_info();
        if let Err(e) = pk.add_metadata(&obj)
        {
            return Err(Error::Io(e));
        }
        if target.typefkjh == "Library"
        {
            //Package a library
            pack_lib(&mut pk, &target, &package)?;
        }
        else if target.typefkjh == "Framework"
        {
            //Package a framework
            pack_framework(&mut pk, &target)?;
        }
        if let Err(e) = pk.save()
        {
            return Err(Error::Io(e));
        }
        return Ok(0)
    }
    eprintln!("WARNING: Nothing to package!");
    return Ok(2);
}

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

use crate::builder::Builder;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::common::Result;
use crate::profile::Profile;
use crate::profile::ProfileManager;
use crate::luaengine::Compiler;
use crate::luaengine::LuaFile;
use crate::builder::check_build_configuration;

fn check_system(profile: &Profile, systems: &Option<Vec<String>>) -> Result<()>
{
    match systems
    {
        None => return Ok(()),
        Some(v) =>
        {
            let platform = &profile.platform;
            if !v.iter().any(|e| e == platform)
            {
                return Err(Error::Generic(ErrorDomain::Builder, format!("Unsupported platform {}", platform)));
            }
            return Ok(());
        }
    }
}

fn check_arch(profile: &Profile, archs: &Option<Vec<String>>) -> Result<()>
{
    match archs
    {
        None => return Ok(()),
        Some(v) =>
        {
            let arch = &profile.architecture;
            if !v.iter().any(|e| e == arch)
            {
                return Err(Error::Generic(ErrorDomain::Builder, format!("Unsupported acrhitecture {}", arch)));
            }
            return Ok(());
        }
    }
}

fn check_compiler_version(version: &String, compiler: &Compiler) -> Result<()>
{
    if let Some(minver) = &compiler.minimum_version
    {
        let rep1 = minver.replace('.', "");
        let rep2 = version.replace('.', "");
        if let Ok(value1) = rep1.parse::<usize>()
        {
            if let Ok(value2) = rep2.parse::<usize>()
            {
                if value2 >= value1
                {
                    return Ok(());
                }
                else
                {
                    return Err(Error::Generic(ErrorDomain::Builder, format!("Unsuported compiler version {}", version)));
                }
            }
        }
    }
    if let Some(versions) = &compiler.versions
    {
        if !versions.iter().any(|e| e == version)
        {
            return Err(Error::Generic(ErrorDomain::Builder, format!("Unsuported compiler version {}", version)));
        }
    }
    return Ok(());
}

fn check_compiler(profile: &Profile, compilers: &Option<Vec<Compiler>>) -> Result<()>
{
    match compilers
    {
        None => return Ok(()),
        Some(v) =>
        {
            let compiler = &profile.compiler_name;
            let version = &profile.compiler_version;
            match v.iter().find(|v| &v.name == compiler)
            {
                None => return Err(Error::Generic(ErrorDomain::Builder, format!("Unsupported compiler"))),
                Some(cfg) => return check_compiler_version(&version, cfg)
            }
        }
    }
}

pub struct LuaBuilder {}

impl Builder for LuaBuilder
{
    fn can_build(&self, path: &Path) -> bool
    {
        let path: PathBuf = [path, Path::new("fpkg.lua")].iter().collect();
        if !path.exists()
        {
            return false;
        }
        let mut lua = LuaFile::new();
        if !lua.open_libs().is_ok() || !lua.open(&path).is_ok()
        {
            return true;
        }
        return lua.has_func_build();
    }

    fn run_build(&self, config: &str, path: &Path, toolchain: Option<&str>) -> Result<i32>
    {
        let profilemgr = ProfileManager::new(path)?;
        if !profilemgr.exists()
        {
            return Err(Error::Generic(ErrorDomain::Builder, String::from("Unable to load project profile; did you forget to run fpkg install?")))
        }
        let profile = profilemgr.get_current()?;
        let path: PathBuf = [path, Path::new("fpkg.lua")].iter().collect();
        let mut lua = LuaFile::new();
        lua.open_libs()?;
        lua.open(&path)?;
        let package = lua.read_table()?;
        let acfg = check_build_configuration(config, &package.configurations)?;

        println!("Building {} - {} ({}) with Lua Engine...", package.name, package.version, package.description);
        check_system(&profile, &package.systems)?;
        check_arch(&profile, &package.architectures)?;
        check_compiler(&profile, &package.compilers)?;
        let res = lua.func_build(&acfg, &profile)?;
        if res != 0
        {
            eprintln!("Build finished with non-zero exit code ({})", res);
        }
        return Ok(res);
    }
}

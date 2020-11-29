use std::path::Path;
use std::path::PathBuf;

use crate::builder::Builder;
use crate::builder::Error;
use crate::profile::Profile;
use crate::luaengine::Compiler;
use crate::luaengine::LuaFile;

fn check_build_configuration(config: &str, configs: &Option<Vec<String>>) -> Result<String, Error>
{
    match configs
    {
        None => return Ok(String::from(config)),
        Some(v) =>
        {
            let cfg = v.iter().find(|v| v == &config || v.to_lowercase() == config);
            match cfg
            {
                None => return Err(Error::Generic(format!("Could not find configuration named {}", config))),
                Some(v) => return Ok(String::from(v))
            }
        }
    }
}

fn check_system(profile: &Profile, systems: &Option<Vec<String>>) -> Result<(), Error>
{
    match systems
    {
        None => return Ok(()),
        Some(v) =>
        {
            let platform = profile.get("Platform").unwrap();
            if !v.iter().any(|e| e == platform)
            {
                return Err(Error::Generic(format!("Unsupported platform {}", platform)));
            }
            return Ok(());
        }
    }
}

fn check_arch(profile: &Profile, archs: &Option<Vec<String>>) -> Result<(), Error>
{
    match archs
    {
        None => return Ok(()),
        Some(v) =>
        {
            let arch = profile.get("Arch").unwrap();
            if !v.iter().any(|e| e == arch)
            {
                return Err(Error::Generic(format!("Unsupported acrhitecture {}", arch)));
            }
            return Ok(());
        }
    }
}

fn check_compiler_version(version: &String, compiler: &Compiler) -> Result<(), Error>
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
                    return Err(Error::Generic(format!("Unsuported compiler version {}", version)));
                }
            }
        }
    }
    if let Some(versions) = &compiler.versions
    {
        if !versions.iter().any(|e| e == version)
        {
            return Err(Error::Generic(format!("Unsuported compiler version {}", version)));
        }
    }
    return Ok(());
}

fn check_compiler(profile: &Profile, compilers: &Option<Vec<Compiler>>) -> Result<(), Error>
{
    match compilers
    {
        None => return Ok(()),
        Some(v) =>
        {
            let compiler = profile.get("CompilerName").unwrap();
            let version = profile.get("CompilerVersion").unwrap();
            match v.iter().find(|v| &v.name == compiler)
            {
                None => return Err(Error::Generic(format!("Unsupported compiler"))),
                Some(cfg) => return check_compiler_version(version, cfg)
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
        if !lua.open(&path).is_ok()
        {
            return true;
        }
        return lua.has_func_build();
    }

    fn run_build(&self, config: &str, path: &Path) -> Result<i32, Error>
    {
        let profile = Profile::new(path);
        if !profile.exists()
        {
            return Err(Error::Generic(String::from("Unable to load project profile; did you forget to run fpkg install?")))
        }
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

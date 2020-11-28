use std::path::Path;
use std::path::PathBuf;

use crate::luaengine::LuaFile;
use crate::builder::Error;
use crate::profile::Profile;
use crate::handle_build_command;
use crate::luaengine::PackageTable;


fn run_build(package: &PackageTable) -> i32
{
    if let Some(configs) = &package.configurations
    {
        for cfg in configs
        {
            let res = handle_build_command(cfg);
            if res != 0
            {
                return res;
            }
        }
    }
    return 0;
}

pub fn package(path: &Path) -> Result<i32, Error>
{
    let profile = Profile::new(path);
    if !profile.exists()
    {
        return Err(Error::Generic(String::from("Unable to load project profile; did you forget to run fpkg install?")))
    }
    let p: PathBuf = [path, Path::new("fpkg.lua")].iter().collect();
    let mut lua = LuaFile::new();
    lua.open_libs()?;
    lua.open(&p)?;
    let package = lua.read_table()?;
    let res = run_build(&package);
    if res != 0
    {
        return Ok(res);
    }

    return Ok(0);
}
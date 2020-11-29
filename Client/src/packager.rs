use std::path::Path;
use std::path::PathBuf;

use crate::luaengine::LuaFile;
use crate::builder::Error;
use crate::profile::Profile;

pub fn package(path: &Path) -> Result<i32, Error>
{
    let profile = Profile::new(path);
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
        return Ok(0)
    }
    eprintln!("WARNING: Nothing to package!");
    return Ok(2);
}
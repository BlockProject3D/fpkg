use std::path::Path;
use hlua::Lua;
use std::fs;

use crate::command::Command;
use crate::builder::Builder;
use crate::builder::Error;

pub struct LuaFile<'a>
{
    state: Lua<'a>
}

impl <'a> LuaFile<'a>
{
    pub fn new() -> LuaFile<'a>
    {
        let mut a = LuaFile
        {
            state: Lua::new()
        };

        a.state.openlibs();
        return a;
    }

    pub fn open(&mut self, path: &Path) -> Result<(), Error>
    {
        match fs::read_to_string(path)
        {
            Ok(s) =>
            {
                match self.state.execute::<()>(&s)
                {
                    Ok(()) => return Ok(()),
                    Err(e) => return Err(Error::Lua(e))
                }
            },
            Err(e) => return Err(Error::Io(e))
        }
    }
}

pub struct LuaBuilder {}

impl Builder for LuaBuilder
{
    fn can_build(&self) -> bool
    {
        return Path::new("./fpkg.lua").exists();
    }

    fn get_build_commands(&self, path: &Path) -> Result<Vec<Command>, Error>
    {
        let mut lua = LuaFile::new();
        let mut v: Vec<Command> = Vec::new();

        lua.open(path)?;
        return Ok(v);
    }
}

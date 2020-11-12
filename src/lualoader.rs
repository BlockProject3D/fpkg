use std::path::Path;
use hlua::Lua;

use crate::command::Command;
use crate::builder::Builder;

pub struct LuaFile
{
    state: Lua
}

impl LuaFile
{
    pub fn new() -> LuaFile
    {
        return LuaFile
        {
            state: Lua::new()
        };
    }
}

pub struct LuaBuilder {}

impl Builder for LuaBuilder
{
    fn can_build(&self) -> bool
    {
        return Path::new("./fpkg.lua").exists();
    }

    fn get_build_commands(&self, _: &Path) -> Vec<Command>
    {
        let mut v: Vec<Command> = Vec::new();


        return v;
    }
}

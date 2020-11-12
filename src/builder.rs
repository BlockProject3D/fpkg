use std::path::Path;
use std::vec::Vec;
use std::boxed::Box;

use crate::command::Command;
use crate::lualoader::LuaBuilder;

pub trait Builder
{
    fn can_build(&self) -> bool;
    fn get_build_commands(&self, path: &Path) -> Vec<Command>;
}

pub fn find_builder() -> Option<Box<dyn Builder>>
{
    let mut builders = vec!(
        Box::new(LuaBuilder {})
    );

    for i in 0..builders.len()
    {
        if builders[i].can_build() {
            return Some(builders.remove(i));
        }
    }
    return None;
}

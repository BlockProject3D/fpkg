use std::path::Path;
use std::vec::Vec;
use std::boxed::Box;

use crate::command::Command;

pub trait Builder
{
    fn can_build(&self) -> bool;
    fn get_build_commands(&self, path: &Path) -> Vec<Command>;
}

struct CMakeBuilder
{
}

impl Builder for CMakeBuilder
{
    fn can_build(&self) -> bool
    {
        return false;
    }

    fn get_build_commands(&self, _: &Path) -> Vec<Command>
    {
        return Vec::new();
    }
}

pub fn find_builder() -> Option<Box<dyn Builder>>
{
    let mut builders = vec!(
        Box::new(CMakeBuilder {})
    );

    for i in 0..builders.len()
    {
        if builders[i].can_build() {
            return Some(builders.remove(i));
        }
    }
    return None;
}

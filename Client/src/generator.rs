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
use std::boxed::Box;
use std::string::String;
use std::vec::Vec;
use std::collections::HashMap;

use crate::common::Result;

#[derive(Clone)]
pub struct Target
{
    pub relative_path: String,
    pub configuration: String
}

//Stores information about a single library with only paths relative to the base_folder
pub struct Library
{
    pub name: String,
    pub binaries: Vec<Target>,
    pub include_dirs: Vec<Target>
}

//The package_name is the name of the package and corresponds to the name of the folder
//which should contain the unpacked package (including package-info.json)
pub trait BuildGenerator
{
    fn add_library(&mut self, package_name: &str, lib: Library) -> Result<()>;
    fn add_framework(&mut self, package_name: &str) -> Result<()>;
    fn generate(&mut self) -> Result<()>;
}

pub trait BuildGeneratorProvider
{
    fn new(&self, base_folder: &Path, toolchain_name: &str) -> Result<Box<dyn BuildGenerator>>;
}

//The base_folder is the path to the root folder of FPKG (usually ./.fpkg)
//The toolchain_name is the name of the folder for the current toolchain
pub fn create_generator(generator_name: &str, base_folder: &Path, toolchain_name: &str) -> Result<Option<Box<dyn BuildGenerator>>>
{
    let mut v: HashMap<&str, Box<dyn BuildGeneratorProvider>> = HashMap::new();
    if let Some(g) = v.remove(generator_name)
    {
        let generator = g.new(base_folder, toolchain_name)?;
        return Ok(Some(generator));
    }
    return Ok(None);
}

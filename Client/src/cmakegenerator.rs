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
use std::string::String;
use std::vec::Vec;

use crate::common::Error;
use crate::common::ErrorDomain;
use crate::common::Result;
use crate::generator::Library;
use crate::generator::Target;
use crate::generator::BuildGenerator;
use crate::generator::BuildGeneratorProvider;

struct CMakeGenerator
{

}

fn compute_cmake_target(targets: Vec<Target>) -> String
{
    let mut s = String::new();

    return s;
}

impl BuildGenerator for CMakeGenerator
{
    fn add_library(&mut self, _: &str, lib: Library) -> Result<()>
    {
        let mut useless = lib;
        for i in 0..useless.binaries.len()
        {
            let mut vec: Vec<Target> = Vec::new();
            for j in 0..useless.binaries.len()
            {
                let v = &useless.binaries[i];
                let v1 = &useless.binaries[j];
                if v1.relative_path == v.relative_path
                    || (v.relative_path.ends_with(".lib") && v1.relative_path.ends_with(".dll"))
                    || (v.relative_path.ends_with(".dll") && v1.relative_path.ends_with(".lib"))
                {
                    vec.push(useless.binaries.remove(j))
                }
            }
            let s = compute_cmake_target(vec);
            println!("{}", s);
        }
        todo!()
    }

    fn add_framework(&mut self, _: &str) -> Result<()>
    {
        todo!()
    }

    fn generate(&mut self) -> Result<()>
    {
        todo!()
    }
}

pub struct CMakeGeneratorProvider {}

impl BuildGeneratorProvider for CMakeGeneratorProvider
{
    fn new(&self, base_folder: &Path, toolchain_name: &str) -> Result<Box<dyn BuildGenerator>>
    {
        todo!();
    }
}

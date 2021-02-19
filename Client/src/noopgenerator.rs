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

use std::boxed::Box;
use std::path::Path;

use crate::common::Result;
use crate::generator::Library;
use crate::generator::BuildGenerator;
use crate::generator::BuildGeneratorProvider;

struct NoOpGenerator {}

impl BuildGenerator for NoOpGenerator
{
    fn add_library(&mut self, _: &str, _: Library) -> Result<()>
    {
        return Ok(());
    }

    fn add_framework(&mut self, _: &str) -> Result<()>
    {
        return Ok(());
    }

    fn generate(&mut self) -> Result<()>
    {
        return Ok(());
    }
}

pub struct NoOpGeneratorProvider {}

impl BuildGeneratorProvider for NoOpGeneratorProvider
{
    fn new(&self, _: &Path, _: &str) -> Result<Box<dyn BuildGenerator>>
    {
        return Ok(Box::new(NoOpGenerator {}));
    }
}
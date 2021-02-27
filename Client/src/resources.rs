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
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use std::io::Result;

fn extract_resource(modules: &Path, name: &str, bytes: &[u8]) -> Result<()>
{
    let p = modules.join(name);
    if !p.exists()
    {
        let mut f = File::create(&p)?;
        let mut cursor: usize = 0;
        let mut remaining = bytes.len();
        while remaining > 0
        {
            let written = f.write(&bytes[cursor..std::cmp::min(8192, remaining)])?;
            cursor += written;
            remaining -= written;
        }
    }
    return Ok(());
}

pub fn check_resources() -> Result<()>
{
    if let Some(dir) = dirs::data_dir()
    {
        let modules: PathBuf = [&dir, Path::new("fpkg-lua-modules")].iter().collect();
        if !modules.exists()
        {
            std::fs::create_dir(&modules)?;
        }
        extract_resource(&modules, "cmake.lua", include_bytes!("../lua/cmake.lua"))?;
        extract_resource(&modules, "script.lua", include_bytes!("../lua/script.lua"))?;
    }
    return Ok(());
}

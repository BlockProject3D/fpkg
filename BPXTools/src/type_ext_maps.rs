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

fn bpxp_type_ext_map(block: &[u8; 16])
{
    match block[0]
    {
        0x0 => println!("Architecture: x86_64"),
        0x1 => println!("Architecture: aarch64"),
        0x2 => println!("Architecture: x86"),
        0x3 => println!("Architecture: armv7hl"),
        0x4 => println!("Architecture: Any"),
        _ => println!("Architecture: Unknown")
    }
    match block[1]
    {
        0x0 => println!("Platform: Linux"),
        0x1 => println!("Platform: Mac"),
        0x2 => println!("Platform: Windows"),
        0x3 => println!("Platform: Android"),
        0x4 => println!("Platform: Any"),
        _ => println!("Platform: Unknown")
    }
    println!("Generator: {}{}", block[2] as char, block[3] as char);
}

pub fn get_type_ext_map(btype: u8) -> Option<fn (block: &[u8; 16])>
{
    match btype
    {
        0x50 => Some(bpxp_type_ext_map),
        _ => None
    }
}
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

use bpx::sd::Value;
use bpx::sd::Object;
use bpx::sd::Array;
use std::string::String;
use std::io::Result;

fn gen_layer_prefix(layer: usize) -> String
{
    let mut res = String::new();

    for _ in 0..layer
    {
        res.push('\t');
    }
    return res;
}

fn print_value(layer: usize, value: &Value) -> Result<()>
{
    match value
    {
        Value::Null => println!("NULL"),
        Value::Uint8(v) => println!("(Uint8) {}", v),
        Value::Uint16(v) => println!("(Uint16) {}", v),
        Value::Uint32(v) => println!("(Uint32) {}", v),
        Value::Uint64(v) => println!("(Uint64) {}", v),
        Value::Int8(v) => println!("(Int8) {}", v),
        Value::Int16(v) => println!("(Int16) {}", v),
        Value::Int32(v) => println!("(Int32) {}", v),
        Value::Int64(v) => println!("(Int64) {}", v),
        Value::Float(v) => println!("(Float) {}", v),
        Value::Double(v) => println!("(Double) {}", v),
        Value::String(v) => println!("{}", v),
        Value::Bool(v) =>
        {
            if *v
            {
                println!("true");
            }
            else
            {
                println!("false");
            }
        },
        Value::Object(v) => print_object(layer + 1, v)?,
        Value::Array(v) => print_array(layer + 1, v)?
    }
    return Ok(());
}

fn print_array(layer: usize, array: &Array) -> Result<()>
{
    println!("[");
    for i in 0..array.len()
    {
        print_value(layer, &array[i])?;
    }
    println!("{}]", gen_layer_prefix(layer - 1));
    return Ok(());
}

pub fn print_object(layer: usize, object: &Object) -> Result<()>
{
    let prefix = gen_layer_prefix(layer);
    let debugger = bpx::sd::DebugSymbols::load(object)?;

    println!("{{");
    for key in object.get_keys()
    {
        print!("{} {}: ", prefix, debugger.lookup(*key));
        print_value(layer, &object[*key])?;
    }
    println!("{}}}", gen_layer_prefix(layer - 1));
    return Ok(());
}

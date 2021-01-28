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

// The BPX Structured Data format (BPXSD)

use std::collections::HashMap;
use std::vec::Vec;
use std::string::String;
use std::ops::Index;
use std::ops::IndexMut;
use std::io::Read;
use std::io::Write;
use std::io::Result;
use std::io::Error;
use std::io::ErrorKind;
use phf::phf_map;
use byteorder::ByteOrder;
use byteorder::LittleEndian;

#[derive(PartialEq, Clone)]
pub enum Value
{
    Null,
    Bool(bool),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Float(f32),
    Double(f64),
    String(String),
    Array(Array),
    Object(Object)
}

#[derive(PartialEq, Clone)]
pub struct Array
{
    data: Vec<Value>
}

impl Array
{
    pub fn new() -> Array
    {
        return Array
        {
            data: Vec::new()
        }
    }

    pub fn add(&mut self, v: Value)
    {
        self.data.push(v);
    }

    pub fn remove_at(&mut self, pos: usize)
    {
        self.data.remove(pos);
    }

    pub fn remove(&mut self, item: Value)
    {
        for i in 0..self.data.len()
        {
            if self.data[i] == item
            {
                self.data.remove(i);
            }
        }
    }

    pub fn get(&self, pos: usize) -> Option<&Value>
    {
        return self.data.get(pos);
    }

    pub fn len(&self) -> usize
    {
        return self.data.len();
    }
}

impl Index<usize> for Array
{
    type Output = Value;

    fn index<'a>(&'a self, i: usize) -> &'a Value
    {
        return &self.data[i];
    }
}

impl IndexMut<usize> for Array
{
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut Value
    {
        return &mut self.data[i];
    }
}

#[derive(PartialEq, Clone)]
pub struct Object
{
    props: HashMap<u64, Value>
}

impl Object
{
    pub fn new() -> Object
    {
        return Object
        {
            props: HashMap::new()
        }
    }

    pub fn raw_set(&mut self, hash: u64, value: Value)
    {
        self.props.insert(hash, value);
    }

    pub fn set(&mut self, name: &str, value: Value)
    {
        self.raw_set(super::utils::hash(name), value);
    }

    pub fn raw_get(&self, hash: u64) -> Option<&Value>
    {
        return self.props.get(&hash);
    }

    pub fn get(&self, name: &str) -> Option<&Value>
    {
        return self.raw_get(super::utils::hash(name));
    }

    pub fn prop_count(&self) -> usize
    {
        return self.props.len();
    }
}

impl Index<&str> for Object
{
    type Output = Value;

    fn index<'a>(&'a self, name: &str) -> &'a Value
    {
        return &self.props.index(&super::utils::hash(name));
    }
}

impl Index<u64> for Object
{
    type Output = Value;

    fn index<'a>(&'a self, hash: u64) -> &'a Value
    {
        return &self.props.index(&hash);
    }
}

fn read_bool(stream: &mut dyn Read) -> Result<Value>
{
    let mut flag: [u8; 1] = [0; 1];

    if stream.read(&mut flag)? != 1
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Bool(flag[0] == 1));
}

fn read_uint8(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 1] = [0; 1];

    if stream.read(&mut val)? != 1
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Uint8(val[0]));
}

fn read_int8(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 1] = [0; 1];

    if stream.read(&mut val)? != 1
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Int8(val[0] as i8));
}

fn read_uint16(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 2] = [0; 2];

    if stream.read(&mut val)? != 2
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Uint16(LittleEndian::read_u16(&val)));
}

fn read_int16(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 2] = [0; 2];

    if stream.read(&mut val)? != 2
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Int16(LittleEndian::read_i16(&val)));
}

fn read_uint32(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 4] = [0; 4];

    if stream.read(&mut val)? != 4
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Uint32(LittleEndian::read_u32(&val)));
}

fn read_int32(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 4] = [0; 4];

    if stream.read(&mut val)? != 4
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Int32(LittleEndian::read_i32(&val)));
}

fn read_uint64(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 8] = [0; 8];

    if stream.read(&mut val)? != 8
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Uint64(LittleEndian::read_u64(&val)));
}

fn read_int64(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 8] = [0; 8];

    if stream.read(&mut val)? != 8
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Int64(LittleEndian::read_i64(&val)));
}

fn read_float(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 4] = [0; 4];

    if stream.read(&mut val)? != 4
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Float(LittleEndian::read_f32(&val)));
}

fn read_double(stream: &mut dyn Read) -> Result<Value>
{
    let mut val: [u8; 8] = [0; 8];

    if stream.read(&mut val)? != 8
    {
        return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
    }
    return Ok(Value::Double(LittleEndian::read_f64(&val)));
}

fn read_string(stream: &mut dyn Read) -> Result<Value>
{
    let mut curs: Vec<u8> = Vec::new();
    let mut chr: [u8; 1] = [0; 1]; //read char by char with a buffer

    stream.read(&mut chr)?;
    while chr[0] != 0x0
    {
        curs.push(chr[0]);
        let res = stream.read(&mut chr)?;
        if res != 1
        {
            return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Value"));
        }
    }
    match String::from_utf8(curs)
    {
        Err(e) => return Err(Error::new(ErrorKind::InvalidData, format!("[BPX] error loading utf8 string: {}", e))),
        Ok(v) => return Ok(Value::String(v))
    }
}

fn parse_object(stream: &mut dyn Read) -> Result<Object>
{
    let mut obj = Object::new();
    let mut count =
    {
        let mut buf: [u8; 1] = [0; 1];
        if stream.read(&mut buf)? != 1
        {
            return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Object"));
        }
        buf[0]
    };

    while count > 0
    {
        let mut prop: [u8; 9] = [0; 9];
        if stream.read(&mut prop)? != 9
        {
            return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Object"));
        }
        let hash = LittleEndian::read_u64(&prop[0..8]);
        let type_code = prop[8];
        match VALUE_PARSERS.get(&type_code)
        {
            Some(func) => obj.raw_set(hash, func(stream)?),
            None => return Err(Error::new(ErrorKind::InvalidData, format!("[BPX] Got unexpected unknown type code ({}) from Structured Data Object", type_code)))
        }
        count -= 1;
    }
    return Ok(obj);
}

fn parse_array(stream: &mut dyn Read) -> Result<Array>
{
    let mut arr = Array::new();
    let mut count =
    {
        let mut buf: [u8; 1] = [0; 1];
        if stream.read(&mut buf)? != 1
        {
            return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Object"));
        }
        buf[0]
    };

    while count > 0
    {
        let mut type_code: [u8; 1] = [0; 1];
        if stream.read(&mut type_code)? != 1
        {
            return Err(Error::new(ErrorKind::UnexpectedEof, "[BPX] Unexpected end of input while reading Structured Data Object"));
        }
        match VALUE_PARSERS.get(&type_code[0])
        {
            Some(func) => arr.add(func(stream)?),
            None => return Err(Error::new(ErrorKind::InvalidData, format!("[BPX] Got unexpected unknown type code ({}) from Structured Data Object", type_code[0])))
        }
        count -= 1;
    }
    return Ok(arr);
}

static VALUE_PARSERS: phf::Map<u8, fn (stream: &mut dyn Read) -> Result<Value>> = phf_map! {
    0x0u8 => |_| { return Ok(Value::Null); }, //Mothershit PHF of my ass unable to read context!!!!
    0x1u8 => read_bool,
    0x2u8 => read_uint8,
    0x3u8 => read_uint16,
    0x4u8 => read_uint32,
    0x5u8 => read_uint64,
    0x6u8 => read_int8,
    0x7u8 => read_int16,
    0x8u8 => read_int32,
    0x9u8 => read_int64,
    0xAu8 => read_float,
    0xBu8 => read_double,
    0xCu8 => read_string,
    0xDu8 => |stream| { return Ok(Value::Array(parse_array(stream)?)); },
    0xEu8 => |stream| { return Ok(Value::Object(parse_object(stream)?)); }
};

pub fn load_structured_data(source: &mut dyn Read) -> Result<Object>
{
    return parse_object(source);
}
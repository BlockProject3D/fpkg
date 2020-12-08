// Copyright (c) 2020, BlockProject 3D
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

//BPX Type P implementation, if working will replace the current type P
//Base structure > String Section > Archive section

use std::fs::File;
use std::path::Path;
use std::vec::Vec;
use std::io;
use std::io::Read;
use std::collections::HashMap;
use byteorder::LittleEndian;
use byteorder::ByteOrder;
use xz::stream::Stream;

use crate::garraylen::*;

fn extract_slice<TArray: GenericArrayLen>(large_buf: &[u8], offset: usize) -> TArray::TArray
{
    let buf = &large_buf[offset..TArray::size];
    return TArray::from_array(buf);
}

#[derive(Copy, Clone)]
struct BPXPMainHeader
{
    signature: [u8;3], //+0
    btype: u8, //+3
    chksum: u32, //+4
    file_size: u64, //+8
    section_num: u32, //+16
    version: u32, //+20
    file_count: u32 //+24
}

const SIZE_MAIN_HEADER: usize = 40;

impl BPXPMainHeader
{
    fn read<TReader: io::Read>(reader: &mut TReader) -> io::Result<(u32, BPXPMainHeader)>
    {
        let mut buf: [u8;SIZE_MAIN_HEADER] = [0;SIZE_MAIN_HEADER];
        let mut checksum: u32 = 0;

        reader.read(&mut buf)?;
        for i in 0..SIZE_MAIN_HEADER
        {
            checksum += buf[i] as u32;
        }
        return Ok((checksum, BPXPMainHeader {
            signature: extract_slice::<T3>(&buf, 0),
            btype: buf[3],
            chksum: LittleEndian::read_u32(&extract_slice::<T4>(&buf, 4)),
            file_size: LittleEndian::read_u64(&extract_slice::<T8>(&buf, 8)),
            section_num: LittleEndian::read_u32(&extract_slice::<T4>(&buf, 16)),
            version: LittleEndian::read_u32(&extract_slice::<T4>(&buf, 20)),
            file_count: LittleEndian::read_u32(&extract_slice::<T4>(&buf, 24))
        }));
    }
}

#[derive(Copy, Clone)]
struct BPXSectionHeader
{
    pointer: u64, //+0
    csize: u32, //+8
    size: u32, //+12
    chksum: u32, //+16
    btype: u8, //+20
    flags: u8 //+21
}

const FLAG_COMPRESS_ZLIB: u8 = 0x1;
const FLAG_COMPRESS_XZ: u8 = 0x2;
const FLAG_CHECK_ADDLER32: u8 = 0x4;
const FLAG_CHECK_WEAK: u8 = 0x8;

const SIZE_SECTION_HEADER: usize = 24;

impl BPXSectionHeader
{
    fn read<TReader: io::Read>(reader: &mut TReader) -> io::Result<(u32, BPXSectionHeader)>
    {
        let mut buf: [u8;SIZE_SECTION_HEADER] = [0;SIZE_SECTION_HEADER];
        let mut checksum: u32 = 0;

        reader.read(&mut buf)?;
        for i in 0..SIZE_SECTION_HEADER
        {
            checksum += buf[i] as u32;
        }
        return Ok((checksum, BPXSectionHeader {
            pointer: LittleEndian::read_u64(&extract_slice::<T8>(&buf, 0)),
            csize: LittleEndian::read_u32(&extract_slice::<T4>(&buf, 8)),
            size: LittleEndian::read_u32(&extract_slice::<T4>(&buf, 12)),
            chksum: LittleEndian::read_u32(&extract_slice::<T4>(&buf, 16)),
            btype: buf[20],
            flags: buf[21]
        }));
    }
}

pub struct Decoder
{
    main_header: BPXPMainHeader,
    sections: Vec<BPXSectionHeader>,
    file: File
}

fn inflate_section(data: Vec<u8>, inflated_size: usize) -> io::Result<Vec<u8>>
{
    let mut unpacked: Vec<u8> = Vec::with_capacity(inflated_size);
    let mut decoder = match Stream::new_stream_decoder(inflated_size as u64, xz::stream::TELL_NO_CHECK)
    {
        Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] inflate error: {}", e))),
        Ok(v) => v
    };
    let mut status = xz::stream::Status::MemNeeded;

    while status != xz::stream::Status::StreamEnd
    {
        match decoder.process_vec(&data, &mut unpacked, xz::stream::Action::Finish)
        {
            Ok(s) => status = s,
            Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] inflate error: {}", e)))
        }
    }
    return Ok(unpacked);
}

fn run_checksum(data: &[u8], expected_checksum: u32) -> io::Result<()>
{
    let mut chk: u32 = 0;

    for i in 0..data.len()
    {
        chk += data[i] as u32;
    }
    if chk != expected_checksum
    {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] checksum validation failed"));
    }
    return Ok(());
}

impl Decoder
{
    fn read_section_header_table(&mut self, checksum: u32) -> io::Result<()>
    {
        let mut final_checksum = checksum;

        for _ in 0..self.main_header.section_num
        {
            let (checksum, header) = BPXSectionHeader::read(&mut self.file)?;
            if header.flags & FLAG_COMPRESS_ZLIB == FLAG_COMPRESS_ZLIB
            {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "[BPX] zlib compression is not supported by FPKG"));
            }
            if header.flags & FLAG_CHECK_ADDLER32 == FLAG_CHECK_ADDLER32
            {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "[BPX] addler32 checksum is not supported by FPKG"));
            }
            final_checksum += checksum;
            self.sections.push(header);
        }
        if final_checksum != self.main_header.chksum
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] checksum validation failed"));
        }
        return Ok(());
    }

    fn find_section_by_type(&self, btype: u8) -> Option<BPXSectionHeader>
    {
        for v in &self.sections
        {
            if v.btype == btype
            {
                return Some(*v);
            }
        }
        return None;
    }

    fn load_section(&mut self, section: &BPXSectionHeader) -> io::Result<Vec<u8>>
    {
        let mut data: Vec<u8> = Vec::with_capacity(section.csize as usize);

        self.file.read(&mut data)?;
        if section.flags & FLAG_COMPRESS_XZ == FLAG_COMPRESS_XZ
        {
            data = inflate_section(data, section.size as usize)?;
        }
        if section.flags & FLAG_CHECK_WEAK == FLAG_CHECK_WEAK
        {
            run_checksum(&data, section.chksum)?;
        }
        return Ok(data);
    }

    fn load_string_section(&mut self) -> io::Result<HashMap<u32, String>>
    {
        if let Some(section) = self.find_section_by_type(255)
        {
            let mut map: HashMap<u32, String> = HashMap::new();
            let data = self.load_section(&section)?;
            let mut ptr: u32 = 0;
            let mut curs: Vec<u8> = Vec::new();

            for i in 0..section.size
            {
                if data[i as usize] != 0x0
                {
                    curs.push(data[i as usize]);
                }
                else
                {
                    match String::from_utf8(curs)
                    {
                        Ok(v) =>
                        {
                            map.insert(ptr, v);
                        },
                        Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("[BPX] error loading utf8 string: {}", e)))
                    };
                    curs = Vec::new();
                    ptr = i + 1;
                }
            }
            return Ok(map);
        }
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "[BPX] could not locate string section"));
    }

    pub fn new(file: &Path) -> io::Result<Decoder>
    {
        let mut fle = File::open(file)?;
        let (checksum, header) = BPXPMainHeader::read(&mut fle)?;
        let num = header.section_num;
        let mut decoder = Decoder
        {
            file: fle,
            main_header: header,
            sections: Vec::with_capacity(num as usize)
        };

        decoder.read_section_header_table(checksum)?;
        return Ok(decoder);
    }

    pub fn extract_content(&mut self, target: &Path)
    {
        
    }
}

pub struct Encoder
{

}

impl Encoder
{

}

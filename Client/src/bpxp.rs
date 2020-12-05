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
use byteorder::LittleEndian;
use byteorder::ByteOrder;

use crate::garraylen::*;

fn extract_slice<TArray: GenericArrayLen>(large_buf: &[u8], offset: usize) -> TArray::TArray
{
    let buf = &large_buf[offset..TArray::size];
    return TArray::from_array(buf);
}

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
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "[BPX] Zlib compression is not supported by FPKG"));
            }
            if header.flags & FLAG_CHECK_ADDLER32 == FLAG_CHECK_ADDLER32
            {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "[BPX] Addler32 checksum is not supported by FPKG"));
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

    pub fn new(file: &Path) -> io::Result<Decoder>
    {
        let mut fle = File::open(file)?;
        let (checksum, header) = BPXPMainHeader::read(&mut fle)?;
        let num = header.section_num;
        let mut decoder = Decoder {
            file: fle,
            main_header: header,
            sections: Vec::with_capacity(num as usize)
        };

        decoder.read_section_header_table(checksum)?;
        return Ok(decoder);
    }

    
}

pub struct Encoder
{

}

impl Encoder
{

}

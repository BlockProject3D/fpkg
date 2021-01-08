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
use std::io::Write;
use std::io::Read;
use std::boxed::Box;
use byteorder::LittleEndian;
use byteorder::ByteOrder;
use super::garraylen::*;
use super::section::*;

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
        let head = BPXPMainHeader {
            signature: extract_slice::<T3>(&buf, 0),
            btype: buf[3],
            chksum: LittleEndian::read_u32(&extract_slice::<T4>(&buf, 4)),
            file_size: LittleEndian::read_u64(&extract_slice::<T8>(&buf, 8)),
            section_num: LittleEndian::read_u32(&extract_slice::<T4>(&buf, 16)),
            version: LittleEndian::read_u32(&extract_slice::<T4>(&buf, 20)),
            file_count: LittleEndian::read_u32(&extract_slice::<T4>(&buf, 24))
        };
        if head.signature[0] != 'B' as u8 || head.signature[1] != 'P' as u8 || head.signature[2] != 'X' as u8
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] File is not a BPX: incorrect signature"));
        }
        if head.btype != 'P' as u8
        {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("[BPX] Unknown type of BPX: {}", head.btype as char)));
        }
        if head.version != 0x1
        {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("[BPX] Unsupported version of BPX: {}", head.version)));
        }
        return Ok((checksum, head));
    }

    fn new() -> BPXPMainHeader
    {
        return BPXPMainHeader
        {
            signature: ['B' as u8, 'P' as u8, 'X' as u8], //+0
            btype: 'P' as u8, //+3
            chksum: 0, //+4
            file_size: SIZE_MAIN_HEADER as u64, //+8
            section_num: 0, //+16
            version: 0x1, //+20
            file_count: 0 //+24
        }
    }

    fn to_bytes(&self) -> [u8; SIZE_MAIN_HEADER]
    {
        let mut block: [u8; SIZE_MAIN_HEADER] = [0; SIZE_MAIN_HEADER];
        block[0] = self.signature[0];
        block[1] = self.signature[1];
        block[2] = self.signature[2];
        block[3] = self.btype;
        LittleEndian::write_u32(&mut block[4..8], self.chksum);
        LittleEndian::write_u64(&mut block[8..16], self.file_size);
        LittleEndian::write_u32(&mut block[16..20], self.section_num);
        LittleEndian::write_u32(&mut block[20..24], self.version);
        LittleEndian::write_u32(&mut block[24..28], self.file_count);
        return block;
    }

    fn get_checksum(&self) -> u32
    {
        let mut checksum: u32 = 0;
        let buf = self.to_bytes();
        for i in 0..SIZE_MAIN_HEADER
        {
            checksum += buf[i] as u32;
        }
        return checksum;
    }

    fn write<TWriter: io::Write>(&self, writer: &mut TWriter) -> io::Result<()>
    {
        let buf = self.to_bytes();
        writer.write(&buf)?;
        writer.flush()?;
        return Ok(());
    }
}


const FLAG_COMPRESS_ZLIB: u8 = 0x1;
const FLAG_CHECK_ADDLER32: u8 = 0x4;

const STRING_SECTION_TYPE: u8 = 0xFF;

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

    pub fn find_section_by_type(&self, btype: u8) -> Option<BPXSectionHeader>
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

    pub fn find_all_sections_of_type(&self, btype: u8) -> Vec<BPXSectionHeader>
    {
        let mut v = Vec::new();
        for s in &self.sections
        {
            if s.btype == btype
            {
                v.push(*s);
            }
        }        
        return v;
    }

    pub fn open_section(&mut self, section: &BPXSectionHeader) -> io::Result<Box<dyn Section>>
    {
        return open_section(&mut self.file, &section);
    }

    fn load_string_section(&mut self) -> io::Result<Box<dyn Section>>
    {
        if let Some(section) = self.find_section_by_type(255)
        {
            return self.open_section(&section);
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
    main_header: BPXPMainHeader,
    sections: Vec<BPXSectionHeader>,
    sections_data: Vec<Box<dyn Section>>,
    file: File
}

impl Encoder
{
    pub fn new(file: &Path) -> io::Result<Encoder>
    {
        let mut fle = File::create(file)?;
        return Ok(Encoder
        {
            main_header: BPXPMainHeader::new(),
            sections: Vec::new(),
            sections_data: Vec::new(),
            file: fle
        });
    }

    //Adds a new section; returns a reference to the new section for use in edit_section
    pub fn add_section(&mut self, btype: u8, size: u32 /* use 0 for automatic size */) -> io::Result<usize>
    {
        self.main_header.section_num += 1;
        let header = BPXSectionHeader::new(size, btype);
        let section = create_section(&header)?;
        self.sections.push(header);
        let r = self.sections.len() - 1;
        self.sections_data.push(section);
        return Ok(r);
    }

    pub fn edit_section<T>(&mut self, reference: usize, callback: fn (section: &mut Box<dyn Section>) -> io::Result<T>) -> io::Result<T>
    {
        let section = &mut self.sections_data[reference];
        let res = callback(section)?;
        if section.size() > u32::MAX as usize
        {
            panic!("BPX cannot support individual sections with size exceeding 4Gb (2 pow 32)");
        }
        return Ok(res);
    }

    fn write_compress_sections(&mut self) -> io::Result<(File, u32, usize)>
    {
        let mut all_sections_size: usize = 0;
        let mut chksum_sht: u32 = 0;
        let mut ptr: u64 = SIZE_MAIN_HEADER as u64 + (self.sections.len() as u64 * SIZE_SECTION_HEADER as u64);
        let mut f = tempfile::tempfile()?;
        for i in 0..self.sections.len()
        {
            let (csize, chksum, flags) = write_section(&mut self.sections_data[i], &mut f)?;
            self.sections[i].csize = csize as u32;
            self.sections[i].chksum = chksum;
            self.sections[i].flags = flags;
            self.sections[i].pointer = ptr;
            ptr += csize as u64;
            chksum_sht += self.sections[i].get_checksum();
            all_sections_size += csize;
        }
        return Ok((f, chksum_sht, all_sections_size));
    }

    fn write_data_file(&mut self, fle: &mut File, all_sections_size: usize) -> io::Result<()>
    {
        let mut idata: [u8; 8192] = [0; 8192];
        let mut count: usize = 0;
        while count < all_sections_size
        {
            let res = fle.read(&mut idata)?;
            self.file.write(&idata[0..res])?;
            count += res;
        }
        return Ok(());
    }

    pub fn save(&mut self) -> io::Result<()>
    {
        let (mut main_data, chksum_sht, all_sections_size) = self.write_compress_sections()?;
        self.main_header.file_size = all_sections_size as u64 + (self.sections.len() * SIZE_SECTION_HEADER) as u64 + SIZE_MAIN_HEADER as u64;
        self.main_header.chksum = chksum_sht + self.main_header.get_checksum();
        self.main_header.write(&mut self.file)?;
        for v in &self.sections
        {
            v.write(&mut self.file)?;
        }
        self.write_data_file(&mut main_data, all_sections_size)?;
        return Ok(());
    }
}

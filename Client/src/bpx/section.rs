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

use byteorder::LittleEndian;
use byteorder::ByteOrder;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::fs::File;
use std::boxed::Box;
use super::garraylen::*;
use xz::stream::Stream;

const SIZE_SECTION_HEADER: usize = 24;

#[derive(Copy, Clone)]
pub struct BPXSectionHeader
{
    pub pointer: u64, //+0
    pub csize: u32, //+8
    pub size: u32, //+12
    pub chksum: u32, //+16
    pub btype: u8, //+20
    pub flags: u8 //+21
}

impl BPXSectionHeader
{
    pub fn read<TReader: io::Read>(reader: &mut TReader) -> io::Result<(u32, BPXSectionHeader)>
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

    pub fn is_huge_section(&self) -> bool
    {
        return self.size > 1000000; //Return true if uncompressed size is greater than 100Mb
    }
}

pub trait Section : io::Read + io::Write + io::Seek
{
    fn load_in_memory(&mut self) -> io::Result<Vec<u8>>;
}

struct InMemorySection
{
    data: Vec<u8>,
    cursor: usize
}

impl InMemorySection
{
    pub fn new(data: Vec<u8>) -> InMemorySection
    {
        return InMemorySection
        {
            data: data,
            cursor: 0
        }
    }
}

impl io::Read for InMemorySection
{
    fn read(&mut self, data: &mut [u8]) -> io::Result<usize>
    {
        for i in 0..data.len()
        {
            if self.cursor + i >= self.data.len()
            {
                return Ok(i);
            }
            data[i] = self.data[self.cursor + i];
        }
        return Ok(data.len())
    }
}

impl io::Write for InMemorySection
{
    fn write(&mut self, data: &[u8]) -> io::Result<usize>
    {
        for i in 0..data.len()
        {
            if self.cursor + i >= self.data.len()
            {
                return Ok(i);
            }
            self.data[self.cursor + i] = data[i];
        }
        return Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()>
    {
        return Ok(());
    }
}

fn slow_but_correct_add(value: usize, offset: isize) -> usize
//Unfortunatly rust requires much slower add operation (another reason to not use rust for large scale projects)
{
    if offset < 0
    {
        return value - -offset as usize;
    }
    else if offset > 0
    {
        return value + offset as usize;
    }
    else
    {
        return value;
    }
}

impl io::Seek for InMemorySection
{
    fn seek(&mut self, state: io::SeekFrom) -> io::Result<u64>
    {
        match state
        {
            io::SeekFrom::Start(pos) => self.cursor += pos as usize,
            io::SeekFrom::End(pos) => self.cursor = slow_but_correct_add(self.data.len(), pos as isize),
            io::SeekFrom::Current(pos) => self.cursor = slow_but_correct_add(self.cursor, pos as isize)
        }
        return Ok(self.cursor as u64);
    }
}

impl Section for InMemorySection
{
    fn load_in_memory(&mut self) -> io::Result<Vec<u8>>
    {
        return Ok(self.data.clone());
    }
}

struct FileBasedSection
{
    data: File,
}

impl FileBasedSection
{
    pub fn new(data: File) -> FileBasedSection
    {
        return FileBasedSection
        {
            data: data
        };
    }
}

impl io::Read for FileBasedSection
{
    fn read(&mut self, data: &mut [u8]) -> io::Result<usize>
    {
        return self.data.read(data);
    }
}

impl io::Write for FileBasedSection
{
    fn write(&mut self, data: &[u8]) -> io::Result<usize>
    {
        return self.data.write(data);
    }

    fn flush(&mut self) -> io::Result<()>
    {
        return self.data.flush();
    }
}

impl io::Seek for FileBasedSection
{
    fn seek(&mut self, state: io::SeekFrom) -> io::Result<u64>
    {
        return self.data.seek(state);
    }
}

impl Section for FileBasedSection
{
    fn load_in_memory(&mut self) -> io::Result<Vec<u8>>
    {
        let mut data: Vec<u8> = Vec::new();
        self.data.read_to_end(&mut data)?;
        return Ok(data);
    }
}

fn read_chksum(data: &[u8]) -> u32
{
    let mut chk: u32 = 0;

    for i in 0..data.len()
    {
        chk += data[i] as u32;
    }
    return chk;
}

const FLAG_COMPRESS_XZ: u8 = 0x2;
const FLAG_CHECK_WEAK: u8 = 0x8;
const READ_BLOCK_SIZE: usize = 65536;

fn block_based_inflate(input: &mut dyn Read, output: &mut dyn Write, inflated_size: usize) -> io::Result<u32>
{
    let mut count: usize = 0;
    let mut decoder = match Stream::new_stream_decoder(inflated_size as u64, xz::stream::TELL_NO_CHECK)
    {
        Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] inflate error: {}", e))),
        Ok(v) => v
    };
    let mut action = xz::stream::Action::Run;
    let mut chksum: u32 = 0;

    while count < inflated_size {
        let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
        let mut status = xz::stream::Status::Ok;
        let res = input.read(&mut idata)?;
        if res < READ_BLOCK_SIZE
        {
            action = xz::stream::Action::Finish;
        }
        while status != xz::stream::Status::MemNeeded
        {
            let mut odata: Vec<u8> = Vec::new();
            match decoder.process_vec(&idata[0..res], &mut odata, action)
            {
                Ok(s) => status = s,
                Err(e) => return Err(io::Error::new(io::ErrorKind::InvalidData, format!("[BPX] inflate error: {}", e)))
            }
            chksum += read_chksum(&odata);
            output.write(&odata)?;
            count += odata.len();
        }
    }
    output.flush()?;
    return Ok(chksum);
}

fn load_section_in_memory(bpx: &mut File, header: &BPXSectionHeader) -> io::Result<InMemorySection>
{
    bpx.seek(io::SeekFrom::Start(header.pointer))?;
    if header.flags & FLAG_COMPRESS_XZ == FLAG_COMPRESS_XZ
    {
        let mut section = InMemorySection::new(vec![0; header.size as usize]);
        section.seek(io::SeekFrom::Start(0))?;
        let chksum = block_based_inflate(bpx, &mut section, header.size as usize)?;
        if header.flags & FLAG_CHECK_WEAK == FLAG_CHECK_WEAK && chksum != header.chksum
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] checksum validation failed"));
        }
        section.seek(io::SeekFrom::Start(0))?;
        return Ok(section);
    }
    else
    {
        let mut data = vec![0; header.size as usize];
        bpx.read(&mut data)?;
        let chksum = read_chksum(&data);
        if header.flags & FLAG_CHECK_WEAK == FLAG_CHECK_WEAK && chksum != header.chksum
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] checksum validation failed"));
        }
        let mut section = InMemorySection::new(data);
        section.seek(io::SeekFrom::Start(0))?;
        return Ok(section);
    }
}

fn load_section_as_file(bpx: &mut File, header: &BPXSectionHeader) -> io::Result<FileBasedSection>
{
    let mut section = FileBasedSection::new(tempfile::tempfile()?);

    bpx.seek(io::SeekFrom::Start(header.pointer))?;
    if header.flags & FLAG_COMPRESS_XZ == FLAG_COMPRESS_XZ
    {
        let chksum = block_based_inflate(bpx, &mut section, header.size as usize)?;
        if header.flags & FLAG_CHECK_WEAK == FLAG_CHECK_WEAK && chksum != header.chksum
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] checksum validation failed"));
        }
    }
    else
    {
        let mut idata: [u8; READ_BLOCK_SIZE] = [0; READ_BLOCK_SIZE];
        let mut count: usize = 0;
        let mut chksum: u32 = 0;
        while count < header.size as usize
        {
            let res = bpx.read(&mut idata)?;
            section.write(&idata[0..res])?;
            chksum += read_chksum(&idata[0..res]);
            count += res;
        }
        if header.flags & FLAG_CHECK_WEAK == FLAG_CHECK_WEAK && chksum != header.chksum
        {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "[BPX] checksum validation failed"));
        }
        section.flush()?;
    }
    section.seek(io::SeekFrom::Start(0))?;
    return Ok(section);
}

pub fn open_section(bpx: &mut File, header: &BPXSectionHeader) -> io::Result<Box<dyn Section>>
{
    if header.is_huge_section()
    {
        let data = load_section_as_file(bpx, &header)?;
        return Ok(Box::from(data));
    }
    else
    {
        let data = load_section_in_memory(bpx, &header)?;
        return Ok(Box::from(data));
    }
}
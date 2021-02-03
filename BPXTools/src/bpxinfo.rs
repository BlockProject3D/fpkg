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

use std::io::Result;
use bpx::bpx::Decoder;
use clap::ArgMatches;
use std::path::Path;
use std::string::String;
use super::type_ext_maps::get_type_ext_map;
use std::io::Error;
use std::io::ErrorKind;
use std::fs::File;
use std::io::Write;

fn print_main_header(bpx: &Decoder)
{
    println!("====> BPX Main Header <====");
    println!("Type: {}", bpx.main_header.btype as char);
    println!("Version: {}", bpx.main_header.version);
    println!("File size: {}", bpx.main_header.file_size);
    println!("Number of sections: {}", bpx.main_header.section_num);
    println!("====> End <====");
    println!("");
}

fn print_sht(bpx: &Decoder)
{
    println!("====> BPX Section Header Table <====");
    for i in 0..bpx.main_header.section_num
    {
        let section = bpx.get_section_by_index(i as usize);
        println!("Section #{}:", i);
        println!("\tType: {}", section.btype);
        println!("\tSize (after compression): {}", section.csize);
        println!("\tSize: {}", section.size);
        let mut flags = String::new();
        if section.flags & 0x1 == 0x1
        {
            flags.push_str(" | CompressZlib");
        }
        if section.flags & 0x2 == 0x2
        {
            flags.push_str(" | CompressXZ");
        }
        if section.flags & 0x4 == 0x4
        {
            flags.push_str(" | CheckCrc32");
        }
        if section.flags & 0x8 == 0x8
        {
            flags.push_str(" | CheckWeak");
        }
        if section.flags & 0x8 != 0x8 && section.flags & 0x4 != 0x4
        {
            flags.push_str(" | CheckNone");
        }
        println!("\tFlags: {}", &flags[2..]);
    }
    println!("====> End <====");
    println!("");
}

fn hex_print(block: &[u8], output: &mut dyn Write) -> Result<()>
{
    for i in 0..block.len()
    {
        if i != 0 && i % 16 == 0
        {
            writeln!(output, "")?;
        }
        write!(output, "{:02X} ", block[i])?;
    }
    return Ok(());
}

fn print_metadata(bpx: &Decoder, hex: bool) -> Result<()>
{
    println!("====> BPX TypeExt <====");
    if hex
    {
        hex_print(&bpx.main_header.type_ext, &mut std::io::stdout())?;
        println!("");
    }
    else
    {
        match get_type_ext_map(bpx.main_header.btype)
        {
            Some(func) => func(&bpx.main_header.type_ext),
            None =>
            {
                hex_print(&bpx.main_header.type_ext, &mut std::io::stdout())?;
                println!("");
            }
        }
    }
    println!("====> End <====");
    println!("");
    return Ok(());
}

//Crazy unlogical syntax ahead. Required cause somehow rust does not understand what mut means
fn print_section(bpx: &mut Decoder, section_id_str: &str, mut out_file: Option<&mut dyn Write>, hex: bool, force: bool) -> Result<()>
{
    let section_id: usize = match section_id_str.parse()
    {
        Ok(id) => id,
        Err(e) => return Err(Error::new(ErrorKind::InvalidInput, format!("Could not parse section index {} ({})", section_id_str, e)))
    };
    let section = match bpx.find_section_by_index(section_id)
    {
        Some(section) => section,
        None => return Err(Error::new(ErrorKind::InvalidInput, format!("Could not find section with index {}", section_id)))
    };
    let mut data = bpx.open_section(&section)?;

    if hex
    {
        let mut buf: [u8; 8192] = [0; 8192];
        let mut res = data.read(&mut buf)?;
        while res > 0
        {
            match out_file
            {
                Some(ref mut v) => hex_print(&buf[0..res], v)?,
                None => hex_print(&buf[0..res], &mut std::io::stdout())?
            }
            res = data.read(&mut buf)?;
        }
        println!("");
    }
    else
    {
        if !force && out_file.is_none()
        {
            return Err(Error::new(ErrorKind::Interrupted, "Outputing binary data to standard output can mess-up your terminal, please use --force if you're sure to continue"));
        }
        let mut buf: [u8; 8192] = [0; 8192];
        let mut res = data.read(&mut buf)?;
        while res > 0
        {
            match out_file
            {
                Some(ref mut v) =>
                { //Rust is an annoying language unable to understand that there's no return!
                    v.write(&buf[0..res])?;
                },
                None =>
                {
                    std::io::stdout().write(&buf[0..res])?;
                }
            }
            res = data.read(&mut buf)?;
        }
    }
    return Ok(());
}

fn print_structured_data(bpx: &mut Decoder, section_id_str: &str) -> Result<()>
{
    let section_id: usize = match section_id_str.parse()
    {
        Ok(id) => id,
        Err(e) => return Err(Error::new(ErrorKind::InvalidInput, format!("Could not parse section index {} ({})", section_id_str, e)))
    };
    let section = match bpx.find_section_by_index(section_id)
    {
        Some(section) => section,
        None => return Err(Error::new(ErrorKind::InvalidInput, format!("Could not find section with index {}", section_id)))
    };
    let mut data = bpx.open_section(&section)?;
    let object = bpx::sd::load_structured_data(&mut data)?;

    return super::printsd::print_object(1, &object);
}

pub fn run(file: &Path, matches: &ArgMatches) -> Result<()>
{
    let mut bpx = bpx::bpx::Decoder::new(Path::new(file))?;

    print_main_header(&bpx);
    if matches.is_present("metadata")
    {
        print_metadata(&bpx, matches.is_present("hex"))?;
    }
    if matches.is_present("sht")
    {
        print_sht(&bpx);
    }
    if let Some(sidstr) = matches.value_of("section_id")
    {
        if matches.is_present("bpxsd")
        {
            print_structured_data(&mut bpx, sidstr)?;
        }
        else
        {
            match matches.value_of("out_file")
            {
                None => print_section(&mut bpx, sidstr, None, matches.is_present("hex"), matches.is_present("force"))?,
                Some(s) =>
                {
                    let mut fle = File::create(s)?;
                    print_section(&mut bpx, sidstr, Some(&mut fle), matches.is_present("hex"), matches.is_present("force"))?;
                }
            }    
        }
    }
    return Ok(());
}
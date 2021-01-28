use std::io::Result;
use bpx::bpx::Decoder;
use clap::ArgMatches;
use std::path::Path;
use std::string::String;

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

pub fn run(file: &Path, matches: &ArgMatches) -> Result<()>
{
    let bpx = bpx::bpx::Decoder::new(Path::new(file))?;

    print_main_header(&bpx);
    if matches.is_present("sht")
    {
        print_sht(&bpx);
    }
    return Ok(());
}
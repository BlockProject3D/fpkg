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

use clap::clap_app;
use clap::AppSettings;
use std::path::Path;

mod bpxinfo;
mod pack;
mod unpack;
mod type_ext_maps;
mod printsd;

fn error(err: &std::io::Error)
{
    eprintln!("{}", err);
    std::process::exit(1);
}

fn main() {
    let matches = clap_app!(bpxd =>
        (version: "1.0")
        (author: "BlockProject3D <https://github.com/BlockProject3D>")
        (about: "BPX Debugging tools")
        (@arg file: -f --file +required +takes_value "Path to the BPX file to debug/create")
        (@subcommand info =>
            (about: "Prints general information about a given BPX file ")
            (@arg sht: -s --sht "Prints the section header table (SHT)")
            (@arg metadata: -m --metadata "Prints metadata of this BPX (metadata here refers to the TypeExt block")
            (@arg hex: -x --hex "Prints data in hex")
            (@arg force: -f --force "Force prints data to terminal ignoring potential terminal destruction")
            (@arg section_id: -d --dump +takes_value "Dumps the content of the section identified by the given index")
            (@arg out_file: -o --output +takes_value "Save dump output to a file")
            (@arg bpxsd: --bpxsd "Parse the section to print (specified in -d) as a BPX Structured Data Object (BPXSD)")
        )
        (@subcommand pack =>
            (about: "Create a BPX type P (Package) with given data inside")
            (@arg files: +required ... "List of files to pack")
        )
        (@subcommand unpack =>
            (about: "Unpacks a given BPX type P (Package) file")
        )
    ).setting(AppSettings::SubcommandRequiredElseHelp).get_matches();
    let file = matches.value_of("file").unwrap();

    if let Some(matches) = matches.subcommand_matches("info")
    {
        match bpxinfo::run(Path::new(file), matches)
        {
            Ok(()) => std::process::exit(0),
            Err(e) => error(&e)
        }
    }
    if let Some(matches) = matches.subcommand_matches("pack")
    {
        match pack::run(Path::new(file), matches)
        {
            Ok(()) => std::process::exit(0),
            Err(e) => error(&e)
        }
    }
    if matches.subcommand_matches("unpack").is_some()
    {
        match unpack::run(Path::new(file))
        {
            Ok(()) => std::process::exit(0),
            Err(e) => error(&e)
        }
    }
}

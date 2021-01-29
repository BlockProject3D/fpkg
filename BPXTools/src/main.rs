use clap::clap_app;
use clap::AppSettings;
use std::path::Path;

mod bpxinfo;
mod pack;
mod unpack;
mod type_ext_maps;

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
            (@arg sht: --sht "Prints the section header table (SHT)")
            (@arg metadata: -m --metadata "Prints metadata of this BPX (metadata here refers to the TypeExt block")
            (@arg hex: -x --hex "Prints data in hex")
            (@arg force: -f --force "Force prints data to terminal ignoring potential terminal destruction")
            (@arg section_id: -d --dump +takes_value "Dumps the content of the section identified by the given index")
            (@arg out_file: -o --output +takes_value "Save dump output to a file")
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

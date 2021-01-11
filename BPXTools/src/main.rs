use clap::clap_app;
use clap::AppSettings;

fn main() {
    let matches = clap_app!(bpxd =>
        (version: "1.0")
        (author: "BlockProject3D <https://github.com/BlockProject3D>")
        (about: "BPX Debugging tools")
        (@arg file: -f --file +required +takes_value "Path to the BPX file to debug")
        (@subcommand info =>
            (about: "Prints general information about a given BPX file ")
            (@arg sht: --sht "Prints the section header table (SHT)")
            (@arg section_id: -d +takes_value "Dumps the content in hexadecimal of the section identified by the given index")
        )
        (@subcommand pack =>
            (about: "Create a BPX type P (Package) with given data inside")
            (@arg files: +required ... "List of files to pack")
        )
        (@subcommand unpack =>
            (about: "Unpacks a given BPX type P (Package) file")
        )
    ).setting(AppSettings::SubcommandRequiredElseHelp).get_matches();

    println!("BPX Debug tools");
}

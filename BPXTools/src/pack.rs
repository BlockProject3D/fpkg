use std::path::Path;
use clap::ArgMatches;
use std::io::Result;
use bpx::bpxp;

pub fn run(file: &Path, matches: &ArgMatches) -> Result<()>
{
    let mut encoder = bpxp::Encoder::new(file)?;
    let files: Vec<&str> = matches.values_of("files").unwrap().collect();

    for v in files
    {
        encoder.pack(Path::new(v))?;
    }
    encoder.save()?;
    return Ok(());
}
use bpx::bpxp;
use std::path::Path;
use std::io::Result;

pub fn run(file: &Path) -> Result<()>
{
    let mut decoder = bpxp::Decoder::new(file)?;
    decoder.unpack(Path::new("."))?;
    return Ok(());
}
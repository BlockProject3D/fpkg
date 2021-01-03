use std::io::SeekFrom;
use std::io::Read;
use std::io::Result;
use std::string::String;
use std::io::BufReader;
use std::io::Error;
use std::io::ErrorKind;
use super::section::Section;

pub fn get_string(ptr: u32, string_section: &mut dyn Section) -> Result<String>
{
    let mut curs: Vec<u8> = Vec::new();
    let mut reader = BufReader::new(string_section);
    let mut chr: [u8; 1] = [0; 1]; //read char by char with a buffer

    string_section.seek(SeekFrom::Start(ptr as u64))?;
    reader.read(&mut chr);
    while chr[0] != 0x0
    {
        curs.push(chr[0]);
    }
    match String::from_utf8(curs)
    {
        Err(e) => return Err(Error::new(ErrorKind::InvalidInput, format!("[BPX] error loading utf8 string: {}", e))),
        Ok(v) => return Ok(v)
    }
}

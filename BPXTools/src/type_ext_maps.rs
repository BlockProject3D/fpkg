fn bpxp_type_ext_map(block: &[u8; 16])
{
    match block[0]
    {
        0x0 => println!("Architecture: x86_64"),
        0x1 => println!("Architecture: aarch64"),
        0x2 => println!("Architecture: x86"),
        0x3 => println!("Architecture: armv7hl"),
        0x4 => println!("Architecture: Any"),
        _ => println!("Architecture: Unknown")
    }
    match block[1]
    {
        0x0 => println!("Platform: Linux"),
        0x1 => println!("Platform: Mac"),
        0x2 => println!("Platform: Windows"),
        0x3 => println!("Platform: Android"),
        0x4 => println!("Platform: Any"),
        _ => println!("Platform: Unknown")
    }
    println!("Generator: {}{}", block[2] as char, block[3] as char);
}

pub fn get_type_ext_map(btype: u8) -> Option<fn (block: &[u8; 16])>
{
    match btype
    {
        0x50 => Some(bpxp_type_ext_map),
        _ => None
    }
}
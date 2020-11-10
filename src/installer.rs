use std::string::String;
use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use std::io;
use std::io::BufRead;
use regex::Regex;

use crate::profile::Profile;
use crate::builder;

fn install_sub_directory(path: &Path, platform: Option<&str>) -> Result<(), String>
{
    let mut profile = Profile::new(path);

    match Profile::mkdir(path)
    {
        Err(e) => return Err(format!("Error creating .fpkg directory {}", e)),
        _ => ()
    }
    if !profile.exists()
    {
        match platform
        {
            Some(name) =>
            {
                if !profile.regenerate_cross(&name)
                {
                    return Err(format!("No such profile {}", name));
                }
            },
            None => profile.regenerate_self()
        }
    }
    match profile.write()
    {
        Err(e) => return Err(format!("Error writing profile {}", e)),
        _ => ()
    }
    //TODO: Implement dependency/sdk downloader/installer and connect it right here
    return Ok(());
}

fn parse_cmake_lists() -> io::Result<Vec<PathBuf>>
{
    let mut dirs: Vec<PathBuf> = Vec::new();
    let file = File::open("./CMakeLists.txt")?;
    let reader = BufReader::new(file);

    for line1 in reader.lines()
    {
        let line = line1?;
        let re = Regex::new(r"add_subdirectory\(([^)]+)\)").unwrap();
        if re.is_match(&line)
        {
            for group in re.captures_iter(&line)
            {
                dirs.push(Path::new(".").join(&group[1]));
            }
        }
    }
    return Ok(dirs);
}

fn check_is_valid_project_dir() -> Result<(), String>
{
    let builder = builder::find_builder();
    if builder.is_none() {
        return Err(String::from("Project directory does not contain a valid project file"));
    }
    return Ok(());
}

pub fn install(platform: Option<&str>) -> Result<(), String>
{
    check_is_valid_project_dir()?;
    install_sub_directory(Path::new("."), platform)?;
    if Path::new("./CMakeLists.txt").exists()
    {
        match parse_cmake_lists()
        {
            Ok(v) =>
            {
                for path in v
                {
                    install_sub_directory(&path, platform)?;
                }
            },
            Err(e) => return Err(format!("Error reading CMakeLists {}", e))
        }
    }
    return Ok(());
}

// Copyright (c) 2020, BlockProject 3D
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
            Some(name) => profile.regenerate_cross(name)?,
            None => profile.regenerate_self()?
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
    let builder = builder::find_builder(Path::new("."));
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

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

//build current project: fpkg build debug/release/whatever config > To work check for CMakeLists.txt or Makefile or fpkg.lua
//  This command is mainly intended to simplify build process on CI servers
//package current project: fpkg package > Check for fpkg.lua
//  This command can be used to build a custom package (defined in fpkg.lua) or to build a fpkg package file itself for the current platform
//install libraries/sdks for current project: fpkg install > Check for fpkg.lua
//  This command accepts an optional platform name and by default will consider the build to be for the current platform. (ex: fpkg install android)
// For install process:
//      Create hidden .fpkg folder at the root of the project, the folder should contain all libraries needed to build the project
//      The folder should contain a build file for the current build system to be included in the build system
//      The folder should contain a build file for the current build system to be included in the build system for each sdk named by the name of the sdk
// SDKs should be downloaded and installed in a cache directory as usually SDKs contains additional build tools
// For fpkg.lua find process (when running fpkg install):
//      Parse CMakeLists.txt in the current project folder, if it exists locate project calls inside and for each of those calls:
//          Search the directory in the project call for fpkg.lua, if found then run install process for the sub-project
//      Leave a chance to define subprojects inside of fpkg.lua in case there is no CMakeLists.txt in the current project folder

//The idea:
//  - No packages is built on the current user's local machine to avoid incompatible build environments
//  - All packages must be pre-compiled for all configurations (debug, release, etc) on a given supported platform
//  - At least all publically available packages MUST be building and running under OSX Sierra +, Ubuntu 18 + and Windows 7+
//  - Optionally a package can provide binarries for any other platform even Android
//  - Any package that does not respect any of the previous requirements may still be published but not to the main registry
//  - A package will be distributed as a compressed archive file containing builds for all configurations of a given platform.
mod command;
mod profile;
mod common;
mod settings;

//Commands implementations
mod installer;
mod packager;
mod publisher;
mod scripts;

//Lua Engine
mod luaengine;
mod lualibfile;
mod lualibcommand;
mod lualibstring;
mod lualibfpkg;
mod lualiblog;

//Registry implementations
mod registry;
mod gitlabregistry;

//Builder implementations
mod builder;
mod luabuilder;
mod cmakebuilder;

//Generator implementations
mod generator;
mod cmakegenerator;
mod noopgenerator;

//Toolchain implementations
mod toolchain;
mod hosttoolchain;

use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::io::Write;
use clap::clap_app;

fn handle_install_command(toolchain: Option<&str>) -> i32
{
    match installer::install(toolchain)
    {
        Err(e) =>
        {
            match e
            {
                common::Error::Io(_, v) => eprintln!("An io error has occured: {}", v),
                common::Error::Lua(_, v) => eprintln!("A lua error has occured: {}", v),
                common::Error::Generic(_, v) => eprintln!("An error has occured: {}", v)
            }
            return 1;
        }
        Ok(()) => return 0
    }
}

fn handle_build_command(config: &str, toolchain: Option<&str>) -> i32
{
    let path = Path::new(".");
    let builder = builder::find_builder(path);

    match builder
    {
        None =>
        {
            eprintln!("No valid builder found for current project");
            return 1;
        }
        Some(b) =>
        {
            match b.run_build(config, path, toolchain)
            {
                Ok(res) => return res,
                Err(e) =>
                {
                    match e
                    {
                        common::Error::Io(_, v) => eprintln!("An io error has occured: {}", v),
                        common::Error::Lua(_, v) => eprintln!("A lua error has occured: {}", v),
                        common::Error::Generic(_, v) => eprintln!("An error has occured: {}", v)
                    }
                    return 1;
                }
            }
        }
    }
}

fn handle_package_command(toolchain: Option<&str>) -> i32
{
    match packager::package(Path::new("./"), toolchain)
    {
        Ok(v) => return v,
        Err(e) =>
        {
            match e
            {
                common::Error::Io(_, v) => eprintln!("An io error has occured: {}", v),
                common::Error::Lua(_, v) => eprintln!("A lua error has occured: {}", v),
                common::Error::Generic(_, v) => eprintln!("An error has occured: {}", v)
            }
            return 1;
        }
    }
}

fn handle_publish_command(registry: Option<&str>) -> i32
{
    match publisher::publish(Path::new("./"), registry)
    {
        Ok(v) => return v,
        Err(e) =>
        {
            match e
            {
                common::Error::Io(_, v) => eprintln!("An io error has occured: {}", v),
                common::Error::Lua(_, v) => eprintln!("A lua error has occured: {}", v),
                common::Error::Generic(_, v) => eprintln!("An error has occured: {}", v)
            }
            return 1;
        }
    }
}

fn handle_run_command(toolchain: Option<&str>, args: Vec<&str>) -> i32
{
    match scripts::run_script(toolchain, Path::new("./"), args)
    {
        Ok(v) => return v,
        Err(e) =>
        {
            match e
            {
                common::Error::Io(_, v) => eprintln!("An io error has occured: {}", v),
                common::Error::Lua(_, v) => eprintln!("A lua error has occured: {}", v),
                common::Error::Generic(_, v) => eprintln!("An error has occured: {}", v)
            }
            return 1;
        }
    }
}

fn extract_resource(modules: &Path, name: &str, bytes: &[u8]) -> std::io::Result<()>
{
    let p = modules.join(name);
    if !p.exists()
    {
        let mut f = File::create(&p)?;
        let mut cursor: usize = 0;
        let mut remaining = bytes.len();
        while remaining > 0
        {
            let written = f.write(&bytes[cursor..std::cmp::min(8192, remaining)])?;
            cursor += written;
            remaining -= written;
        }
    }
    return Ok(());
}

fn initial_setup() -> std::io::Result<()>
{
    if let Some(dir) = dirs::data_dir()
    {
        let modules: PathBuf = [&dir, Path::new("fpkg-lua-modules")].iter().collect();
        if !modules.exists()
        {
            std::fs::create_dir(&modules)?;
        }
        extract_resource(&modules, "cmake.lua", include_bytes!("../lua/cmake.lua"))?;
    }
    return Ok(());
}

fn main() {
    initial_setup().unwrap();
    let matches = clap_app!(fpkg =>
        (version: "1.0")
        (author: "BlockProject3D <https://github.com/BlockProject3D>")
        (about: "The easy C++ package manager built for BlockProject 3D")
        (@subcommand build =>
            (about: "Run automated build using CMake or Lua")
            (@arg toolchain: +takes_value -t --toolchain "Specifies the toolchain to install packages for. Defaults to the host toolchain.")
            (@arg configuration: "Specifies an optional configuration to build with.")
        )
        (@subcommand test =>
            (about: "Build and run unit tests")
        )
        (@subcommand package =>
            (about: "Run automated packaging using Lua")
            (@arg publish: -p --publish "Publish the package.")
            (@arg registry: +takes_value --registry -r "Specify the name of the registry to publish to.")
            (@arg toolchain: +takes_value -t --toolchain "Specifies the toolchain to install packages for. Defaults to the host toolchain.")
        )
        (@subcommand install =>
            (about: "Install all required dependencies and SDKs")
            (@arg toolchain: +takes_value -t --toolchain "Specifies the toolchain to install packages for. Defaults to the host toolchain.")
        )
        (@subcommand run =>
            (about: "Run custom lua scripts")
            (@arg toolchain: +takes_value -t --toolchain "Specifies the toolchain to install packages for. Defaults to the host toolchain.")
            (@arg args: min_values(1) "The arguments to pass to the custom script args[0] usually refers to the script name.")
        )
    ).get_matches();

    if let Some(matches) = matches.subcommand_matches("install")
    {
        std::process::exit(handle_install_command(matches.value_of("toolchain")));
    }
    if let Some(matches) = matches.subcommand_matches("build")
    {
        match matches.value_of("configuration")
        {
            Some(v) => std::process::exit(handle_build_command(v, matches.value_of("toolchain"))),
            None => std::process::exit(handle_build_command("debug", matches.value_of("toolchain")))
        }
    }
    if let Some(matches) = matches.subcommand_matches("package")
    {
        let res = handle_package_command(matches.value_of("toolchain"));
        if res != 0
        {
            std::process::exit(res);
        }
        if matches.is_present("publish")
        {
            std::process::exit(handle_publish_command(matches.value_of("registry")));
        }
    }
    if let Some(matches) = matches.subcommand_matches("run")
    {
        if let Some(useless) = matches.values_of("args")
        {
            let vec = useless.collect();
            std::process::exit(handle_run_command(matches.value_of("toolchain"), vec));
        }
        else
        {
            std::process::exit(handle_run_command(matches.value_of("toolchain"), Vec::new()));
        }
    }
}

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

mod builder;
mod command;
mod profile;
mod installer;

use clap::clap_app;

fn handle_install_command(platform: Option<&str>) -> i32
{
    match installer::install(platform)
    {
        Err(e) =>
        {
            eprintln!("{}", e);
            return 1;
        }
        Ok(()) => return 0
    }
}

fn main() {
    let matches = clap_app!(fpkg =>
        (version: "1.0")
        (author: "BlockProject3D <https://github.com/BlockProject3D>")
        (about: "The easy C++ package manager built for BlockProject 3D")
        (@subcommand build =>
            (about: "Run automated build using CMake, Make or Lua")
            (@arg configuration: "Specifies an optional configuration to build with.")
        )
        (@subcommand test =>
            (about: "Build and run unit tests")
        )
        (@subcommand package =>
            (about: "Run automated packaging using Lua")
        )
        (@subcommand install =>
            (about: "Install all required dependencies and SDKs")
            (@arg platform: "Specifies the platform to install packages for. By default relies on automatic platform detection. Only useful when building for cross-compile targets such as Android.")
        )
    ).get_matches();

    if let Some(platform) = matches.subcommand_matches("install")
    {
        std::process::exit(handle_install_command(platform.value_of("platform")));
    }
    /*if let Some(config) = matches.subcommand_matches("build") {
        let builder = find_builder();
        if let Some(buildcfg) = config.value_of("configuration") {
            println!("Build with config: {}", buildcfg);
        }
        println!("Build with config: debug");
    }
    let mut test = Lua::new();
    match test.execute::<()>("print(\"This is a test\")")
    {
        Ok(()) => return,
        Err(e) => println!("Error: {:#?}", e)
    }*/
    //let c: f32 = test.get("a").unwrap();
    // Same as before...
    //println!("test {}", c);
}

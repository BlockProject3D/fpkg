// Copyright (c) 2021, BlockProject 3D
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

use std::path::Path;
use std::path::PathBuf;
use std::string::String;
use std::vec::Vec;
use std::collections::HashSet;
use std::fs;

use crate::common::Error;
use crate::common::ErrorDomain;
use crate::common::Result;
use crate::generator::Library;
use crate::generator::Target;
use crate::generator::BuildGenerator;
use crate::generator::BuildGeneratorProvider;

struct CMakeGenerator
{
    toolchain_name: String,
    base_folder: PathBuf,
    lib_names: Vec<String>,
    framework_names: Vec<String>
}

#[derive(Clone)]
struct CMakeTarget
{
    configuration: String,
    implib: Option<String>,
    location: String
}

fn compute_cmake_target(macro_name: &str, package_name: &str, targets: Vec<CMakeTarget>) -> (String, String)
{
    let name;
    if let Some(idx) = targets[0].location.rfind('/')
    {
        let sub = &targets[0].location[idx + 1..].replace('.', "");
        name = format!("{}::{}", package_name, &sub);
    }
    else
    {
        name = format!("{}::{}", package_name, &targets[0].location.replace('.', ""));
    }
    
    let mut s = format!("add_library({} UNKNOWN IMPORTED)", name);
    s.push('\n');
    for target in targets
    {
        s.push_str(&format!("set_property(TARGET {} PROPERTY IMPORTED_LOCATION_{} \"${{{}}}/{}\")", name, target.configuration.to_uppercase(), macro_name, target.location));
        if let Some(implib) = target.implib
        {
            s.push_str(&format!("set_property(TARGET {} PROPERTY IMPORTED_IMPLIB_{} \"${{{}}}/{}\")", name, target.configuration.to_uppercase(), macro_name, implib));
        }
        s.push('\n');
    }
    return (name, s);
}

//This is a hack intended to FORCE cmake to handle multiple binaries per IMPORTED target
fn group_binaries_by_implib(lib: &Library) -> Vec<CMakeTarget>
{
    let mut set: HashSet<usize> = HashSet::new();
    let mut targets: Vec<CMakeTarget> = Vec::new();

    for i in 0..lib.binaries.len()
    {
        if set.contains(&i)
        {
            continue;
        }
        let v = &lib.binaries[i];
        for j in 0..lib.binaries.len()
        {
            if set.contains(&j) || i == j
            {
                continue;
            }
            let v1 = &lib.binaries[j];
            let file_part = &v.relative_path[0..v.relative_path.len() - 3];
            let file_part1 = &v1.relative_path[0..v1.relative_path.len() - 3];
            if file_part == file_part1 && (
                    (v.relative_path.ends_with(".lib") && v1.relative_path.ends_with(".dll")) ||
                    (v.relative_path.ends_with(".dll") && v1.relative_path.ends_with(".lib"))
                )
            {
                //Should detect matching LIB+DLL
                if v.relative_path.ends_with(".lib") && v1.relative_path.ends_with(".dll")
                {
                    // i = IMPLIB, j = LOCATION
                    targets.push(CMakeTarget
                    {
                        location: v1.relative_path.clone(),
                        implib: Some(v.relative_path.clone()),
                        configuration: v.configuration.clone()
                    });
                }
                else
                {
                    // i = LOCATION, j = IMPLIB
                    targets.push(CMakeTarget
                    {
                        location: v.relative_path.clone(),
                        implib: Some(v1.relative_path.clone()),
                        configuration: v.configuration.clone()
                    });
                }
                set.insert(j);
            }
        }
        targets.push(CMakeTarget
        {
            location: v.relative_path.clone(),
            configuration: v.configuration.clone(),
            implib: None
        });
        set.insert(i);
    }
    return targets;
}

fn group_binaries_by_config(pass1: Vec<CMakeTarget>, macro_name: &str, package_name: &str) -> Vec<(String, String)>
{
    let mut vec = Vec::new();
    let mut set: HashSet<usize> = HashSet::new();

    for i in 0..pass1.len()
    {
        if set.contains(&i)
        {
            continue;
        }
        let mut targets: Vec<CMakeTarget> = Vec::new();
        let v = &pass1[i];
        for j in 0..pass1.len()
        {
            if set.contains(&j) || i == j
            {
                continue;
            }
            let v1 = &pass1[j];
            if let (Some(id), Some(id1)) = (v.location.find('/'), v1.location.find('/'))
            {
                let vpath = &v.location[id + 1..];
                let vpath1 = &v1.location[id1 + 1..];
                if vpath == vpath1
                {
                    targets.push(v.clone());
                    targets.push(v1.clone());
                }
            }
            set.insert(j);
        }
        let (name, res) = compute_cmake_target(&macro_name, &package_name, targets);
        vec.push((name, res));
        set.insert(i);
    }
    return vec;
}

fn compute_master_target(macro_name: &str, package_name: &str, include_dirs: &Vec<Target>, cmake_targets: &Vec<(String, String)>) -> String
{
    let mut s = format!("add_library(fpkg::{} INTERFACE IMPORTED)\n", package_name);

    for inc in include_dirs
    {
        s.push_str(&format!("target_include_directories(fpkg::{} INTERFACE $<$<CONFIG:{}>:${{{}}}/{}>)\n", package_name, &inc.configuration, macro_name, &inc.relative_path));
    }
    for (name, _) in cmake_targets
    {
        s.push_str(&format!("target_link_libraries(fpkg::{} INTERFACE {})\n", package_name, &name));
    }
    return s;
}

impl BuildGenerator for CMakeGenerator
{
    fn add_library(&mut self, package_name: &str, lib: Library) -> Result<()>
    {
        let macro_name = format!("INTERNAL_FPKG_{}_CMAKE_SELF", package_name.replace('-', "_").replace(':', "").to_uppercase());
        let pass1 = group_binaries_by_implib(&lib);
        let cmake_targets = group_binaries_by_config(pass1, &macro_name, &package_name);
        let mut s = String::from("cmake_minimum_required(VERSION 3.10)\n");
        s.push_str(&format!("set({} ${{CMAKE_CURRENT_LIST_DIR}})\n\n", &macro_name));
        for (_, data) in &cmake_targets
        {
            s.push_str(&data);
            s.push('\n');
        }
        s.push_str(&compute_master_target(&macro_name, &package_name, &lib.include_dirs, &cmake_targets));
        s.push('\n');
        if let Err(e) = fs::write(self.base_folder.join(&self.toolchain_name).join(Path::new(package_name)).join(Path::new("library.cmake")), &s)
        {
            return Err(Error::Io(ErrorDomain::Generator, e));
        }
        self.lib_names.push(String::from(package_name));
        return Ok(());
    }

    fn add_framework(&mut self, package_name: &str) -> Result<()>
    {
        self.framework_names.push(String::from(package_name));
        return Ok(());
    }

    fn generate(&mut self) -> Result<()>
    {
        let mut toolchain_cmake = String::from("cmake_minimum_required(VERSION 3.10)\n\n");
        toolchain_cmake.push_str("if (NOT DEFINED CMAKE_MODULE_PATH)\n");
        toolchain_cmake.push_str("\tset(CMAKE_MODULE_PATH \"\")\n");
        toolchain_cmake.push_str("endif (NOT DEFINED CMAKE_MODULE_PATH)\n");
        toolchain_cmake.push_str("if (NOT DEFINED BP_MODULE_PATH)\n");
        toolchain_cmake.push_str("\tset(BP_MODULE_PATH \"\")\n");
        toolchain_cmake.push_str("endif (NOT DEFINED BP_MODULE_PATH)\n\n");
        for lib in &self.lib_names
        {
            toolchain_cmake.push_str(&format!("include(\"${{CMAKE_CURRENT_LIST_DIR}}/{}/library.cmake\")\n", &lib));
        }
        toolchain_cmake.push('\n');
        for framework in &self.framework_names
        {
            toolchain_cmake.push_str(&format!("list(APPEND CMAKE_MODULE_PATH \"${{CMAKE_CURRENT_LIST_DIR}}/{}\")\n", &framework));
            toolchain_cmake.push_str(&format!("list(APPEND BP_MODULE_PATH \"${{CMAKE_CURRENT_LIST_DIR}}/{}\")\n", &framework));
        }
        if let Err(e) = fs::write(self.base_folder.join(&self.toolchain_name).join(Path::new("install.cmake")), &toolchain_cmake)
        {
            return Err(Error::Io(ErrorDomain::Generator, e));
        }
        let mut main_cmake = String::from("cmake_minimum_required(VERSION 3.10)\n\n");
        main_cmake.push_str(&format!("include(\"${{CMAKE_CURRENT_LIST_DIR}}/{}/install.cmake\")\n", &self.toolchain_name));
        if let Err(e) = fs::write(self.base_folder.join(Path::new("install.cmake")), &main_cmake)
        {
            return Err(Error::Io(ErrorDomain::Generator, e));
        }
        return Ok(());
    }
}

pub struct CMakeGeneratorProvider {}

impl BuildGeneratorProvider for CMakeGeneratorProvider
{
    fn new(&self, base_folder: &Path, toolchain_name: &str) -> Result<Box<dyn BuildGenerator>>
    {
        return Ok(Box::new(CMakeGenerator
        {
            base_folder: PathBuf::from(base_folder),
            toolchain_name: String::from(toolchain_name),
            lib_names: Vec::new(),
            framework_names: Vec::new()
        }));
    }
}

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

use std::collections::HashMap;
use std::boxed::Box;
use std::path::Path;
use std::string::String;
use std::vec::Vec;

use crate::common::Result;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::luaengine::PackageTable;
use crate::settings::RegistryInfo;
use crate::gitlabregistry::GitLabRegistryProvider;

pub trait PackageRegistry
{
    fn ensure_valid_package(&mut self, package: &PackageTable) -> Result<()>;
    fn publish(&mut self, package: &PackageTable, file_name: &str, file: &Path) -> Result<()>;
    fn find_latest(&mut self, name: &str) -> Result<Option<Vec<String>>>;
    fn find(&mut self, name: &str, version: &str) -> Result<Option<Vec<String>>>;
    fn download(&mut self, target_folder: &Path, name: &str, version: &str, file_name: &str) -> Result<()>;
}

pub trait RegistryProvider
{
    fn open(&self, info: &RegistryInfo) -> Result<Box<dyn PackageRegistry>>;
}

fn get_scheme(base_url: &str) -> Result<&str>
{
    return match base_url.find(':')
    {
        Some(i) => Ok(&base_url[0..i]),
        None => Err(Error::Generic(ErrorDomain::Settings, String::from("The specified registry URL does not have a valid scheme")))
    }
}

fn get_provider_by_scheme(scheme: &str) -> Result<Box<dyn RegistryProvider>>
{
    let mut map: HashMap<&str, Box<dyn RegistryProvider>> = HashMap::new();

    map.insert("gitlab", GitLabRegistryProvider::new());
    map.insert("gitlab-priv", GitLabRegistryProvider::new());
    if map.contains_key(&scheme)
    {
        let obj = map.remove(&scheme).unwrap();
        return Ok(obj);
    }
    return Err(Error::Generic(ErrorDomain::Settings, format!("Unknown registry URL scheme: {}", &scheme)));
}

pub fn open_package_registry(info: &RegistryInfo) -> Result<Box<dyn PackageRegistry>>
{
    let scheme = get_scheme(&info.base_url)?;
    let provider = get_provider_by_scheme(&scheme)?;
    return provider.open(&info);
}

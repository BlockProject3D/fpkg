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

use std::boxed::Box;
use std::string::String;
use std::path::Path;
use regex::Regex;
use std::fs::File;

use crate::common::Result;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::settings::RegistryInfo;
use crate::registry::PackageRegistry;
use crate::registry::RegistryProvider;
use crate::luaengine::PackageTable;

struct GitLabRegistry
{
    base_url: String,
    access_token: Option<String>,
    private: bool
}

impl PackageRegistry for GitLabRegistry
{
    fn ensure_valid_package(&self, package: &PackageTable) -> Result<()>
    {
        let re = Regex::new(r"^\A\d+\.\d+\.\d+\z$").unwrap();
        let re1 = Regex::new(r"^([a-z]|[A-Z]|\d|\.|-|_)+$").unwrap();
    
        if !re.is_match(&package.version)
        {
            return Err(Error::Generic(ErrorDomain::Publisher, format!("The package version string {} is not supported by GitLab Generic Packages", &package.version)));
        }
        if !re1.is_match(&package.name)
        {
            return Err(Error::Generic(ErrorDomain::Publisher, format!("The package name string {} is not supported by GitLab Generic Packages", &package.name)));
        }
        return Ok(());
    }

    fn publish(&self, package: &PackageTable, file_name: &str, file: &Path) -> Result<()>
    {
        if let Some(v) = &self.access_token
        {
            let mut mgr = glgp::manager::PackageManager::new(self.base_url.clone(), v.clone());
            let f = match File::open(&file)
            {
                Ok(v) => v,
                Err(e) => return Err(Error::Io(ErrorDomain::Publisher, e))
            };
            match mgr.upload(&package.name, &package.version, &file_name, f)
            {
                Ok(()) => (),
                Err(e) => return Err(Error::Generic(ErrorDomain::Publisher, format!("A HTTP request has failed: {}", e)))
            }
        }
        return Err(Error::Generic(ErrorDomain::Publisher, String::from("The registry does not have a valid access token!")))
    }
}

pub struct GitLabRegistryProvider
{
}

impl GitLabRegistryProvider
{
    pub fn new() -> Box<GitLabRegistryProvider>
    {
        return Box::new(GitLabRegistryProvider
        {

        });
    }
}

impl RegistryProvider for GitLabRegistryProvider
{
    fn open(&self, info: &RegistryInfo) -> Result<Box<dyn PackageRegistry>>
    {
        if info.base_url.starts_with("gitlab-priv")
        {
            return Ok(Box::new(GitLabRegistry
            {
                base_url: String::from(&info.base_url[16..]),
                access_token: info.access_token.clone(),
                private: true
            }));
        }
        else
        {
            return Ok(Box::new(GitLabRegistry
            {
                base_url: String::from(&info.base_url[10..]),
                access_token: info.access_token.clone(),
                private: false
            }));
        }
    }
}

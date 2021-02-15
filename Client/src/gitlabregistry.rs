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
use std::io::Read;
use std::io::Write;
use std::io;

use crate::common::Result;
use crate::common::Error;
use crate::common::ErrorDomain;
use crate::settings::RegistryInfo;
use crate::registry::PackageRegistry;
use crate::registry::RegistryProvider;
use crate::luaengine::PackageTable;

struct GitLabRegistry
{
    list: glgp::list::PackageList,
    manager: Option<glgp::manager::PackageManager>
}

impl GitLabRegistry
{
    fn find_package(&mut self, name: &str, version: &str) -> Result<Option<glgp::types::PackageEntry>>
    {
        let mut page = 1;
        loop
        {
            let mut data = match self.list.search(page, name)
            {
                Ok(v) => v,
                Err(e) => return Err(Error::Generic(ErrorDomain::Registry, format!("A HTTP request has failed: {}", e)))
            };
            if data.len() == 0
            {
                return Ok(None);
            }
            for i in 0..data.len()
            {
                if data[i].version == version
                {
                    return Ok(Some(data.remove(i)));
                }
            }
            page += 1;
        }
    }

    fn list_file_names(&mut self, package: &glgp::types::PackageEntry) -> Result<Vec<String>>
    {
        let mut res = Vec::new();
        let mut page = 1;
        loop
        {
            let mut data = match self.list.list_files(page, &package)
            {
                Ok(v) => v,
                Err(e) => return Err(Error::Generic(ErrorDomain::Registry, format!("A HTTP request has failed: {}", e)))
            };
            if data.len() == 0
            {
                break;
            }
            for i in 0..data.len()
            {
                res.push(data.remove(i).file_name);
            }
            page += 1;
        }
        return Ok(res);
    }

    fn find_file(&mut self, package: &glgp::types::PackageEntry, file_name: &str) -> Result<Option<glgp::types::PackageFile>>
    {
        let mut page = 1;
        loop
        {
            let mut data = match self.list.list_files(page, &package)
            {
                Ok(v) => v,
                Err(e) => return Err(Error::Generic(ErrorDomain::Registry, format!("A HTTP request has failed: {}", e)))
            };
            if data.len() == 0
            {
                return Ok(None);
            }
            for i in 0..data.len()
            {
                if data[i].file_name == file_name
                {
                    return Ok(Some(data.remove(i)));
                }
            }
            page += 1;
        }
    }
}

fn download_file(target: &Path, src: &mut dyn Read) -> io::Result<()>
{
    let mut buf: [u8; 8192] = [0; 8192];
    let mut f = File::create(target)?;

    loop
    {
        let bytes = src.read(&mut buf)?;
        if bytes == 0
        {
            break;
        }
        f.write(&buf[0..bytes])?;
    }
    return Ok(());
}

impl PackageRegistry for GitLabRegistry
{
    fn ensure_valid_package(&mut self, package: &PackageTable) -> Result<()>
    {
        let re = Regex::new(r"^\A\d+\.\d+\.\d+\z$").unwrap();
        let re1 = Regex::new(r"^([a-z]|[A-Z]|\d|\.|-|_)+$").unwrap();
    
        if !re.is_match(&package.version)
        {
            return Err(Error::Generic(ErrorDomain::Registry, format!("The package version string {} is not supported by GitLab Generic Packages", &package.version)));
        }
        if !re1.is_match(&package.name)
        {
            return Err(Error::Generic(ErrorDomain::Registry, format!("The package name string {} is not supported by GitLab Generic Packages", &package.name)));
        }
        return Ok(());
    }

    fn publish(&mut self, package: &PackageTable, file_name: &str, file: &Path) -> Result<()>
    {
        if let Some(pkg) = self.find_package(&package.name, &package.version)?
        {
            if let Some(_) = self.find_file(&pkg, &file_name)?
            {
                return Err(Error::Generic(ErrorDomain::Registry, format!("A package release already exists for the combination {}>{}>{}", &package.name, &package.version, &file_name)));
            }
        }
        if let Some(mgr) = &mut self.manager
        {
            let f = match File::open(&file)
            {
                Ok(v) => v,
                Err(e) => return Err(Error::Io(ErrorDomain::Registry, e))
            };
            match mgr.upload(&package.name, &package.version, &file_name, f)
            {
                Ok(()) => return Ok(()),
                Err(e) => return Err(Error::Generic(ErrorDomain::Registry, format!("A HTTP request has failed: {}", e)))
            }
        }
        return Err(Error::Generic(ErrorDomain::Registry, String::from("The registry does not have a valid access token!")))
    }

    fn find_latest(&mut self, name: &str) -> Result<Option<Vec<String>>>
    {
        let data = match self.list.search(1, name)
        {
            Ok(v) => v,
            Err(e) => return Err(Error::Generic(ErrorDomain::Registry, format!("A HTTP request has failed: {}", e)))
        };
        if data.len() == 0
        {
            return Ok(None);
        }
        let fuck = self.list_file_names(&data[0])?;
        return Ok(Some(fuck));
    }

    fn find(&mut self, name: &str, version: &str) -> Result<Option<Vec<String>>>
    {
        let package = self.find_package(&name, &version)?;
        if let Some(pkg) = package
        {
            let fuck = self.list_file_names(&pkg)?;
            return Ok(Some(fuck));
        }
        return Ok(None);
    }

    fn download(&mut self, target_folder: &Path, name: &str, version: &str, file_name: &str) -> Result<()>
    {
        if let Some(mgr) = &mut self.manager
        {
            let pkg = glgp::types::PackageEntry
            {
                id: 0,
                version: String::from(version),
                name: String::from(name)
            };
            let file = glgp::types::PackageFile
            {
                id: 0,
                file_name: String::from(file_name),
                size: 0
            };
            match mgr.download(&pkg, &file)
            {
                Err(e) => return Err(Error::Generic(ErrorDomain::Registry, format!("A HTTP request has failed: {}", e))),
                Ok(mut response) =>
                {
                    if let Err(e) = download_file(&target_folder.join(Path::new(file_name)), &mut response)
                    {
                        return Err(Error::Io(ErrorDomain::Registry, e));
                    }
                    return Ok(());
                }
            };
        }
        return Err(Error::Generic(ErrorDomain::Registry, String::from("The registry does not have a valid access token!")));
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
            if let Some(v) = &info.access_token
            {
                return Ok(Box::new(GitLabRegistry
                {
                    list: glgp::list::PackageList::new_authenticated(String::from(&info.base_url[15..]), v.clone()),
                    manager: Some(glgp::manager::PackageManager::new(String::from(&info.base_url[15..]), v.clone()))
                }));
            }
            return Err(Error::Generic(ErrorDomain::Publisher, String::from("The registry does not have a valid access token!")));
        }
        else
        {
            if let Some(v) = &info.access_token
            {
                return Ok(Box::new(GitLabRegistry
                {
                    list: glgp::list::PackageList::new_guest(String::from(&info.base_url[9..])),
                    manager: Some(glgp::manager::PackageManager::new(String::from(&info.base_url[9..]), v.clone()))
                }));
            }
            else
            {
                return Ok(Box::new(GitLabRegistry
                {
                    list: glgp::list::PackageList::new_guest(String::from(&info.base_url[9..])),
                    manager: None
                }));
            }
        }
    }
}

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

use reqwest::blocking::Client;
use reqwest::blocking::Response;
use std::fs::File;
use regex::Regex;

use crate::types::Result;
use crate::types::PackageEntry;
use crate::types::PackageFile;

pub struct PackageManager
{
    client: Client,
    base_url: String, //This way we support other gitlab instances (not just gitlab.com)
    access_token: String
}

impl PackageManager
{
    pub fn new(base_url: String, access_token: String) -> PackageManager
    {
        return PackageManager
        {
            client: Client::new(),
            base_url: base_url,
            access_token: access_token
        };
    }

    pub fn download(&mut self, package: &PackageEntry, file: &PackageFile) -> Result<Response>
    {
        let mut path = self.base_url.clone();
        if path.ends_with("/")
        {
            path.push_str("packages/generic/");
        }
        else
        {
            path.push_str("/packages/generic/");
        }
        path.push_str(&package.name);
        path.push('/');
        path.push_str(&package.version);
        path.push('/');
        path.push_str(&file.file_name);
        return self.client.get(&path).header("PRIVATE-TOKEN", &self.access_token).send();
    }

    pub fn upload(&mut self, package_name: &str, package_version: &str, file_name: &str, file: File) -> Result<()>
    {
        let re = Regex::new(r"^\A\d+\.\d+\.\d+\z$").unwrap();
        let re1 = Regex::new(r"^([a-z]|[A-Z]|\d|\.|-|_)+$").unwrap();
        assert!(re.is_match(package_version)); // It is a programmer error to pass non matched strings
        //See https://docs.gitlab.com/ee/user/packages/generic_packages/ to know why those stupid restrictions exists
        assert!(re1.is_match(package_name));
        assert!(re1.is_match(file_name));
        let mut path = self.base_url.clone();
        if path.ends_with("/")
        {
            path.push_str("packages/generic/");
        }
        else
        {
            path.push_str("/packages/generic/");
        }
        path.push_str(&package_name);
        path.push('/');
        path.push_str(&package_version);
        path.push('/');
        path.push_str(&file_name);
        self.client.put(&path).header("PRIVATE-TOKEN", &self.access_token).body(file).send()?;
        return Ok(());
    }
}

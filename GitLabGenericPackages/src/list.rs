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

use std::string::String;
use std::vec::Vec;
use reqwest::blocking::Client;

use crate::types::Result;
use crate::types::PackageEntry;
use crate::types::PackageFile;

pub struct PackageList
{
    client: Client,
    base_url: String, //This way we support other gitlab instances (not just gitlab.com)
    access_token: Option<String>, //Allow specifying a custom access token typically to use private registries
}

impl PackageList
{
    pub fn new_authenticated(base_url: String, access_token: String) -> PackageList
    {
        return PackageList
        {
            client: Client::new(),
            base_url: base_url,
            access_token: Some(access_token)
        };
    }

    pub fn new_guest(base_url: String) -> PackageList
    {
        return PackageList
        {
            client: Client::new(),
            base_url: base_url,
            access_token: None
        };
    }

    pub fn get_page(&mut self, page: usize) -> Result<Vec<PackageEntry>>
    {
        let mut path = self.base_url.clone();
        if path.ends_with("/")
        {
            path.push_str("packages");
        }
        else
        {
            path.push_str("/packages");
        }
        let mut request = self.client.get(&path).query(&[
            ("package_type", "generic"),
            ("order_by", "version"),
            ("sort", "desc"),
            ("per_page", "100"),
            ("page", &page.to_string())
        ]);
        if let Some(token) = &self.access_token
        {
            request = request.header("PRIVATE-TOKEN", token);
        }
        let data = request.send()?.json::<Vec<PackageEntry>>();
        return data;
    }

    pub fn search(&mut self, page: usize, name: &str) -> Result<Vec<PackageEntry>>
    {
        let mut path = self.base_url.clone();
        if path.ends_with("/")
        {
            path.push_str("packages");
        }
        else
        {
            path.push_str("/packages");
        }
        let mut request = self.client.get(&path).query(&[
            ("package_type", "generic"),
            ("package_name", name),
            ("order_by", "version"),
            ("sort", "desc"),
            ("per_page", "100"),
            ("page", &page.to_string())
        ]);
        if let Some(token) = &self.access_token
        {
            request = request.header("PRIVATE-TOKEN", token);
        }
        let data = request.send()?.json::<Vec<PackageEntry>>();
        return data;
    }

    pub fn list_files(&mut self, page: usize, package: &PackageEntry) -> Result<Vec<PackageFile>>
    {
        let mut path = self.base_url.clone();
        if path.ends_with("/")
        {
            path.push_str("packages/");
        }
        else
        {
            path.push_str("/packages/");
        }
        path.push_str(&package.id.to_string());
        path.push_str("/package_files");
        let mut request = self.client.get(&path).query(&[("page_count", "100"), ("page", &page.to_string())]);
        if let Some(token) = &self.access_token
        {
            request = request.header("PRIVATE-TOKEN", token);
        }
        let data = request.send()?.json::<Vec<PackageFile>>();
        return data;
    }
}

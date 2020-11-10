use std::collections::HashMap;
use std::string::String;
use std::path::Path;
use std::fs;

pub struct Profile
{
    data: HashMap<String, String>
}

fn read_profile(path: &Path, map: &mut HashMap<String, String>)
{
    let res = fs::read_to_string(path);
    if res.is_err() {
        return;
    }
    let jres = json::parse(&res.unwrap());
    if jres.is_err() {
        return;
    }
    let json = jres.unwrap();
    for v in json.entries()
    {
        let (f, f1) = v;
        map.insert(String::from(f), f1.to_string());
    }
}

impl Profile
{
    pub fn new() -> Profile
    {
        let mut map = HashMap::new();
        let p = Path::new("./.fpkg/profile");

        if p.exists() {
            read_profile(p, &mut map);
        }
        return Profile
        {
            data: map
        };
    }

    pub fn get(&self, name: &str) -> Option<&String>
    {
        return self.data.get(&String::from(name));
    }

    pub fn regenerate_cross(&mut self, _name: &str) -> bool //Regenerate profile for cross-compile platform
    {
        return false;
    }

    pub fn regenerate_self(&mut self) //Regenerate profile for current platform
    {
        if cfg!(target_os = "windows") {
            self.data.insert(String::from("Platform"), String::from("Windows"));
        } else if cfg!(target_os = "macos") {
            self.data.insert(String::from("Platform"), String::from("OSX"));
        } else if cfg!(target_os = "linux") {
            self.data.insert(String::from("Platform"), String::from("Linux"));
        } else if cfg!(target_os = "android") {
            self.data.insert(String::from("Platform"), String::from("Android"));
        }
        if cfg!(target_arch = "x86") {
            self.data.insert(String::from("Arch"), String::from("x86"));
        } else if cfg!(target_arch = "x86_64") {
            self.data.insert(String::from("Arch"), String::from("x86_64"));
        } else if cfg!(target_arch = "arm") {
            self.data.insert(String::from("Arch"), String::from("arm"));
        } else if cfg!(target_arch = "aarch64") {
            self.data.insert(String::from("Arch"), String::from("aarch64"));
        }
    }
}
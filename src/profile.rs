use std::collections::HashMap;
use std::string::String;
use std::path::Path;
use std::path::PathBuf;
use std::fs;
use std::io;

#[cfg(windows)]
use winapi::um::fileapi;

pub struct Profile
{
    path: PathBuf,
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
    pub fn mkdir(path: &Path) -> io::Result<()>
    {
        let p = path.join(Path::new("/.fpkg/profile"));
        if !p.exists() {
            fs::create_dir(&p)?;
            #[cfg(windows)]
            fileapi::SetFileAttributesA(&p, fileapi::FILE_ATTRIBUTE_HIDDEN);
        }
        return Ok(());
    }

    pub fn new(path: &Path) -> Profile
    {
        let mut map = HashMap::new();
        let p = path.join(Path::new("/.fpkg/profile"));

        if p.exists() {
            read_profile(&p, &mut map);
        }
        return Profile
        {
            path: p,
            data: map
        };
    }

    pub fn exists(&self) -> bool
    {
        return self.path.exists();
    }

    pub fn get(&self, name: &str) -> Option<&String>
    {
        return self.data.get(&String::from(name));
    }

    pub fn regenerate_cross(&mut self, _name: &str) -> bool //Regenerate profile for cross-compile platform
    {
        return false;
    }

    pub fn write(&self) -> io::Result<()>
    {
        let mut json = json::JsonValue::new_object();

        for (k, v) in &self.data
        {
            json[k] = json::JsonValue::String(v.to_string());
        }
        fs::write(&self.path, json.dump())?;
        return Ok(());
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
        //TODO: Identify compiler ID and version
    }
}

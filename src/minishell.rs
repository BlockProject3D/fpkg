use std::string::String;
use std::collections::HashMap;

pub type BuiltInFunc = 

//Represents a cross-platform integrated shell
pub struct Minishell
{
    env: HashMap<String, String>
    builtins: HashMap
}

impl Minishell
{
    pub fn new() -> Minishell
    {
        let mut shell = Minishell
        {
            env: HashMap::new()
        };

        for (k, v) in std::env::vars()
        {
            shell.env.insert(k, v);
        }
        return shell;
    }

    pub fn setenv(&mut self, key: &str, value: &str)
    {
        self.env.insert(String::from(key), String::from(value));
    }

    pub fn getenv(&self, key: &str) -> Option<&str>
    {
        match self.env.get(key)
        {
            Some(v) => return Some(&v),
            None => return None
        }
    }

    pub fn run(command: &str)
    {

    }
}

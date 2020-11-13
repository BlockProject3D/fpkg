use std::string::String;

pub enum Shell
{
    Bash,
    PowerShell,
    Cmd,
    PlatformIndependent
}

pub struct Command
{
    pub cmd_str: String,
    pub shell: Shell
}

impl Command
{
    pub fn new(s: &str, shell: Shell) -> Command
    {
        return Command
        {
            cmd_str: String::from(s),
            shell: shell
        };
    }
}

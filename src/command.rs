use std::string::String;

pub enum Shell
{
    Bash,
    PowerShell,
    Cmd,
    Unspecified
}

pub struct Command
{
    pub cmd_str: String,
    pub shell: Shell
}

impl Command
{
    pub fn new(s: &str) -> Command
    {
        return Command
        {
            cmd_str: String::from(s),
            shell: Shell::Unspecified
        };
    }

    //Because rust neither has default parameter nor overload code-duplication with weird function name is required
    pub fn new1(s: &str, shell: Shell) -> Command
    {
        return Command
        {
            cmd_str: String::from(s),
            shell: shell
        };
    }
}

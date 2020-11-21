/*
digit = "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9";
symbol_full =  "!" | "$" | "&" | ";" | "|" | " " | "<" | ">";
symbol =  "#" | "%" | "(" | ")" | "*" | "+" | "," | "-" | "." | "/" | ":" | "=" | "?" | "@" | "[" | "\\" | "]" | "^" | "_" | "`" | "{" | "}" | "~";
letter = "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M" | "N" | "O" | "P" | "Q" | "R" | "S" | "T" | "U" | "V" | "W" | "X" | "Y" | "Z" | "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n" | "o" | "p" | "q" | "r" | "s" | "t" | "u" | "v" | "w" | "x" | "y" | "z";
escaped_symbol = ("\\" symbol_full) | ("\\" "\"");
character = letter | digit | symbol | escaped_symbol;
quoted_string_character = letter | digit | symbol | symbol_full | ("\\" "\"");

string = {character};
quoted_string = "\"" {quoted_string_character} "\"";

command_argument = string | quoted_string;
command = {command_argument " "};

redirect_in = "<" string;
redirect_out = ">" string;

pipe = command [redirect_in] "|" {command "|"} command [redirect_out];
or = command [redirect_in] [redirect_out] "||" command [redirect_in] [redirect_out];
and = command [redirect_in] [redirect_out] "&&" command [redirect_in] [redirect_out];
multi = command [redirect_in] [redirect_out] {";" command [redirect_in] [redirect_out]};
statement = pipe | or | and | multi;

command_line = {statement};
*/

use std::vec::Vec;
use std::string::String;

pub struct Command
{
    arguments: Vec<String>,
    redirect_in: Option<String>,
    redirect_out: Option<String>
}

pub enum Statement
{
    Pipe(Vec<Command>),
    Or((Command, Command)),
    And((Command, Command)),
    CommandList(Vec<Command>)
}

pub struct CommandLine
{
    statements: Vec<Statement>
}
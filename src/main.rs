use std::io::Write;
use std::{io, str::FromStr};

use crate::Cmd::{EchoCmd, ExitCmd, NotRecognisedCmd};

enum Cmd {
    ExitCmd(i32),
    EchoCmd(String),
    NotRecognisedCmd(String),
}

fn parse_command(cmd: &str) -> Cmd {
    let slice: Vec<&str> = cmd.split_whitespace().collect();
    match &slice[..] {
        ["exit"] => ExitCmd(0),
        ["echo", rest @ ..] => EchoCmd(rest.join(" ").to_string()),
        a => NotRecognisedCmd(a.join(" ").to_string()),
    }
}

fn main() {
    let mut cmd = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let result = io::stdin().read_line(&mut cmd);
        let cmd_trimmed = cmd.trim();
        match result {
            Ok(_size) => match parse_command(cmd_trimmed) {
                ExitCmd(num) => {
                    break;
                },
                EchoCmd(s) => {
                    println!("{}", s)
                },
                NotRecognisedCmd(cm) => {
                    println!("{}: command not found", cm)
                }
            },
            Err(er) => {
                println!("Error reading from stdin {}", er.to_string())
            }
        }
        cmd.clear();
    }
}

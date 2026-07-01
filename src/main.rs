use std::io;
use std::io::Write;
use std::ops::Deref;

enum Cmd {
    ExitCmd(i32),
    EchoCmd(String),
    NotRecognisedCmd(String),
    TypeCmd(String),
}

fn parse_command(cmd: &str) -> Cmd {
    let slice: Vec<&str> = cmd.split_whitespace().collect();
    match &slice[..] {
        ["exit"] => Cmd::ExitCmd(0),
        ["echo", rest @ ..] => Cmd::EchoCmd(rest.join(" ").to_string()),
        ["type", cmd] => Cmd::TypeCmd(cmd.to_string()),
        a => Cmd::NotRecognisedCmd(a.join(" ").to_string()),
    }
}

fn is_builtin(cmd: &String) -> bool {
    ["echo", "exit", "type"].contains(&cmd.as_str())
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
                Cmd::ExitCmd(num) => {
                    break;
                }
                Cmd::EchoCmd(s) => {
                    println!("{}", s)
                }
                Cmd::TypeCmd(c) => {
                    if is_builtin(&c) {
                        println!("{} is a shell builtin", c)
                    } else {
                        println!("{}: not found", c)
                    }
                }
                Cmd::NotRecognisedCmd(cm) => {
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

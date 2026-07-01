use std::collections::HashMap;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::{env, fs, io};

enum Type {
    Alias,
    Builtin,
    Function,
    File(String),
    Keyword,
    NotRecognised,
}

enum Cmd {
    ExitCmd(i32),
    EchoCmd(String),
    TypeCmd(String, Type),
    ExportCmd(String, (String, String)),
    NotRecognisedCmd(String),
}

struct State {
    env: HashMap<String, String>,
    alias: HashMap<String, String>,
    keywords: Vec<String>,
    functions: HashMap<String, String>,
}

fn parse_command(cmd: &str, state: &State) -> Cmd {
    let slice: Vec<&str> = cmd.split_whitespace().collect();
    match &slice[..] {
        ["exit"] => Cmd::ExitCmd(0),
        ["echo", rest @ ..] => Cmd::EchoCmd(rest.join(" ").to_string()),
        ["type", cmd] => Cmd::TypeCmd(cmd.to_string(), which_type(cmd.to_string(), state)),
        ["export", env_var] => {
            let (key, value) = env_var.split_once("=").expect("Malformed ");
            Cmd::ExportCmd(cmd.to_string(), (key.to_string(), value.to_string()))
        }
        a => Cmd::NotRecognisedCmd(a.join(" ").to_string()),
    }
}

fn which_type(cmd: String, state: &State) -> Type {
    match cmd.as_str() {
        "echo" | "exit" | "type" => Type::Builtin,
        alias if state.alias.contains_key(alias) => Type::Alias,
        keyword if state.keywords.contains(&keyword.to_string()) => Type::Keyword,
        function if state.functions.contains_key(function) => Type::Function,
        _ => match state.env.get("PATH") {
            Some(path) => {
                let p = path
                    .as_str()
                    .split(":")
                    .into_iter()
                    .filter(|path| {
                        let meta = fs::metadata(format!("{}/{}", path, cmd));
                        match meta {
                            Ok(r) => r.permissions().mode() & 0o111 != 0,
                            Err(e) => false,
                        }
                    })
                    .last();

                if p.is_some() {
                    Type::File(format!("{}/{}", p.unwrap_or(""), cmd))
                } else {
                    Type::NotRecognised
                }
            }
            None => Type::NotRecognised,
        },
    }
}

fn main() {
    let path = env::var("PATH").unwrap_or("".to_string());
    let mut state = State {
        env: HashMap::from([("PATH".to_string(), path)]),
        alias: HashMap::new(),
        keywords: vec![
            "if", "then", "elif", "else", "fi", "case", "in", "esac", "for", "while", "until",
            "do", "done", "{", "}", "!",
        ]
        .iter()
        .map(|e| e.to_string())
        .collect(),
        functions: HashMap::new(),
    };
    let mut cmd = String::new();
    let default_prompt = &"$".to_string();
    loop {
        let prompt = &state.env.get("PROMPT").unwrap_or(default_prompt).clone();
        print!("{} ", prompt);
        io::stdout().flush().unwrap();
        let result = io::stdin().read_line(&mut cmd);
        let cmd_trimmed = cmd.trim();
        match result {
            Ok(_size) => match parse_command(cmd_trimmed, &state) {
                Cmd::ExitCmd(_num) => {
                    break;
                }
                Cmd::EchoCmd(s) => {
                    println!("{}", s)
                }
                Cmd::TypeCmd(c, typ) => match typ {
                    Type::Builtin => println!("{} is a shell builtin", c),
                    Type::Keyword => println!("{} is a reserved word", c),
                    Type::File(path) => println!("{} is {}", c, path),
                    _ => println!("{}: not found", c),
                },
                Cmd::ExportCmd(_c, (k, v)) => {
                    state.env.insert(k, v);
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

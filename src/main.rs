use std::collections::HashMap;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
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
    PwdCmd,
    TypeCmd(String, Type),
    ExportCmd(String, (String, String)),
    BinaryCmd(String, Vec<String>),
    CarriageReturn,
    NotRecognisedCmd(String),
}

struct State {
    env: HashMap<String, String>,
    alias: HashMap<String, String>,
    keywords: Vec<String>,
    functions: HashMap<String, String>,
    current_dir: String,
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

        ["pwd"] => Cmd::PwdCmd,
        a => {
            let fst = a.first().unwrap_or(&"");
            if fst.is_empty() {
                return Cmd::CarriageReturn;
            }
            match cmd_exists(fst.to_string(), state) {
                Some(_) => Cmd::BinaryCmd(
                    fst.to_string(),
                    a.iter().skip(1).map(|arg| arg.to_string()).collect(),
                ),
                None => Cmd::NotRecognisedCmd(cmd.to_string()),
            }
        }
    }
}

fn cmd_exists(cmd: String, state: &State) -> Option<String> {
    state.env.get("PATH").and_then(|path| {
        path.as_str()
            .split(":")
            .into_iter()
            .filter(|path| {
                let meta = fs::metadata(format!("{}/{}", path, cmd));
                match meta {
                    Ok(r) => r.permissions().mode() & 0o111 != 0,
                    Err(_) => false,
                }
            })
            .map(|path| path.to_string())
            .next()
    })
}

fn which_type(cmd: String, state: &State) -> Type {
    match cmd.as_str() {
        "echo" | "exit" | "type" => Type::Builtin,
        alias if state.alias.contains_key(alias) => Type::Alias,
        keyword if state.keywords.contains(&keyword.to_string()) => Type::Keyword,
        function if state.functions.contains_key(function) => Type::Function,
        _ => match cmd_exists(cmd.clone(), state) {
            Some(path) => Type::File(format!("{}/{}", path, cmd)),
            None => Type::NotRecognised,
        },
    }
}

fn main() {
    let path = env::var("PATH").unwrap_or("".to_string());
    let mut state = State {
        env: HashMap::from([("PATH".to_string(), path)]),
        alias: HashMap::new(),
        keywords: [
            "if", "then", "elif", "else", "fi", "case", "in", "esac", "for", "while", "until",
            "do", "done", "{", "}", "!",
        ]
        .iter()
        .map(|e| e.to_string())
        .collect(),
        functions: HashMap::new(),
        current_dir: env::current_dir()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_string(),
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
                Cmd::CarriageReturn => {
                    println!()
                }
                Cmd::BinaryCmd(command, args) => {
                    let out = Command::new(command).args(args.iter().as_slice()).output();
                    match out {
                        Ok(o) => {
                            print!("{}", String::from_utf8_lossy(o.stdout.iter().as_slice()));
                        }
                        Err(e) => {
                            print!("{}", e.to_string());
                        }
                    }
                }
                Cmd::PwdCmd => {
                    println!("{}", state.current_dir)
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

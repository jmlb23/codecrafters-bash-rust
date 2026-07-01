use std::io;
use std::io::Write;
use std::process::exit;

fn parse_command(cmd: &str) -> i32 {
    match cmd {
        "exit" => 0,
        _ => 1,
    }
}

fn main() {
    let mut cmd = String::new();
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let result = io::stdin().read_line(&mut cmd);
        match result {
            Ok(_size) => {
                println!("{}: command not found", cmd.trim())
            }
            Err(er) => {
                println!("Error reading from stdin {}", er.to_string())
            }
        }
        cmd.clear();
    }
}

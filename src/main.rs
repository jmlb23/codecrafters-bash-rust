use std::io;
use std::io::{Write};

fn main() {
    let mut cmd = String::new();
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
}

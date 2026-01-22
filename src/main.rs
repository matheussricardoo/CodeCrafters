#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        let bytes_read = io::stdin().read_line(&mut buffer).unwrap();
        let clean_input = buffer.trim();

        let (command, args) = match clean_input.split_once(' ') {
            Some((cmd, rest)) => (cmd, rest),
            None => (clean_input, ""),
        };

        match command {
            "exit" => break,
            "" => continue,
            "echo" => {
                println!("{}", args)
            }
            "type" => {
                if args == "echo" {
                    println!("echo is a shell builtin");
                } else if args == "exit" {
                    println!("exit is a shell builtin");
                } else if args == "type" {
                    println!("type is a shell builtin");
                } else {
                    println!("{}: not found", args);
                }
            }
            _ => println!("{}: command not found", clean_input),
        }
    }
}

use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;

const BUILTINS: [&str; 3] = ["exit", "echo", "type"];

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
                if BUILTINS.contains(&args) {
                    println!("{} is a shell builtin", args);
                } else {
                    let path_var = std::env::var("PATH").unwrap_or_default();
                    let mut found = false;

                    for dir in path_var.split(':') {
                        let mut full_path = std::path::PathBuf::from(dir);
                        full_path.push(args);

                        if let Ok(metadata) = fs::metadata(&full_path) {
                            if metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
                                println!("{} is {}", args, full_path.display());
                                found = true;
                                break;
                            }
                        }
                    }

                    if !found {
                        println!("{}: not found", args);
                    }
                }
            }
            _ => println!("{}: command not found", clean_input),
        }
    }
}

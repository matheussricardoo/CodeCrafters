use std::env;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

const BUILTINS: [&str; 5] = ["exit", "echo", "type", "pwd", "cd"];

fn find_executable(command_name: &str) -> Option<PathBuf> {
    let path_var = env::var("PATH").unwrap_or_default();

    for dir in path_var.split(':') {
        let mut full_path = PathBuf::from(dir);
        full_path.push(command_name);

        if let Ok(metadata) = fs::metadata(&full_path) {
            if metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0) {
                return Some(full_path);
            }
        }
    }
    None
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        let _bytes_read = io::stdin().read_line(&mut buffer).unwrap();
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
                    match find_executable(args) {
                        Some(path) => println!("{} is {}", args, path.display()),
                        None => println!("{}: not found", args),
                    }
                }
            }
            "pwd" => match env::current_dir() {
                Ok(path) => {
                    println!("{}", path.display());
                }
                Err(e) => {
                    eprintln!("Error retrieving directory: {}", e)
                }
            },
            "cd" => {
                let new_dir = if args == "~" {
                    match env::var("HOME") {
                        Ok(path) => path,
                        Err(_) => {
                            println!("cd: HOME not set");
                            continue;
                        }
                    }
                } else {
                    args.to_string()
                };
                let path = Path::new(&new_dir);
                if let Err(_) = env::set_current_dir(path) {
                    println!("cd: {}: No such file or directory", new_dir);
                }
            }
            _ => match find_executable(command) {
                Some(path) => {
                    let res = Command::new(path)
                        .arg0(command)
                        .args(args.split_whitespace())
                        .status();

                    if let Err(e) = res {
                        eprintln!("Error while executing: {}", e);
                    }
                }
                None => {
                    println!("{}: command not found", clean_input);
                }
            },
        }
    }
}

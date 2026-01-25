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

fn parse_input(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current_arg = String::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut escaped = false;

    for c in input.chars() {
        if in_single_quote {
            if c == '\'' {
                in_single_quote = false;
            } else {
                current_arg.push(c);
            }
        } else if in_double_quote {
            if escaped {
                current_arg.push(c);
                escaped = false
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_double_quote = false
            } else {
                current_arg.push(c);
            }
        } else {
            if c == '\'' {
                in_single_quote = true;
            } else if c == '"' {
                in_double_quote = true;
            } else if c.is_whitespace() {
                if !current_arg.is_empty() {
                    args.push(current_arg);
                    current_arg = String::new();
                }
            } else {
                current_arg.push(c);
            }
        }
    }

    if !current_arg.is_empty() {
        args.push(current_arg);
    }

    args
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut buffer = String::new();
        let _bytes_read = io::stdin().read_line(&mut buffer).unwrap();
        let clean_input = buffer.trim();

        let parsed_args = parse_input(clean_input);

        if parsed_args.is_empty() {
            continue;
        }

        let command = &parsed_args[0];
        let args = &parsed_args[1..];

        match command.as_str() {
            "exit" => break,
            "echo" => {
                println!("{}", args.join(" "))
            }
            "type" => {
                if let Some(arg) = args.get(0) {
                    if BUILTINS.contains(&arg.as_str()) {
                        println!("{} is a shell builtin", arg);
                    } else {
                        match find_executable(arg) {
                            Some(path) => println!("{} is {}", arg, path.display()),
                            None => println!("{}: not found", arg),
                        }
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
                let arg = args.get(0).map(|s| s.as_str()).unwrap_or("~");
                let new_dir = if arg == "~" {
                    match env::var("HOME") {
                        Ok(path) => path,
                        Err(_) => {
                            println!("cd: HOME not set");
                            continue;
                        }
                    }
                } else {
                    arg.to_string()
                };
                let path = Path::new(&new_dir);
                if let Err(_) = env::set_current_dir(path) {
                    println!("cd: {}: No such file or directory", new_dir);
                }
            }
            _ => match find_executable(command) {
                Some(path) => {
                    let res = Command::new(path).arg0(command).args(args).status();

                    if let Err(e) = res {
                        eprintln!("Error while executing: {}", e);
                    }
                }
                None => {
                    println!("{}: command not found", command);
                }
            },
        }
    }
}

use std::collections::HashSet;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::builtins::BUILTINS;
use crate::parser::{parse_input, split_by_pipe};
use crate::terminal::{disable_raw_mode, enable_raw_mode};

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

pub fn execute_command_line(input: &str) -> bool {
    let clean_input = input.trim();
    if clean_input.is_empty() {
        return false;
    }

    let parsed_args = parse_input(clean_input);
    let commands = split_by_pipe(parsed_args);

    if commands.len() > 1 {
        execute_pipeline(commands);
        return false;
    }

    let mut parsed_args = commands.into_iter().next().unwrap_or_default();
    let mut output_file: Option<File> = None;
    let mut error_file: Option<File> = None;

    if let Some(index) = parsed_args
        .iter()
        .position(|arg| arg == ">>" || arg == "1>>")
    {
        if index + 1 < parsed_args.len() {
            let filename = &parsed_args[index + 1];

            if let Some(parent) = Path::new(filename).parent() {
                let _ = fs::create_dir_all(parent);
            }

            match OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(filename)
            {
                Ok(file) => {
                    output_file = Some(file);
                    parsed_args.drain(index..=index + 1);
                }
                Err(e) => {
                    eprintln!("Failed to open file for appending: {}", e);
                    return false;
                }
            }
        }
    } else if let Some(index) = parsed_args.iter().position(|arg| arg == ">" || arg == "1>") {
        if index + 1 < parsed_args.len() {
            let filename = &parsed_args[index + 1];

            match File::create(filename) {
                Ok(file) => {
                    output_file = Some(file);
                    parsed_args.drain(index..=index + 1);
                }
                Err(e) => {
                    eprintln!("Failed to create file: {}", e);
                    return false;
                }
            }
        }
    }

    if let Some(index) = parsed_args.iter().position(|arg| arg == "2>>") {
        if index + 1 < parsed_args.len() {
            let filename = &parsed_args[index + 1];

            if let Some(parent) = Path::new(filename).parent() {
                let _ = fs::create_dir_all(parent);
            }

            match OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(filename)
            {
                Ok(file) => {
                    error_file = Some(file);
                    parsed_args.drain(index..=index + 1);
                }
                Err(e) => {
                    eprintln!("Failed to open file for appending stderr: {}", e);
                    return false;
                }
            }
        }
    } else if let Some(index) = parsed_args.iter().position(|arg| arg == "2>") {
        if index + 1 < parsed_args.len() {
            let filename = &parsed_args[index + 1];
            match File::create(filename) {
                Ok(file) => {
                    error_file = Some(file);
                    parsed_args.drain(index..=index + 1);
                }
                Err(e) => {
                    eprintln!("Failed to create error file: {}", e);
                    return false;
                }
            }
        }
    }

    if parsed_args.is_empty() {
        return false;
    }

    let command = &parsed_args[0];
    let args = &parsed_args[1..];

    match command.as_str() {
        "exit" => return true,
        "echo" => {
            let output = args.join(" ");
            match output_file {
                Some(mut file) => {
                    if let Err(e) = writeln!(file, "{}", output) {
                        eprintln!("Error writing to file: {}", e);
                    }
                }
                None => println!("{}", output),
            }
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
                        return false;
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
                let command_name = Path::new(command).file_name().unwrap().to_str().unwrap();

                let stdout_dest = match output_file {
                    Some(file) => Stdio::from(file),
                    None => Stdio::inherit(),
                };

                let stderr_dest = match error_file {
                    Some(file) => Stdio::from(file),
                    None => Stdio::inherit(),
                };

                disable_raw_mode();

                let res = Command::new(&path)
                    .arg0(command_name)
                    .args(args)
                    .stdout(stdout_dest)
                    .stderr(stderr_dest)
                    .status();

                enable_raw_mode();

                if let Err(e) = res {
                    eprintln!("Error while executing: {}", e);
                }
            }
            None => {
                println!("{}: command not found", command);
            }
        },
    }
    false
}

fn execute_pipeline(commands: Vec<Vec<String>>) {
    if commands.len() != 2 {
        eprintln!("Only two-command pipelines are supported");
        return;
    }

    let cmd1_args = &commands[0];
    let cmd2_args = &commands[1];

    if cmd1_args.is_empty() || cmd2_args.is_empty() {
        eprintln!("Empty command in pipeline");
        return;
    }

    let cmd1_name = &cmd1_args[0];
    let cmd1_rest = &cmd1_args[1..];
    let cmd2_name = &cmd2_args[0];
    let cmd2_rest = &cmd2_args[1..];

    let cmd1_path = match find_executable(cmd1_name) {
        Some(path) => path,
        None => {
            println!("{}: command not found", cmd1_name);
            return;
        }
    };

    let cmd2_path = match find_executable(cmd2_name) {
        Some(path) => path,
        None => {
            println!("{}: command not found", cmd2_name);
            return;
        }
    };

    let mut pipe_fds: [libc::c_int; 2] = [0; 2];
    unsafe {
        if libc::pipe(pipe_fds.as_mut_ptr()) == -1 {
            eprintln!("Failed to create pipe");
            return;
        }
    }

    let pipe_read_fd = pipe_fds[0];
    let pipe_write_fd = pipe_fds[1];

    disable_raw_mode();

    let pid1 = unsafe { libc::fork() };

    if pid1 == -1 {
        eprintln!("Failed to fork for first command");
        unsafe {
            libc::close(pipe_read_fd);
            libc::close(pipe_write_fd);
        }
        enable_raw_mode();
        return;
    }

    if pid1 == 0 {
        unsafe {
            libc::close(pipe_read_fd);
            libc::dup2(pipe_write_fd, libc::STDOUT_FILENO);
            libc::close(pipe_write_fd);
        }

        let cmd1_file_name = Path::new(cmd1_name).file_name().unwrap().to_str().unwrap();

        let _ = Command::new(&cmd1_path)
            .arg0(cmd1_file_name)
            .args(cmd1_rest)
            .exec();

        std::process::exit(1);
    }

    let pid2 = unsafe { libc::fork() };

    if pid2 == -1 {
        eprintln!("Failed to fork for second command");
        unsafe {
            libc::close(pipe_read_fd);
            libc::close(pipe_write_fd);
        }
        enable_raw_mode();
        return;
    }

    if pid2 == 0 {
        unsafe {
            libc::close(pipe_write_fd);
            libc::dup2(pipe_read_fd, libc::STDIN_FILENO);
            libc::close(pipe_read_fd);
        }

        let cmd2_file_name = Path::new(cmd2_name).file_name().unwrap().to_str().unwrap();

        let _ = Command::new(&cmd2_path)
            .arg0(cmd2_file_name)
            .args(cmd2_rest)
            .exec();

        std::process::exit(1);
    }

    unsafe {
        libc::close(pipe_read_fd);
        libc::close(pipe_write_fd);

        let mut status: libc::c_int = 0;
        libc::waitpid(pid1, &mut status, 0);
        libc::waitpid(pid2, &mut status, 0);
    }

    enable_raw_mode();
}

pub fn find_completions(prefix: &str) -> Vec<String> {
    let mut candidates = HashSet::new();

    for &builtin in BUILTINS.iter() {
        if builtin.starts_with(prefix) {
            candidates.insert(builtin.to_string());
        }
    }

    if let Ok(path_var) = env::var("PATH") {
        for dir in path_var.split(':') {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let filename = entry.file_name().to_string_lossy().to_string();

                        if filename.starts_with(prefix) {
                            if let Ok(metadata) = entry.metadata() {
                                if metadata.is_file()
                                    && (metadata.permissions().mode() & 0o111 != 0)
                                {
                                    candidates.insert(filename);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let mut results: Vec<String> = candidates.into_iter().collect();
    results.sort();
    results
}

pub fn get_longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let mut prefix = strings[0].clone();

    for s in &strings[1..] {
        while !s.starts_with(&prefix) {
            prefix.pop();
        }
    }
    prefix
}

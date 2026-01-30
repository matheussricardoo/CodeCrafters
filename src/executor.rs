#[allow(unused_imports)]
use std::io::{self, BufRead, BufReader, Write};

use crate::builtins::BUILTINS;
use crate::parser::{parse_input, split_by_pipe};
use crate::terminal::{disable_raw_mode, enable_raw_mode};
use std::collections::HashSet;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

pub fn execute_command_line(input: &str, history: &mut Vec<String>) -> bool {
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
        "history" => {
            if args.get(0).map(|s| s.as_str()) == Some("-r") {
                if let Some(filepath) = args.get(1) {
                    match File::open(filepath) {
                        Ok(file) => {
                            let reader = BufReader::new(file);
                            for line in reader.lines() {
                                match line {
                                    Ok(cmd) => {
                                        let trimmed = cmd.trim();
                                        if !trimmed.is_empty() {
                                            history.push(trimmed.to_string());
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Error reading history file: {}", e);
                                        break;
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("history: {}: {}", filepath, e);
                        }
                    }
                } else {
                    eprintln!("history: -r requires a filename argument");
                }
            } else if args.get(0).map(|s| s.as_str()) == Some("-w") {
                if let Some(filepath) = args.get(1) {
                    match File::create(filepath) {
                        Ok(mut file) => {
                            for cmd in history.iter() {
                                if let Err(e) = writeln!(file, "{}", cmd) {
                                    eprintln!("Error writing to history file: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("history: {}: {}", filepath, e);
                        }
                    }
                } else {
                    eprintln!("history: -w requires a filename argument");
                }
            } else {
                let n: usize = args
                    .get(0)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(history.len());
                let start = history.len().saturating_sub(n);
                for (i, cmd) in history.iter().enumerate().skip(start) {
                    println!("{:5}  {}", i + 1, cmd);
                }
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

fn is_builtin(cmd: &str) -> bool {
    BUILTINS.contains(&cmd)
}

fn execute_builtin_in_pipeline(cmd_name: &str, args: &[String]) {
    match cmd_name {
        "echo" => {
            let output = args.join(" ");
            println!("{}", output);
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
        "pwd" => {
            if let Ok(path) = std::env::current_dir() {
                println!("{}", path.display());
            }
        }
        _ => {}
    }
}

fn execute_pipeline(commands: Vec<Vec<String>>) {
    let n = commands.len();
    if n < 2 {
        return;
    }

    for cmd_args in &commands {
        if cmd_args.is_empty() {
            eprintln!("Empty command in pipeline");
            return;
        }
    }

    let mut cmd_paths: Vec<Option<PathBuf>> = Vec::new();
    for cmd_args in &commands {
        let cmd_name = &cmd_args[0];
        if is_builtin(cmd_name) {
            cmd_paths.push(None);
        } else {
            match find_executable(cmd_name) {
                Some(path) => cmd_paths.push(Some(path)),
                None => {
                    println!("{}: command not found", cmd_name);
                    return;
                }
            }
        }
    }

    let mut pipes: Vec<[libc::c_int; 2]> = Vec::new();
    for _ in 0..(n - 1) {
        let mut pipe_fds: [libc::c_int; 2] = [0; 2];
        unsafe {
            if libc::pipe(pipe_fds.as_mut_ptr()) == -1 {
                eprintln!("Failed to create pipe");
                for p in &pipes {
                    libc::close(p[0]);
                    libc::close(p[1]);
                }
                return;
            }
        }
        pipes.push(pipe_fds);
    }

    disable_raw_mode();

    let mut pids: Vec<libc::pid_t> = Vec::new();

    for i in 0..n {
        let pid = unsafe { libc::fork() };

        if pid == -1 {
            eprintln!("Failed to fork for command {}", i);
            unsafe {
                for p in &pipes {
                    libc::close(p[0]);
                    libc::close(p[1]);
                }
            }
            enable_raw_mode();
            return;
        }

        if pid == 0 {
            if i > 0 {
                unsafe {
                    libc::dup2(pipes[i - 1][0], libc::STDIN_FILENO);
                }
            }

            if i < n - 1 {
                unsafe {
                    libc::dup2(pipes[i][1], libc::STDOUT_FILENO);
                }
            }

            unsafe {
                for p in &pipes {
                    libc::close(p[0]);
                    libc::close(p[1]);
                }
            }

            let cmd_name = &commands[i][0];
            let cmd_rest = &commands[i][1..];

            if is_builtin(cmd_name) {
                execute_builtin_in_pipeline(cmd_name, cmd_rest);
                std::process::exit(0);
            } else {
                let cmd_file_name = Path::new(cmd_name).file_name().unwrap().to_str().unwrap();
                let _ = Command::new(cmd_paths[i].as_ref().unwrap())
                    .arg0(cmd_file_name)
                    .args(cmd_rest)
                    .exec();
                std::process::exit(1);
            }
        }

        pids.push(pid);
    }

    unsafe {
        for p in &pipes {
            libc::close(p[0]);
            libc::close(p[1]);
        }

        let mut status: libc::c_int = 0;
        for pid in &pids {
            libc::waitpid(*pid, &mut status, 0);
        }
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

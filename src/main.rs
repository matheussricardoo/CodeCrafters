mod builtins;
mod executor;
mod parser;
mod terminal;

use crate::executor::{execute_command_line, find_completions};
use crate::terminal::enable_raw_mode;
use std::io::{self, Read, Write};

fn main() {
    enable_raw_mode();

    print!("$ ");
    io::stdout().flush().unwrap();

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    loop {
        let mut byte_buffer = [0u8; 1];
        if handle.read_exact(&mut byte_buffer).is_err() {
            break;
        }
        let byte = byte_buffer[0];

        match byte {
            9 => {
                let matches = find_completions(&buffer);

                if matches.len() == 1 {
                    let completed = &matches[0];
                    if completed.len() >= buffer.len() {
                        let remainder = &completed[buffer.len()..];
                        print!("{} ", remainder);
                        io::stdout().flush().unwrap();
                        buffer.push_str(remainder);
                        buffer.push(' ');
                    }
                } else if matches.len() > 1 {
                    print!("\x07");
                    io::stdout().flush().unwrap();
                } else {
                    print!("\x07");
                    io::stdout().flush().unwrap();
                }
            }

            10 => {
                println!();
                if execute_command_line(&buffer) {
                    break;
                }
                buffer.clear();
                print!("$ ");
                io::stdout().flush().unwrap();
            }
            127 => {
                if !buffer.is_empty() {
                    buffer.pop();
                    print!("\x08 \x08");
                    io::stdout().flush().unwrap();
                }
            }
            c => {
                let char = c as char;
                print!("{}", char);
                io::stdout().flush().unwrap();
                buffer.push(char);
            }
        }
    }
}

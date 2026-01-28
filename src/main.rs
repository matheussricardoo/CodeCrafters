mod builtins;
mod executor;
mod parser;
mod terminal;

use crate::executor::{execute_command_line, find_completions, get_longest_common_prefix};
use crate::terminal::enable_raw_mode;
use std::io::{self, Read, Write};

fn main() {
    enable_raw_mode();

    print!("$ ");
    io::stdout().flush().unwrap();

    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    let mut tab_press_count = 0;

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
                    tab_press_count = 0;
                } else if matches.len() > 1 {
                    let lcp = get_longest_common_prefix(&matches);

                    if lcp.len() > buffer.len() {
                        let remainder = &lcp[buffer.len()..];
                        print!("{}", remainder);
                        io::stdout().flush().unwrap();
                        buffer.push_str(remainder);
                        tab_press_count = 0;
                    } else {
                        tab_press_count += 1;

                        if tab_press_count == 1 {
                            print!("\x07");
                            io::stdout().flush().unwrap();
                        } else {
                            print!("\r\n");
                            let list = matches.join("  ");
                            print!("{}", list);
                            print!("\r\n");
                            print!("$ {}", buffer);
                            io::stdout().flush().unwrap();
                        }
                    }
                } else {
                    print!("\x07");
                    io::stdout().flush().unwrap();
                    tab_press_count = 0;
                }
            }

            10 => {
                tab_press_count = 0;
                println!();
                if execute_command_line(&buffer) {
                    break;
                }
                buffer.clear();
                print!("$ ");
                io::stdout().flush().unwrap();
            }

            127 => {
                tab_press_count = 0;
                if !buffer.is_empty() {
                    buffer.pop();
                    print!("\x08 \x08");
                    io::stdout().flush().unwrap();
                }
            }

            c => {
                tab_press_count = 0;
                let char = c as char;
                print!("{}", char);
                io::stdout().flush().unwrap();
                buffer.push(char);
            }
        }
    }
}

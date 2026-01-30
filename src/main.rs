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
    let mut history: Vec<String> = Vec::new();
    let mut history_index: usize = 0;
    let mut last_saved_index: usize = 0;

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
                let trimmed = buffer.trim();
                if !trimmed.is_empty() {
                    history.push(trimmed.to_string());
                }
                history_index = history.len();
                if execute_command_line(&buffer, &mut history, &mut last_saved_index) {
                    break;
                }
                buffer.clear();
                print!("$ ");
                io::stdout().flush().unwrap();
            }

            27 => {
                let mut seq = [0u8; 2];
                if handle.read_exact(&mut seq).is_ok() {
                    if seq[0] == 91 {
                        match seq[1] {
                            65 => {
                                if !history.is_empty() && history_index > 0 {
                                    history_index -= 1;
                                    print!("\r$ ");
                                    for _ in 0..buffer.len() {
                                        print!(" ");
                                    }
                                    print!("\r$ ");
                                    buffer = history[history_index].clone();
                                    print!("{}", buffer);
                                    io::stdout().flush().unwrap();
                                }
                            }
                            66 => {
                                if !history.is_empty() && history_index < history.len() {
                                    history_index += 1;
                                    print!("\r$ ");
                                    for _ in 0..buffer.len() {
                                        print!(" ");
                                    }
                                    print!("\r$ ");
                                    if history_index < history.len() {
                                        buffer = history[history_index].clone();
                                    } else {
                                        buffer.clear();
                                    }
                                    print!("{}", buffer);
                                    io::stdout().flush().unwrap();
                                }
                            }
                            _ => {}
                        }
                    }
                }
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

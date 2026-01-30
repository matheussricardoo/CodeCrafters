pub fn parse_input(input: &str) -> Vec<String> {
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
                if c == '"' || c == '\\' {
                    current_arg.push(c);
                } else {
                    current_arg.push('\\');
                    current_arg.push(c);
                }
                escaped = false
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_double_quote = false
            } else {
                current_arg.push(c);
            }
        } else {
            if escaped {
                current_arg.push(c);
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '\'' {
                in_single_quote = true;
            } else if c == '"' {
                in_double_quote = true;
            } else if c == '|' {
                if !current_arg.is_empty() {
                    args.push(current_arg);
                    current_arg = String::new();
                }
                args.push("|".to_string());
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

pub fn split_by_pipe(args: Vec<String>) -> Vec<Vec<String>> {
    let mut commands = Vec::new();
    let mut current_cmd = Vec::new();

    for arg in args {
        if arg == "|" {
            if !current_cmd.is_empty() {
                commands.push(current_cmd);
                current_cmd = Vec::new();
            }
        } else {
            current_cmd.push(arg);
        }
    }

    if !current_cmd.is_empty() {
        commands.push(current_cmd);
    }

    commands
}

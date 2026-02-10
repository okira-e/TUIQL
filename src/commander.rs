use color_eyre::eyre::Result;
use color_eyre::eyre::bail;
use std::str::SplitWhitespace;

#[derive(Debug)]
pub enum Cmd {
    Quit,
    /// Returns the total count of the currently selected table
    Count,
    Goto(GotoCmd),
}

#[derive(Debug)]
pub enum GotoCmd {
    Page(usize),
}

pub fn parse_cmd(input: &str) -> Result<Cmd> {
    let mut iter = input.split_whitespace();

    let cmd = iter.next();
    return match cmd {
        Some(cmd) => match cmd {
            "quit" | "q" => Ok(Cmd::Quit),
            "count" | "c" => Ok(Cmd::Count),
            "goto" | "g" => parse_goto_cmd(&mut iter),
            _ => bail!("Unknown command: {}", cmd),
        },
        None => bail!("Empty command"),
    };
}

fn parse_goto_cmd(iter: &mut SplitWhitespace) -> Result<Cmd> {
    let cmd = iter.next();
    return match cmd {
        Some(cmd) => match cmd {
            "page" | "p" => parse_goto_arg_cmd(iter),
            _ => bail!("Unknown goto sub-command: {}", cmd),
        },
        None => bail!("Missing goto sub-command"),
    };
}

fn parse_goto_arg_cmd(iter: &mut SplitWhitespace) -> Result<Cmd> {
    let arg = iter.next();
    return match arg {
        Some(arg) => {
            if let Ok(page_num) = arg.parse::<usize>() {
                Ok(Cmd::Goto(GotoCmd::Page(page_num)))
            } else {
                bail!("Invalid page number: {}", arg)
            }
        }
        None => bail!("Missing page number argument"),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_command() {
        let result = parse_cmd("");
        assert!(result.is_err());
        // Empty string splits to [""], which gets matched as unknown command
        assert!(result.unwrap_err().to_string().contains("Empty command"));
    }

    #[test]
    fn test_parse_unknown_command() {
        let result = parse_cmd("invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown command"));
    }

    #[test]
    fn test_parse_without_subcommand() {
        let result = parse_cmd("goto");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing goto sub-command"));
    }

    #[test]
    fn test_parse_unknown_subcommand() {
        let result = parse_cmd("goto invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown goto sub-command"));
    }

    #[test]
    fn test_parse_goto_without_argument() {
        let result = parse_cmd("goto page");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing page number argument"));
    }

    #[test]
    fn test_parse_quit_command() {
        assert!(matches!(parse_cmd("quit").unwrap(), Cmd::Quit));
        assert!(matches!(parse_cmd("q").unwrap(), Cmd::Quit));
    }

    #[test]
    fn test_parse_count_command() {
        assert!(matches!(parse_cmd("count").unwrap(), Cmd::Count));
        assert!(matches!(parse_cmd("c").unwrap(), Cmd::Count));
    }

    #[test]
    fn test_parse_command_edge_cases() {
        match parse_cmd("goto   page  5").unwrap() {
            Cmd::Goto(GotoCmd::Page(n)) => assert_eq!(n, 5),
            _ => panic!("Expected Goto::Page command"),
        }
    }

    #[test]
    fn test_parse_goto_page_command() {
        match parse_cmd("goto page 5").unwrap() {
            Cmd::Goto(GotoCmd::Page(n)) => assert_eq!(n, 5),
            _ => panic!("Expected Goto::Page command"),
        }

        match parse_cmd("goto page 1").unwrap() {
            Cmd::Goto(GotoCmd::Page(n)) => assert_eq!(n, 1),
            _ => panic!("Expected Goto::Page command"),
        }

        match parse_cmd("goto page 999").unwrap() {
            Cmd::Goto(GotoCmd::Page(n)) => assert_eq!(n, 999),
            _ => panic!("Expected Goto::Page command"),
        }
    }

    #[test]
    fn test_parse_goto_page_invalid_number() {
        let result = parse_cmd("goto page abc");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid page number"));

        let result = parse_cmd("goto page -5");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid page number"));

        let result = parse_cmd("goto page 3.14");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid page number"));
    }
}

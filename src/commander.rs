use color_eyre::eyre::Result;
use color_eyre::eyre::bail;
use std::str::SplitWhitespace;

const MAX_LIMIT_ALLOWED: usize = 10_000;

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Quit,
    /// Returns the total count of the currently selected table
    Count,
    Goto(GotoCmd),
    Sort(Option<String>, SortCmdDirection),
    Limit(usize),
    RefreshTable,
}

#[derive(Debug, PartialEq)]
pub enum GotoCmd {
    Page(usize),
    Table(String),
}

#[derive(Debug, PartialEq)]
pub enum SortCmdDirection {
    Asc,
    Desc,
}

pub fn parse_cmd(input: &str) -> Result<Cmd> {
    let mut iter = input.split_whitespace();

    let cmd = iter.next();
    return match cmd {
        Some(cmd) => match cmd {
            "quit" | "q" => Ok(Cmd::Quit),
            "count" | "c" => Ok(Cmd::Count),
            "goto" | "g" => parse_goto_cmd(&mut iter),
            "sort" | "s" => parse_sort_cmd(&mut iter),
            "limit" | "l" => parse_limit_cmd(&mut iter),
            "refresh" | "r" => Ok(Cmd::RefreshTable),
            _ => bail!("Unknown command: {}", cmd),
        },
        None => bail!("Empty command"),
    };
}

fn parse_goto_cmd(iter: &mut SplitWhitespace) -> Result<Cmd> {
    return match iter.next() {
        Some(sub_cmd) => match sub_cmd {
            "page" | "p" => match iter.next() {
                Some(arg) => {
                    if let Ok(page_num) = arg.parse::<usize>() {
                        Ok(Cmd::Goto(GotoCmd::Page(page_num)))
                    } else {
                        bail!("Invalid page number: {}", arg)
                    }
                }
                None => bail!("Missing page number argument"),
            },
            _ => Ok(Cmd::Goto(GotoCmd::Table(sub_cmd.to_string()))),
        },
        None => bail!("Missing goto sub-command"),
    };
}

fn parse_sort_cmd(iter: &mut SplitWhitespace) -> Result<Cmd> {
    let default_sort_direction = SortCmdDirection::Asc;

    let column = iter.next();
    return match column {
        Some(column) => match iter.next() {
            Some(direction_str) => {
                let direction = match direction_str {
                    "asc" => SortCmdDirection::Asc,
                    "desc" => SortCmdDirection::Desc,
                    _ => bail!("Sort directions: asc, desc"),
                };

                Ok(Cmd::Sort(Some(column.to_string()), direction))
            }
            None => Ok(Cmd::Sort(Some(column.to_string()), default_sort_direction)),
        },
        None => Ok(Cmd::Sort(None, SortCmdDirection::Asc)),
    };
}

fn parse_limit_cmd(iter: &mut SplitWhitespace) -> Result<Cmd> {
    return match iter.next() {
        Some(arg) => {
            let limit = if let Ok(l) = arg.parse::<usize>() {
                if l == 0 {
                    bail!("Limit must be greater than 0");
                }

                l
            } else {
                match parse_metric(arg) {
                    Some(l) => l,
                    None => bail!("Invalid limit: {}", arg),
                }
            };

            if limit > MAX_LIMIT_ALLOWED {
                bail!("Limit is too big!");
            }

            Ok(Cmd::Limit(limit))
        }
        None => bail!("Limit command requires a number as an argument"),
    };
}

// helper function
fn parse_metric(input: &str) -> Option<usize> {
    let (num, suffix) = input
        .trim()
        .split_at(input.find(|c: char| !c.is_ascii_digit()).unwrap_or(input.len()));

    let n: f64 = num.parse().ok()?;

    let multiplier = match suffix.to_ascii_lowercase().as_str() {
        "" => 1.0,
        "k" => 1e3,
        _ => return None,
    };

    Some((n * multiplier) as usize)
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
    fn test_parse_sort_command() {
        let result = parse_cmd("sort id desc");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Cmd::Sort(Some(String::from("id")), SortCmdDirection::Desc)
        );
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

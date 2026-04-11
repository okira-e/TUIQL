use color_eyre::eyre::Result;
use color_eyre::eyre::bail;
use std::str::SplitWhitespace;

const MAX_LIMIT_ALLOWED: usize = 10_000;

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Quit,
    /// Returns the total count of the currently selected table
    Count,
    TotalCount,
    Goto(GotoCmd),
    OrderBy(Option<String>),
    Where(Option<String>),
    Limit(usize),
    RefreshTable,
    Set(String, Option<String>),
    OpenHelp,
}

#[derive(Debug, PartialEq)]
pub enum GotoCmd {
    Page(usize),
    Table(String),
}

pub fn parse_cmd(input: &str) -> Result<Cmd> {
    let mut iter = input.split_whitespace();

    let cmd = iter.next();
    return match cmd {
        Some(cmd) => match cmd {
            "q" | "quit" => Ok(Cmd::Quit),
            "c" | "count" => Ok(Cmd::Count),
            "tc" | "total-count" => Ok(Cmd::TotalCount),
            "g" | "goto" => parse_goto_cmd(&mut iter),
            "ob" | "order-by" => parse_order_by_cmd(input),
            "w" | "where" => parse_where_cmd(input),
            "l" | "limit" => parse_limit_cmd(&mut iter),
            "r" | "refresh" => Ok(Cmd::RefreshTable),
            "set" => parse_set_cmd(&mut iter),
            "help" | "h" => Ok(Cmd::OpenHelp),
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

fn parse_order_by_cmd(input: &str) -> Result<Cmd> {
    let clause = input
        .trim()
        .strip_prefix("order-by")
        .or_else(|| input.trim().strip_prefix("ob"))
        .unwrap_or("")
        .trim();

    if clause.is_empty() {
        return Ok(Cmd::OrderBy(None));
    }

    return Ok(Cmd::OrderBy(Some(clause.to_string())));
}

fn parse_where_cmd(input: &str) -> Result<Cmd> {
    let clause = input
        .trim()
        .strip_prefix("where")
        .or_else(|| input.trim().strip_prefix("w"))
        .unwrap_or("")
        .trim();

    if clause.is_empty() {
        return Ok(Cmd::Where(None));
    }

    return Ok(Cmd::Where(Some(clause.to_string())));
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

fn parse_set_cmd(iter: &mut SplitWhitespace) -> Result<Cmd> {
    return match iter.next() {
        Some(key) => {
            let value: Option<String> = iter.next().map(String::from);

            Ok(Cmd::Set(key.to_string(), value))
        }
        None => bail!("Set command needs a key at least"),
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

    return Some((n * multiplier) as usize);
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
    fn test_parse_goto_table_name() {
        let result = parse_cmd("goto users");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Cmd::Goto(GotoCmd::Table(String::from("users")))
        );
    }

    #[test]
    fn test_parse_goto_without_argument() {
        let result = parse_cmd("goto page");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Missing page number argument"));
    }

    #[test]
    fn test_parse_order_by_command() {
        let result = parse_cmd("order-by id desc");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Cmd::OrderBy(Some(String::from("id desc"))));

        let result = parse_cmd("ob created_at desc, id asc");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Cmd::OrderBy(Some(String::from("created_at desc, id asc")))
        );
    }

    #[test]
    fn test_parse_order_by_missing_clause() {
        let result = parse_cmd("order-by");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("order-by command requires a clause")
        );

        let result = parse_cmd("ob");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_where_command() {
        let result = parse_cmd("where status = 'active'");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Cmd::Where(Some(String::from("status = 'active'")))
        );

        let result = parse_cmd("w id > 10 AND name LIKE '%test%'");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            Cmd::Where(Some(String::from("id > 10 AND name LIKE '%test%'")))
        );
    }

    #[test]
    fn test_parse_where_missing_clause() {
        let result = parse_cmd("where");
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("where command requires a clause")
        );

        let result = parse_cmd("w");
        assert!(result.is_err());
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

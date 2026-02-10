use color_eyre::eyre::Result;
use color_eyre::eyre::bail;
use std::str::Split;

#[derive(Debug)]
pub enum Cmd {
    /// Returns the total count of the currently selected table
    Count,
    Goto(GotoCmd),
}

#[derive(Debug)]
pub enum GotoCmd {
    Page(usize),
}

pub fn parse_cmd(cmd: &str) -> Result<Cmd> {
    let mut iter = cmd.split(" ");

    let cmd = iter.next();
    return match cmd {
        Some(cmd) => match cmd {
            "count" | "c" => Ok(Cmd::Count),
            "goto" | "g" => parse_goto_cmd(&mut iter),
            _ => bail!("Unknown command: {}", cmd),
        },
        None => bail!("Empty command"),
    };
}

fn parse_goto_cmd(iter: &mut Split<&str>) -> Result<Cmd> {
    let cmd = iter.next();
    return match cmd {
        Some(cmd) => match cmd {
            "page" | "p" => parse_goto_arg_cmd(iter),
            _ => bail!("Unknown goto sub-command: {}", cmd),
        },
        None => bail!("Missing goto sub-command"),
    };
}

fn parse_goto_arg_cmd(iter: &mut Split<&str>) -> Result<Cmd> {
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

use crate::suggestor::Suggestion;
use std::ops::Range;

pub struct StatusLineModel {
    pub mode: StatusLineMode,
    pub msg: StatusLineMsg,
    pub cmd: StatusLineCommand,
    pub spinner_animation_tick_count: usize,
    pub history_cursor: usize,
    pub completion: Completion,
}

impl StatusLineModel {
    /// Resets the status line state that's related to the mode you give it.
    pub fn reset(&mut self) {
        match self.mode {
            StatusLineMode::Status => *self = Self::default(),
            StatusLineMode::Command => {
                self.cmd.cursor = 0;
                self.cmd.text = String::new();
                self.history_cursor = 0;
                self.completion = Completion::default();
            }
        }
    }

    pub fn cycle_completion(&mut self, going_forward: bool) {
        let count = self.completion.candidates.len();
        if count == 0 {
            return;
        }

        // First press freezes the span of text we're completing.
        if self.completion.selected.is_none() {
            self.completion.anchor = completion_anchor(&self.cmd.text, self.cmd.cursor);
        }

        let next = match self.completion.selected {
            None => {
                if going_forward {
                    0
                } else {
                    count - 1
                }
            }
            Some(i) => {
                if going_forward {
                    (i + 1) % count
                } else {
                    (i + count - 1) % count
                }
            }
        };

        self.completion.selected = Some(next);

        let candidate = self.completion.candidates[next].display.clone();
        let anchor = self.completion.anchor.clone();
        self.cmd.text.replace_range(anchor.clone(), &candidate);
        let end = anchor.start + candidate.len();
        self.cmd.cursor = end;
        self.completion.anchor = anchor.start..end;
    }
}

impl Default for StatusLineModel {
    fn default() -> Self {
        return Self {
            mode: StatusLineMode::Status,
            msg: StatusLineMsg::default(),
            cmd: StatusLineCommand::default(),
            spinner_animation_tick_count: 0,
            history_cursor: 0,
            completion: Completion::default(),
        };
    }
}

/// Tab-completion state for the command line. `candidates` is refreshed live on
/// every keystroke while `selected` is `None`. The first Tab freezes the list and
/// the `anchor` — the byte range of the token being completed — so subsequent Tabs
/// overwrite that same span instead of appending.
pub struct Completion {
    pub candidates: Vec<Suggestion>,
    pub selected: Option<usize>,
    anchor: Range<usize>,
}

impl Default for Completion {
    fn default() -> Self {
        return Self { candidates: Vec::new(), selected: None, anchor: 0..0 };
    }
}

/// Gives back a byte range from the last space to the cursor/index given.
fn completion_anchor(text: &str, cursor: usize) -> Range<usize> {
    let start = text[..cursor]
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
        .map(|(i, c)| i + c.len_utf8())
        .unwrap_or(0);

    return start..cursor;
}

#[derive(Debug, Clone)]
pub enum MsgKind {
    Error,
    Success,
    Neutral,
}

#[derive(Debug, Clone)]
pub enum MsgLifetime {
    Forever,
    Short,
    Long,
}

impl MsgLifetime {
    pub fn to_duration(&self) -> std::time::Duration {
        match self {
            MsgLifetime::Forever => std::time::Duration::MAX,
            MsgLifetime::Short => std::time::Duration::from_secs(3),
            MsgLifetime::Long => std::time::Duration::from_secs(8),
        }
    }
}

#[derive(Clone)]
pub struct StatusLineMsg {
    pub text: String,
    pub kind: MsgKind,
    pub lifetime: MsgLifetime,
    pub created_at: std::time::Instant,
}

impl Default for StatusLineMsg {
    fn default() -> Self {
        return Self {
            text: String::from("Press ? for help"),
            kind: MsgKind::Neutral,
            lifetime: MsgLifetime::Forever,
            created_at: std::time::Instant::now(),
        };
    }
}

pub struct StatusLineCommand {
    pub text: String,
    pub cursor: usize,
}

impl Default for StatusLineCommand {
    fn default() -> Self {
        return Self { text: String::new(), cursor: 0 };
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StatusLineMode {
    Status,
    Command,
}

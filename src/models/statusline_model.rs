pub enum StatusLineMsgKind {
    Error,
    Success,
    Neutral,
}

pub struct StatusLineMsg {
    pub text: String,
    pub kind: StatusLineMsgKind
}

pub struct StatusLineCommand {}

pub enum StatusLineMode {
    Status(Option<StatusLineMsg>),
    Command(StatusLineCommand),
}

pub struct StatusLineModel {
    pub mode: StatusLineMode,
}

impl StatusLineModel {
    pub fn new() -> Self {
        return Self {
            mode: StatusLineMode::Status(None)
            // mode: StatusLineMode::Status(Some(StatusLineMsg { text: "hi there hello".to_string(), kind: StatusLineMsgKind::Neutral }))
        }
    }

    pub fn report_message(&mut self, text: impl Into<String>, kind: StatusLineMsgKind) {
        self.mode = StatusLineMode::Status(Some(StatusLineMsg {
            text: text.into(),
            kind,
        }));
    }

    pub fn clear(&mut self) {
        self.mode = StatusLineMode::Status(None);
    }
}
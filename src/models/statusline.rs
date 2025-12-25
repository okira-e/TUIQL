pub struct StatusLineModel {
    pub mode: StatusLineMode,
    pub msg: StatusLineMsg,
    pub is_loading: bool,
    pub spinner_animation_tick_count: usize,
}

impl StatusLineModel {
    pub fn new() -> Self {
        return Self {
            mode: StatusLineMode::Status,
            msg: StatusLineMsg::default(),
            is_loading: false,
            spinner_animation_tick_count: 0,
        };
    }
}

#[derive(Clone)]
pub enum MsgKind {
    Error,
    Success,
    Neutral,
}

#[derive(Clone)]
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
        Self {
            text: String::from("Press ? for help"),
            kind: MsgKind::Neutral,
            lifetime: MsgLifetime::Forever,
            created_at: std::time::Instant::now(),
        }
    }
}

pub struct StatusLineCommand {}

pub enum StatusLineMode {
    Status,
    Command(StatusLineCommand),
}

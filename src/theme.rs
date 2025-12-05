use ratatui::style::{Color, Style};

#[derive(Clone)]
pub struct Theme {
    pub bg: Style,
    pub fg: Style,
    pub accent: Style,
    pub selection: Style,
    pub error: Style,
    pub success: Style,
    pub pane_focus: Style,
}

impl Theme {
    pub fn catppuccin(flavor: Flavor) -> Self {
        let p = match flavor {
            Flavor::Latte => LATTE,
            Flavor::Mocha => MOCHA,
        };

        Self {
            bg: Style::new().bg(p.base),
            fg: Style::new().fg(p.text),
            accent:     Style::new().fg(p.flamingo),
            selection:  Style::new().fg(p.green),
            error:      Style::new().fg(p.red),
            success:    Style::new().fg(p.green),
            pane_focus: Style::new().fg(p.blue),
        }
    }
}

pub enum Flavor {
    Latte,
    Mocha,
}

pub struct Palette {
    pub base: Color,
    pub text: Color,
    pub blue: Color,
    pub green: Color,
    pub mauve: Color,
    pub flamingo: Color,
    pub red: Color,
}

pub const LATTE: Palette = Palette {
    base: Color::Rgb(239, 241, 245),
    text: Color::Rgb(76, 79, 105),
    blue: Color::Rgb(30, 102, 245),
    green: Color::Rgb(64, 160, 43),
    mauve: Color::Rgb(136, 57, 239),
    flamingo: Color::Rgb(221, 120, 120),
    red: Color::Rgb(210, 15, 57),
};

pub const MOCHA: Palette = Palette {
    base: Color::Rgb(30, 30, 46),
    text: Color::Rgb(205, 214, 244),
    blue: Color::Rgb(137, 180, 250),
    green: Color::Rgb(166, 227, 161),
    mauve: Color::Rgb(203, 166, 247),
    flamingo: Color::Rgb(242, 205, 205),
    red: Color::Rgb(243, 139, 168),
};

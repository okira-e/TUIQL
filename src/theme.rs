use ratatui::style::Color;

#[derive(Clone)]
pub struct Theme {
    pub bg: Color,
    pub fg: Color,
    pub accent: Color,
    pub selection: Color,
    pub error: Color,
    pub success: Color,
    pub pane_focus: Color,
    pub highlight: Color,
}

impl Theme {
    pub fn catppuccin(flavor: Flavor) -> Self {
        let p = match flavor {
            Flavor::Latte => LATTE,
            Flavor::Mocha => MOCHA,
        };

        Self {
            bg: p.base,
            fg: p.text,
            accent: p.flamingo,
            selection: p.green,
            error: p.red,
            success: p.green,
            pane_focus: p.blue,
            highlight: p.highlight,
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
    pub highlight: Color,
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
    highlight: Color::Rgb(230, 233, 239),
};

pub const MOCHA: Palette = Palette {
    base: Color::Rgb(30, 30, 46),
    text: Color::Rgb(205, 214, 244),
    blue: Color::Rgb(137, 180, 250),
    green: Color::Rgb(166, 227, 161),
    mauve: Color::Rgb(203, 166, 247),
    flamingo: Color::Rgb(242, 205, 205),
    red: Color::Rgb(243, 139, 168),
    highlight: Color::Rgb(49, 50, 68),
};

use color_eyre::Result;
use ratatui::style::Color;
use std::str::FromStr;

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

impl FromStr for Theme {
    type Err = color_eyre::eyre::Error;

    fn from_str(s: &str) -> Result<Self> {
        let name = s.trim().to_ascii_lowercase().replace('_', "-").replace(' ', "-");

        match name.as_str() {
            "catppuccin-mocha" | "mocha" => Ok(Self::catppuccin_mocha()),
            "catppuccin-macchiato" | "macchiato" => Ok(Self::catppuccin_macchiato()),
            "catppuccin-frappe" | "frappe" => Ok(Self::catppuccin_frappe()),
            "catppuccin-latte" | "latte" => Ok(Self::catppuccin_latte()),
            "dracula" => Ok(Self::dracula()),
            "nord" => Ok(Self::nord()),
            "gruvbox-dark" => Ok(Self::gruvbox_dark()),
            "gruvbox-light" => Ok(Self::gruvbox_light()),
            "tokyo-night" => Ok(Self::tokyo_night()),
            "tokyo-night-storm" => Ok(Self::tokyo_night_storm()),
            "one-dark" => Ok(Self::one_dark()),
            "solarized-dark" => Ok(Self::solarized_dark()),
            "solarized-light" => Ok(Self::solarized_light()),
            "monokai" => Ok(Self::monokai()),
            "kanagawa-wave" | "kanagawa" => Ok(Self::kanagawa_wave()),
            "rose-pine" => Ok(Self::rose_pine()),
            "rose-pine-dawn" => Ok(Self::rose_pine_dawn()),
            "everforest-dark" | "everforest" => Ok(Self::everforest_dark()),
            "ayu-dark" => Ok(Self::ayu_dark()),
            "ayu-mirage" => Ok(Self::ayu_mirage()),
            "ayu-light" => Ok(Self::ayu_light()),
            "github-dark" => Ok(Self::github_dark()),
            "github-light" => Ok(Self::github_light()),
            _ => color_eyre::eyre::bail!("Unknown theme: {s}"),
        }
    }
}

impl Theme {
    pub const fn options() -> &'static [&'static str] {
        return &[
            "catppuccin-mocha",
            "catppuccin-macchiato",
            "catppuccin-frappe",
            "catppuccin-latte",
            "dracula",
            "nord",
            "gruvbox-dark",
            "gruvbox-light",
            "tokyo-night",
            "tokyo-night-storm",
            "one-dark",
            "solarized-dark",
            "solarized-light",
            "monokai",
            "kanagawa-wave",
            "rose-pine",
            "rose-pine-dawn",
            "everforest-dark",
            "ayu-dark",
            "ayu-mirage",
            "ayu-light",
            "github-dark",
            "github-light",
        ];
    }

    const fn from_colors(colors: [Color; 8]) -> Self {
        Self {
            bg: colors[0],
            fg: colors[1],
            accent: colors[2],
            selection: colors[3],
            error: colors[4],
            success: colors[5],
            pane_focus: colors[6],
            highlight: colors[7],
        }
    }

    pub const fn catppuccin_mocha() -> Self {
        Self::from_colors([
            Color::Rgb(30, 30, 46),
            Color::Rgb(205, 214, 244),
            Color::Rgb(242, 205, 205),
            Color::Rgb(166, 227, 161),
            Color::Rgb(243, 139, 168),
            Color::Rgb(166, 227, 161),
            Color::Rgb(137, 180, 250),
            Color::Rgb(49, 50, 68),
        ])
    }

    pub const fn catppuccin_macchiato() -> Self {
        Self::from_colors([
            Color::Rgb(36, 39, 58),
            Color::Rgb(202, 211, 245),
            Color::Rgb(240, 198, 198),
            Color::Rgb(166, 218, 149),
            Color::Rgb(237, 135, 150),
            Color::Rgb(166, 218, 149),
            Color::Rgb(138, 173, 244),
            Color::Rgb(54, 58, 79),
        ])
    }

    pub const fn catppuccin_frappe() -> Self {
        Self::from_colors([
            Color::Rgb(48, 52, 70),
            Color::Rgb(198, 208, 245),
            Color::Rgb(238, 190, 190),
            Color::Rgb(166, 209, 137),
            Color::Rgb(231, 130, 132),
            Color::Rgb(166, 209, 137),
            Color::Rgb(140, 170, 238),
            Color::Rgb(65, 69, 89),
        ])
    }

    pub const fn catppuccin_latte() -> Self {
        Self::from_colors([
            Color::Rgb(239, 241, 245),
            Color::Rgb(76, 79, 105),
            Color::Rgb(221, 120, 120),
            Color::Rgb(64, 160, 43),
            Color::Rgb(210, 15, 57),
            Color::Rgb(64, 160, 43),
            Color::Rgb(30, 102, 245),
            Color::Rgb(230, 233, 239),
        ])
    }

    pub const fn dracula() -> Self {
        Self::from_colors([
            Color::Rgb(40, 42, 54),
            Color::Rgb(248, 248, 242),
            Color::Rgb(255, 121, 198),
            Color::Rgb(80, 250, 123),
            Color::Rgb(255, 85, 85),
            Color::Rgb(80, 250, 123),
            Color::Rgb(139, 233, 253),
            Color::Rgb(68, 71, 90),
        ])
    }

    pub const fn nord() -> Self {
        Self::from_colors([
            Color::Rgb(46, 52, 64),
            Color::Rgb(216, 222, 233),
            Color::Rgb(180, 142, 173),
            Color::Rgb(163, 190, 140),
            Color::Rgb(191, 97, 106),
            Color::Rgb(163, 190, 140),
            Color::Rgb(136, 192, 208),
            Color::Rgb(59, 66, 82),
        ])
    }

    pub const fn gruvbox_dark() -> Self {
        Self::from_colors([
            Color::Rgb(40, 40, 40),
            Color::Rgb(235, 219, 178),
            Color::Rgb(211, 134, 155),
            Color::Rgb(184, 187, 38),
            Color::Rgb(251, 73, 52),
            Color::Rgb(184, 187, 38),
            Color::Rgb(131, 165, 152),
            Color::Rgb(60, 56, 54),
        ])
    }

    pub const fn gruvbox_light() -> Self {
        Self::from_colors([
            Color::Rgb(251, 241, 199),
            Color::Rgb(60, 56, 54),
            Color::Rgb(143, 63, 113),
            Color::Rgb(121, 116, 14),
            Color::Rgb(204, 36, 29),
            Color::Rgb(121, 116, 14),
            Color::Rgb(69, 133, 136),
            Color::Rgb(235, 219, 178),
        ])
    }

    pub const fn tokyo_night() -> Self {
        Self::from_colors([
            Color::Rgb(26, 27, 38),
            Color::Rgb(169, 177, 214),
            Color::Rgb(187, 154, 247),
            Color::Rgb(158, 206, 106),
            Color::Rgb(247, 118, 142),
            Color::Rgb(158, 206, 106),
            Color::Rgb(122, 162, 247),
            Color::Rgb(41, 46, 66),
        ])
    }

    pub const fn tokyo_night_storm() -> Self {
        Self::from_colors([
            Color::Rgb(36, 40, 59),
            Color::Rgb(169, 177, 214),
            Color::Rgb(187, 154, 247),
            Color::Rgb(158, 206, 106),
            Color::Rgb(247, 118, 142),
            Color::Rgb(158, 206, 106),
            Color::Rgb(122, 162, 247),
            Color::Rgb(59, 66, 97),
        ])
    }

    pub const fn one_dark() -> Self {
        Self::from_colors([
            Color::Rgb(40, 44, 52),
            Color::Rgb(171, 178, 191),
            Color::Rgb(198, 120, 221),
            Color::Rgb(152, 195, 121),
            Color::Rgb(224, 108, 117),
            Color::Rgb(152, 195, 121),
            Color::Rgb(97, 175, 239),
            Color::Rgb(62, 68, 81),
        ])
    }

    pub const fn solarized_dark() -> Self {
        Self::from_colors([
            Color::Rgb(0, 43, 54),
            Color::Rgb(131, 148, 150),
            Color::Rgb(211, 54, 130),
            Color::Rgb(133, 153, 0),
            Color::Rgb(220, 50, 47),
            Color::Rgb(133, 153, 0),
            Color::Rgb(38, 139, 210),
            Color::Rgb(7, 54, 66),
        ])
    }

    pub const fn solarized_light() -> Self {
        Self::from_colors([
            Color::Rgb(253, 246, 227),
            Color::Rgb(101, 123, 131),
            Color::Rgb(211, 54, 130),
            Color::Rgb(133, 153, 0),
            Color::Rgb(220, 50, 47),
            Color::Rgb(133, 153, 0),
            Color::Rgb(38, 139, 210),
            Color::Rgb(238, 232, 213),
        ])
    }

    pub const fn monokai() -> Self {
        Self::from_colors([
            Color::Rgb(39, 40, 34),
            Color::Rgb(248, 248, 242),
            Color::Rgb(249, 38, 114),
            Color::Rgb(166, 226, 46),
            Color::Rgb(249, 38, 114),
            Color::Rgb(166, 226, 46),
            Color::Rgb(102, 217, 239),
            Color::Rgb(73, 72, 62),
        ])
    }

    pub const fn kanagawa_wave() -> Self {
        Self::from_colors([
            Color::Rgb(31, 31, 40),
            Color::Rgb(220, 215, 186),
            Color::Rgb(210, 126, 153),
            Color::Rgb(152, 187, 108),
            Color::Rgb(195, 64, 67),
            Color::Rgb(152, 187, 108),
            Color::Rgb(126, 156, 216),
            Color::Rgb(42, 42, 55),
        ])
    }

    pub const fn rose_pine() -> Self {
        Self::from_colors([
            Color::Rgb(25, 23, 36),
            Color::Rgb(224, 222, 244),
            Color::Rgb(235, 188, 186),
            Color::Rgb(156, 207, 216),
            Color::Rgb(235, 111, 146),
            Color::Rgb(49, 116, 143),
            Color::Rgb(196, 167, 231),
            Color::Rgb(38, 35, 58),
        ])
    }

    pub const fn rose_pine_dawn() -> Self {
        Self::from_colors([
            Color::Rgb(250, 244, 237),
            Color::Rgb(87, 82, 121),
            Color::Rgb(215, 130, 126),
            Color::Rgb(40, 105, 131),
            Color::Rgb(180, 99, 122),
            Color::Rgb(40, 105, 131),
            Color::Rgb(144, 122, 169),
            Color::Rgb(242, 233, 222),
        ])
    }

    pub const fn everforest_dark() -> Self {
        Self::from_colors([
            Color::Rgb(39, 46, 51),
            Color::Rgb(211, 198, 170),
            Color::Rgb(214, 153, 182),
            Color::Rgb(167, 192, 128),
            Color::Rgb(230, 126, 128),
            Color::Rgb(167, 192, 128),
            Color::Rgb(127, 187, 179),
            Color::Rgb(52, 63, 68),
        ])
    }

    pub const fn ayu_dark() -> Self {
        Self::from_colors([
            Color::Rgb(11, 14, 20),
            Color::Rgb(191, 189, 182),
            Color::Rgb(255, 180, 84),
            Color::Rgb(170, 217, 76),
            Color::Rgb(240, 113, 120),
            Color::Rgb(170, 217, 76),
            Color::Rgb(89, 194, 255),
            Color::Rgb(20, 25, 33),
        ])
    }

    pub const fn ayu_mirage() -> Self {
        Self::from_colors([
            Color::Rgb(31, 36, 48),
            Color::Rgb(203, 204, 198),
            Color::Rgb(255, 204, 102),
            Color::Rgb(186, 230, 126),
            Color::Rgb(242, 108, 133),
            Color::Rgb(186, 230, 126),
            Color::Rgb(115, 190, 252),
            Color::Rgb(43, 50, 66),
        ])
    }

    pub const fn ayu_light() -> Self {
        Self::from_colors([
            Color::Rgb(250, 250, 250),
            Color::Rgb(92, 103, 115),
            Color::Rgb(250, 141, 40),
            Color::Rgb(134, 179, 0),
            Color::Rgb(240, 71, 71),
            Color::Rgb(134, 179, 0),
            Color::Rgb(57, 124, 184),
            Color::Rgb(240, 240, 240),
        ])
    }

    pub const fn github_dark() -> Self {
        Self::from_colors([
            Color::Rgb(13, 17, 23),
            Color::Rgb(201, 209, 217),
            Color::Rgb(188, 140, 255),
            Color::Rgb(63, 185, 80),
            Color::Rgb(248, 81, 73),
            Color::Rgb(63, 185, 80),
            Color::Rgb(88, 166, 255),
            Color::Rgb(22, 27, 34),
        ])
    }

    pub const fn github_light() -> Self {
        Self::from_colors([
            Color::Rgb(255, 255, 255),
            Color::Rgb(31, 35, 40),
            Color::Rgb(130, 80, 223),
            Color::Rgb(31, 136, 61),
            Color::Rgb(207, 34, 46),
            Color::Rgb(31, 136, 61),
            Color::Rgb(9, 105, 218),
            Color::Rgb(246, 248, 250),
        ])
    }
}

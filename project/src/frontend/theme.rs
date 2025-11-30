// --- File: src/theme.rs ---
// --- Purpose: Defines color palettes (Dark, Light, Nordic, Gruvbox, Catppuccin) and styling logic ---

use ratatui::style::{Color, Modifier, Style};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThemeType {
    Dark,
    Light,
    Nordic,
    Gruvbox,
    Catppuccin,
}

pub struct Theme {
    pub variant: ThemeType,
    pub root: Style,
    pub focused_border: Style,
    pub normal_border: Style,
    pub text_highlight: Style,
    pub text_normal: Style,
    pub sidebar_selected: Style,
    pub sidebar_normal: Style,
    pub gauge_color: Color,
}

impl Theme {
    pub fn new(variant: ThemeType) -> Self {
        match variant {
            ThemeType::Dark => Self {
                variant,
                root: Style::default().bg(Color::Reset).fg(Color::White),
                focused_border: Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
                normal_border: Style::default().fg(Color::DarkGray),
                text_highlight: Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                text_normal: Style::default().fg(Color::Gray),
                sidebar_selected: Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD),
                sidebar_normal: Style::default().fg(Color::Gray),
                gauge_color: Color::Magenta,
            },
            ThemeType::Light => Self {
                variant,
                root: Style::default().bg(Color::White).fg(Color::Black),
                focused_border: Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
                normal_border: Style::default().fg(Color::Gray),
                text_highlight: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
                text_normal: Style::default().fg(Color::Black),
                sidebar_selected: Style::default().fg(Color::White).bg(Color::Blue).add_modifier(Modifier::BOLD),
                sidebar_normal: Style::default().fg(Color::Black),
                gauge_color: Color::Blue,
            },
            ThemeType::Nordic => Self {
                variant,
                // Polar Night: #2E3440, Snow Storm: #D8DEE9, Frost: #88C0D0, Aurora Red: #BF616A
                root: Style::default().bg(Color::Rgb(46, 52, 64)).fg(Color::Rgb(216, 222, 233)),
                focused_border: Style::default().fg(Color::Rgb(136, 192, 208)).add_modifier(Modifier::BOLD),
                normal_border: Style::default().fg(Color::Rgb(76, 86, 106)),
                text_highlight: Style::default().fg(Color::Rgb(235, 203, 139)).add_modifier(Modifier::BOLD),
                text_normal: Style::default().fg(Color::Rgb(216, 222, 233)),
                sidebar_selected: Style::default().fg(Color::Rgb(46, 52, 64)).bg(Color::Rgb(136, 192, 208)).add_modifier(Modifier::BOLD),
                sidebar_normal: Style::default().fg(Color::Rgb(216, 222, 233)),
                gauge_color: Color::Rgb(136, 192, 208),
            },
            ThemeType::Gruvbox => Self {
                variant,
                // Dark: #282828, Light: #ebdbb2, Accent: #d79921 (Yellow) or #fe8019 (Orange)
                root: Style::default().bg(Color::Rgb(40, 40, 40)).fg(Color::Rgb(235, 219, 178)),
                focused_border: Style::default().fg(Color::Rgb(254, 128, 25)).add_modifier(Modifier::BOLD), // Orange
                normal_border: Style::default().fg(Color::Rgb(146, 131, 116)), // Gray
                text_highlight: Style::default().fg(Color::Rgb(250, 189, 47)).add_modifier(Modifier::BOLD), // Yellow
                text_normal: Style::default().fg(Color::Rgb(235, 219, 178)),
                sidebar_selected: Style::default().fg(Color::Rgb(40, 40, 40)).bg(Color::Rgb(254, 128, 25)).add_modifier(Modifier::BOLD),
                sidebar_normal: Style::default().fg(Color::Rgb(235, 219, 178)),
                gauge_color: Color::Rgb(250, 189, 47),
            },
            ThemeType::Catppuccin => Self {
                variant,
                // Mocha Flavor
                // Base: #1e1e2e, Text: #cdd6f4, Mauve: #cba6f7, Overlay0: #6c7086, Yellow: #f9e2af, Green: #a6e3a1
                root: Style::default().bg(Color::Rgb(30, 30, 46)).fg(Color::Rgb(205, 214, 244)),
                focused_border: Style::default().fg(Color::Rgb(203, 166, 247)).add_modifier(Modifier::BOLD), // Mauve
                normal_border: Style::default().fg(Color::Rgb(108, 112, 134)), // Overlay0
                text_highlight: Style::default().fg(Color::Rgb(249, 226, 175)).add_modifier(Modifier::BOLD), // Yellow
                text_normal: Style::default().fg(Color::Rgb(205, 214, 244)), // Text
                sidebar_selected: Style::default().fg(Color::Rgb(30, 30, 46)).bg(Color::Rgb(166, 227, 161)).add_modifier(Modifier::BOLD), // Green
                sidebar_normal: Style::default().fg(Color::Rgb(205, 214, 244)),
                gauge_color: Color::Rgb(203, 166, 247), // Mauve
            },
        }
    }
}
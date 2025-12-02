// --- File: src/frontend/overlays/theme_selector.rs ---
// --- Purpose: Popup list to select a theme ---

use ratatui::{prelude::*, widgets::*};
use crate::App;
use crate::frontend::theme::ThemeType;

pub const AVAILABLE_THEMES: [(ThemeType, &str); 5] = [
    (ThemeType::Dark, "Dark"),
    (ThemeType::Light, "Light"),
    (ThemeType::Nordic, "Nordic"),
    (ThemeType::Gruvbox, "Gruvbox"),
    (ThemeType::Catppuccin, "Catppuccin"),
];

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let area = crate::frontend::overlays::help::centered_rect(30, 30, area);
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = AVAILABLE_THEMES
        .iter()
        .enumerate()
        .map(|(i, (variant, label))| {
            // Highlight if selected OR if it's the currently active theme
            let is_selected = i == app.theme_selector_index;
            let is_active = *variant == app.theme.variant;

            let style = if is_selected {
                app.theme.sidebar_selected
            } else {
                app.theme.text_normal
            };

            let prefix = if is_active { "*" } else { " " };
            ListItem::new(format!("{} {}", prefix, label)).style(style)
        })
        .collect();

    let block = Block::default()
        .title(" Select Theme ")
        .borders(Borders::ALL)
        .border_style(app.theme.focused_border)
        .style(app.theme.root);

    let list = List::new(items)
        .block(block);

    f.render_widget(list, area);
}
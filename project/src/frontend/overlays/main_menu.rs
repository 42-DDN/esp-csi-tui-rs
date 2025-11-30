// --- File: src/frontend/overlays/main_menu.rs ---
// --- Purpose: Main application menu (Theme, Export, etc.) ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub const MENU_ITEMS: [&str; 3] = [
    "Change Theme",
    "Export Data (RRD/CSV) [TODO]",
    "Close Menu"
];

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let area = crate::frontend::overlays::help::centered_rect(40, 30, area);
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = MENU_ITEMS
        .iter()
        .enumerate()
        .map(|(i, &label)| {
            let style = if i == app.main_menu_index {
                Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!(" {} ", label)).style(style)
        })
        .collect();

    let block = Block::default()
        .title(" Main Menu ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let list = List::new(items).block(block);

    f.render_widget(list, area);
}
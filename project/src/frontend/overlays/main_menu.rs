// --- File: src/frontend/overlays/main_menu.rs ---
// --- Purpose: Main application menu (Theme, Export, etc.) ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub const MENU_ITEMS: [&str; 5] = [
    "Change Theme",
    "Save Template",
    "Load Template",
    "Export Data [TODO]",
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
                app.theme.sidebar_selected
            } else {
                app.theme.text_normal
            };

            // Display current theme next to the "Change Theme" option
            let display_label = if i == 0 {
                format!(" {} ({:?}) ", label, app.theme.variant)
            } else {
                format!(" {} ", label)
            };

            ListItem::new(display_label).style(style)
        })
        .collect();

    let block = Block::default()
        .title(" Main Menu ")
        .borders(Borders::ALL)
        .border_style(app.theme.focused_border)
        .style(app.theme.root);

    let list = List::new(items).block(block);

    f.render_widget(list, area);
}
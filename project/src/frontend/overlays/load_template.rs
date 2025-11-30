// --- File: src/frontend/overlays/load_template.rs ---
// --- Purpose: Popup list to select a template to load ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let area = crate::frontend::overlays::help::centered_rect(40, 40, area);
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = app.available_templates
        .iter()
        .enumerate()
        .map(|(i, (name, is_default))| {
            let style = if i == app.load_selector_index {
                Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Format name with * for default
            let display_name = name.strip_suffix(".json").unwrap_or(name);
            let label = if *is_default {
                format!(" {} (*) ", display_name)
            } else {
                format!(" {} ", display_name)
            };

            ListItem::new(label).style(style)
        })
        .collect();

    let title = if app.available_templates.is_empty() {
        " Load Template (None Found) "
    } else {
        " Load Template ([D] Set Default) "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}
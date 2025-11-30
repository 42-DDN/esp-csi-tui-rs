// --- File: src/frontend/overlays/view_selector.rs ---
// --- Purpose: Popup list to select a view type for the current pane ---

use ratatui::{prelude::*, widgets::*};
use crate::App;
use crate::layout_tree::ViewType;

pub const AVAILABLE_VIEWS: [(ViewType, &str); 6] = [
    (ViewType::Dashboard, "Dashboard Stats"),
    (ViewType::Polar, "Polar Scatter"),
    (ViewType::Isometric, "3D Isometric"),
    (ViewType::Spectrogram, "Spectrogram"),
    (ViewType::Phase, "Phase Plot"),
    (ViewType::Camera, "Camera Feed"),
];

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    // Reuse the centered_rect helper from help.rs
    let area = crate::frontend::overlays::help::centered_rect(40, 40, area);

    // Clear background so we don't see content behind
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = AVAILABLE_VIEWS
        .iter()
        .enumerate()
        .map(|(i, (_, label))| {
            let style = if i == app.view_selector_index {
                // Highlight selection
                Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(format!(" {} ", label)).style(style)
        })
        .collect();

    let block = Block::default()
        .title(" Select View ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}
// --- File: src/frontend/overlays/view_selector.rs ---
// --- Purpose: Popup list to select a view type for the current pane ---

use ratatui::{prelude::*, widgets::*};
use crate::App;
use crate::layout_tree::ViewType;

pub const AVAILABLE_VIEWS: [(ViewType, &str); 7] = [
    (ViewType::Dashboard, "Dashboard Stats"),
    (ViewType::Polar, "Polar Scatter"),
    (ViewType::Isometric, "3D Isometric"),
    (ViewType::Spectrogram, "Spectrogram"),
    (ViewType::Phase, "Phase Plot"),
    (ViewType::Camera, "Camera Feed"),
    (ViewType::RawScatter, "Multipath Scatter"),
];

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let area = crate::frontend::overlays::help::centered_rect(40, 40, area);
    f.render_widget(Clear, area);

    let items: Vec<ListItem> = AVAILABLE_VIEWS
        .iter()
        .enumerate()
        .map(|(i, (_, label))| {
            // Dynamic selection style based on theme
            let style = if i == app.view_selector_index {
                app.theme.sidebar_selected
            } else {
                app.theme.text_normal
            };
            ListItem::new(format!(" {} ", label)).style(style)
        })
        .collect();

    let block = Block::default()
        .title(" Select View ")
        .borders(Borders::ALL)
        .border_style(app.theme.focused_border)
        .style(app.theme.root);

    let list = List::new(items)
        .block(block);

    f.render_widget(list, area);
}
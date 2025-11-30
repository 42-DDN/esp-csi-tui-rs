// --- File: src/frontend/overlays/options.rs ---
// --- Purpose: Options popup overlay for export and settings ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let area = crate::frontend::overlays::help::centered_rect(50, 20, area);

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Options ")
        .borders(Borders::ALL)
        .border_style(app.theme.focused_border)
        .style(app.theme.root);

    let text = Paragraph::new("Export Data [E] \nTheme [T] \nClose [O]")
        .block(block)
        .alignment(Alignment::Center)
        .style(app.theme.text_normal);

    f.render_widget(text, area);
}
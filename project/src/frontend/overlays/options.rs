// --- File: src/options.rs ---
// --- Purpose: Options popup overlay for export and settings ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, _app: &App, area: Rect) {
    let area = crate::help::centered_rect(50, 20, area);

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Options ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::DarkGray));

    let text = Paragraph::new("Export Data [E] \nTheme [T] \nClose [O]")
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(text, area);
}
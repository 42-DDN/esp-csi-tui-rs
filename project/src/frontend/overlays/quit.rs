// --- File: src/frontend/overlays/quit.rs ---
// --- Purpose: Confirmation popup when quitting ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let area = crate::frontend::overlays::help::centered_rect(40, 20, area);

    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Confirm Quit ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .style(app.theme.root);

    let text = Paragraph::new("Are you sure you want to quit?\n\n[Y] Yes    [N] No")
        .block(block)
        .alignment(Alignment::Center)
        .style(app.theme.text_highlight); // Highlighted text for emphasis

    f.render_widget(text, area);
}
// --- File: src/frontend/overlays/quit.rs ---
// --- Purpose: Confirmation popup when quitting ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let area = crate::frontend::overlays::help::centered_rect(40, 20, area);

    // Clear background so lines don't bleed through
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Confirm Quit ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::DarkGray)); // Distinct background

    let text = Paragraph::new("Are you sure you want to quit?\n\n[Y] Yes    [N] No")
        .block(block)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
        
    f.render_widget(text, area);
}
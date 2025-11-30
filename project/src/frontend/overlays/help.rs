// --- File: src/help.rs ---
// --- Purpose: Help popup overlay showing keybindings ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, _app: &App, area: Rect) {
    let block = Block::default().title(" Help ").borders(Borders::ALL);
    let text = Paragraph::new(
        "NAVIGATION:\n\
        Shift + Arrows : Split Pane\n\
        Tab            : Cycle Focus\n\
        Enter          : Toggle Menu / Select View\n\
        T              : Toggle Theme\n\
        Q              : Quit"
    )
    .block(block)
    .alignment(Alignment::Center);

    let area = centered_rect(60, 40, area);
    f.render_widget(Clear, area);
    f.render_widget(text, area);
}

// Utility to center a rect
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
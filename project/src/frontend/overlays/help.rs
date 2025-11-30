// --- File: src/frontend/overlays/help.rs ---
// --- Purpose: Help popup overlay showing keybindings ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    // 1. Center the popup
    let area = centered_rect(60, 50, area);

    // 2. Clear background to avoid bleed-through
    f.render_widget(Clear, area);

    // 3. Define Block with Theme Colors
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(app.theme.focused_border) // Theme Border
        .style(app.theme.root);                 // Theme Background

    // 4. Content
    let text = Paragraph::new(
        "NAVIGATION:\n\
        Shift + Arrows : Split Pane\n\
        Tab            : Cycle Focus\n\
        Delete         : Close Pane\n\
        0-9 / Click    : Select Pane\n\
        \n\
        ACTIONS:\n\
        Enter          : Toggle Menu / Select View\n\
        T              : Toggle Theme\n\
        M              : Main Menu\n\
        Q              : Quit"
    )
    .block(block)
    .alignment(Alignment::Center)
    .style(app.theme.text_normal); // Theme Text

    f.render_widget(text, area);
}

// Utility to center a rect (Public so other overlays can use it)
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
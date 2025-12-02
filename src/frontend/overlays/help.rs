// --- File: src/frontend/overlays/help.rs ---
// --- Purpose: Help popup overlay showing keybindings ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    // 1. Center the popup
    let area = centered_rect(70, 70, area); // Increased width for table

    // 2. Clear background
    f.render_widget(Clear, area);

    // 3. Define Block
    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .border_style(app.theme.focused_border)
        .style(app.theme.root);

    // 4. Content - Table
    let rows = vec![
        // Section: Tiling
        Row::new(vec![Span::styled(" TILING & GENERAL ", Style::default().add_modifier(Modifier::BOLD)), Span::raw("")]),
        Row::new(vec![" Shift + Arrows", " Split Pane"]),
        Row::new(vec![" Delete", " Close Pane"]),
        Row::new(vec![" Tab / Click", " Focus Pane"]),
        Row::new(vec![" Space", " Toggle Fullscreen"]),
        Row::new(vec![" Drag Divider", " Resize Panes"]), // Added Dragging info
        Row::new(vec!["", ""]),

        // Section: Playback
        Row::new(vec![Span::styled(" PLAYBACK & CAMERA ", Style::default().add_modifier(Modifier::BOLD)), Span::raw("")]),
        Row::new(vec![" Left / Right", " Step History (Paused)"]),
        Row::new(vec![" W / A / S / D", " Move 3D Camera"]),
        Row::new(vec![" R", " Reset to Live/Default"]),
        Row::new(vec!["", ""]),

        // Section: Menus
        Row::new(vec![Span::styled(" MENUS & SYSTEM ", Style::default().add_modifier(Modifier::BOLD)), Span::raw("")]),
        Row::new(vec![" Enter", " View Selector"]),
        Row::new(vec![" M", " Main Menu"]),
        Row::new(vec![" T", " Next Theme"]),
        Row::new(vec![" Q", " Quit"]),
    ];

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(60),
    ];

    let table = Table::new(rows, widths)
        .block(block)
        .column_spacing(1)
        .style(app.theme.text_normal)
        .header(
            Row::new(vec!["Key", "Action"])
                .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .bottom_margin(1)
        );

    f.render_widget(table, area);
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
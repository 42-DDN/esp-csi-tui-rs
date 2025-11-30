// --- File: src/stats.rs ---
// --- Purpose: Dashboard view displaying live statistics and signal quality ---
//! ZOMBIE AND AMMAR USE THIS FILE AS A TEMPLATE FOR OTHER VIEW MODULES!
use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect, is_focused: bool) {
    // 1. Select Style based on theme & focus
    let border_style = if is_focused { app.theme.focused_border } else { app.theme.normal_border };

    // 2. Define the container block with Theme Root style
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Dashboard Stats ")
        .border_style(border_style)
        .style(app.theme.root); // Apply background color

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // 3. Render Content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)])
        .split(inner_area);

    let stats_text = vec![
        Line::from(vec![Span::raw("Packets: "), Span::styled(format!("{}", app.packet_count), app.theme.text_highlight)]),
        Line::from(vec![Span::raw("Last RSSI: "), Span::styled(format!("{} dBm", app.last_rssi), app.theme.text_normal)]),
        Line::from(vec![Span::raw("Current Theme: "), Span::styled(format!("{:?}", app.theme.variant), app.theme.text_highlight)]),
    ];

    f.render_widget(Paragraph::new(stats_text).style(app.theme.root), chunks[0]);

    // Gauge
    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::TOP).title(" Signal Quality ").border_style(app.theme.normal_border))
        .gauge_style(Style::default().fg(app.theme.gauge_color))
        .percent(((app.last_rssi + 100).max(0) as u16).min(100));

    f.render_widget(gauge, chunks[1]);
}
// --- File: src/frontend/views/stats.rs ---
// --- Purpose: Dashboard view displaying live statistics and signal quality ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect, is_focused: bool, id: usize) {
    let border_style = if is_focused { app.theme.focused_border } else { app.theme.normal_border };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" [Pane {}] Dashboard ", id))
        .border_style(border_style)
        .style(app.theme.root);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Standard Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Min(0)])
        .split(inner_area);

    let stats_text = vec![
        Line::from(vec![Span::raw("Packets: "), Span::styled(format!("{}", app.packet_count), app.theme.text_highlight)]),
        Line::from(vec![Span::raw("Last RSSI: "), Span::styled(format!("{} dBm", app.last_rssi), app.theme.text_normal)]),
        Line::from(vec![Span::raw("Theme: "), Span::styled(format!("{:?}", app.theme.variant), app.theme.text_highlight)]),
    ];

    f.render_widget(Paragraph::new(stats_text).style(app.theme.root), chunks[0]);

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::TOP).title(" Signal ").border_style(app.theme.normal_border))
        .gauge_style(Style::default().fg(app.theme.gauge_color))
        .percent(((app.last_rssi + 100).max(0) as u16).min(100));

    f.render_widget(gauge, chunks[1]);
}
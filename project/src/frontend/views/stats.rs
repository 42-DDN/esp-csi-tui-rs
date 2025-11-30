// --- File: src/frontend/views/stats.rs ---
// --- Purpose: Dashboard view displaying live statistics and signal quality ---

use ratatui::{prelude::*, widgets::*};
use crate::App;
use crate::frontend::responsive::LayoutDensity;

pub fn draw(f: &mut Frame, app: &App, area: Rect, is_focused: bool, id: usize, density: LayoutDensity) {
    let border_style = if is_focused { app.theme.focused_border } else { app.theme.normal_border };

    // 1. Configure Title based on space
    let title = if density == LayoutDensity::Compact {
        format!(" #{} ", id) // Minimal title
    } else {
        format!(" [Pane {}] Dashboard ", id)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style)
        .style(app.theme.root);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // 2. Responsive Layout
    if density == LayoutDensity::Compact {
        // COMPACT MODE: Just show the text, hide the gauge
        let stats_text = vec![
            Line::from(vec![Span::raw("Pkts: "), Span::styled(format!("{}", app.packet_count), app.theme.text_highlight)]),
            Line::from(vec![Span::raw("RSSI: "), Span::styled(format!("{}", app.last_rssi), app.theme.text_normal)]),
        ];
        f.render_widget(Paragraph::new(stats_text).style(app.theme.root), inner_area);
    } else {
        // NORMAL MODE: Show Text AND Gauge
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
}
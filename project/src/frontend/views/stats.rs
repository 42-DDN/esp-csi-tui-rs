// --- File: src/frontend/views/stats.rs ---
// --- Purpose: Dashboard view displaying live statistics and signal quality ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect, is_focused: bool, id: usize) {
    let border_style = if is_focused { app.theme.focused_border } else { app.theme.normal_border };

    // 1. Determine Data Source (Live or History)
    let (data, status_label) = if let Some(state) = app.pane_states.get(&id) {
        if state.history_offset > 0 && state.history_offset <= app.history.len() {
            let index = app.history.len() - state.history_offset;
            (app.history[index], format!(" [REPLAY -{}] ", state.history_offset))
        } else {
            (app.current_stats, " [LIVE] ".to_string())
        }
    } else {
        (app.current_stats, "".to_string())
    };

    // 2. Build Title
    let title = format!(" [Pane {}]{}Network Stats ", id, status_label);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style)
        .style(app.theme.root);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Centered Vertical Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3), // PPS
            Constraint::Length(1),
            Constraint::Length(3), // SNR
            Constraint::Length(1),
            Constraint::Length(3), // RSSI
            Constraint::Min(0),
        ])
        .split(inner_area);

    // --- 1. Packets Per Second (PPS) ---
    let pps_percent = (data.pps as f64 / 1000.0 * 100.0).clamp(0.0, 100.0) as u16;
    let pps_gauge = Gauge::default()
        .block(Block::default().title(" Packets Per Second ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(pps_percent)
        .label(format!("{} PPS", data.pps));
    f.render_widget(pps_gauge, chunks[1]);

    // --- 2. SNR ---
    let snr_percent = (data.snr as f64 / 50.0 * 100.0).clamp(0.0, 100.0) as u16;
    let snr_gauge = Gauge::default()
        .block(Block::default().title(" Signal-to-Noise Ratio (SNR) ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(snr_percent)
        .label(format!("{} dB", data.snr));
    f.render_widget(snr_gauge, chunks[3]);

    // --- 3. RSSI ---
    let rssi_percent = ((data.rssi + 100).max(0) as u16).min(100);
    let rssi_gauge = Gauge::default()
        .block(Block::default().title(" RSSI (Signal Strength) ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(app.theme.gauge_color))
        .percent(rssi_percent)
        .label(format!("{} dBm", data.rssi));
    f.render_widget(rssi_gauge, chunks[5]);
}
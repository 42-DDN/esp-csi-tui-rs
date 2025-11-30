// --- File: src/frontend/views/stats.rs ---
// --- Purpose: Dashboard view displaying live statistics and signal quality ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect, is_focused: bool, id: usize) {
    let border_style = if is_focused { app.theme.focused_border } else { app.theme.normal_border };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" [Pane {}] Network Statistics ", id))
        .border_style(border_style)
        .style(app.theme.root);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Centered Vertical Layout
    // We use Min(0) for top/bottom to act as flexible spacers that share remaining height equally
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Top Spacer (Flexible)
            Constraint::Length(3), // PPS Meter
            Constraint::Length(1), // Gap
            Constraint::Length(3), // SNR Meter
            Constraint::Length(1), // Gap
            Constraint::Length(3), // RSSI Gauge
            Constraint::Min(0),    // Bottom Spacer (Flexible)
        ])
        .split(inner_area);

    // --- 1. Packets Per Second (PPS) Meter ---
    let mock_pps = (app.packet_count % 50) * 12 + 100;
    let pps_percent = (mock_pps as f64 / 1000.0 * 100.0).clamp(0.0, 100.0) as u16;

    let pps_gauge = Gauge::default()
        .block(Block::default().title(" Packets Per Second ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(pps_percent)
        .label(format!("{} PPS", mock_pps));

    // Render at index 1 (after top spacer)
    f.render_widget(pps_gauge, chunks[1]);

    // --- 2. Signal to Noise Ratio (SNR) Meter ---
    let noise_floor = -95;
    let snr = app.last_rssi - noise_floor;
    let snr_percent = (snr as f64 / 50.0 * 100.0).clamp(0.0, 100.0) as u16;

    let snr_gauge = Gauge::default()
        .block(Block::default().title(" Signal-to-Noise Ratio (SNR) ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(snr_percent)
        .label(format!("{} dB", snr));

    // Render at index 3 (after gap)
    f.render_widget(snr_gauge, chunks[3]);

    // --- 3. RSSI Gauge ---
    let rssi_percent = ((app.last_rssi + 100).max(0) as u16).min(100);

    let rssi_gauge = Gauge::default()
        .block(Block::default().title(" RSSI (Signal Strength) ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(app.theme.gauge_color))
        .percent(rssi_percent)
        .label(format!("{} dBm", app.last_rssi));

    // Render at index 5 (after gap)
    f.render_widget(rssi_gauge, chunks[5]);
}
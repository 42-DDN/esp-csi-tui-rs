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

    // Vertical Stack: 3 Gauges
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // PPS Meter
            Constraint::Length(3), // SNR Meter
            Constraint::Length(3), // RSSI Gauge
            Constraint::Min(0),    // Spacing
        ])
        .split(inner_area);

    // --- 1. Packets Per Second (PPS) Meter ---
    // MOCK: Simulating PPS variation based on packet_count for visualization
    let mock_pps = (app.packet_count % 50) * 12 + 100; // Fluctuates between 100-700
    let pps_percent = (mock_pps as f64 / 1000.0 * 100.0).clamp(0.0, 100.0) as u16;

    let pps_gauge = Gauge::default()
        .block(Block::default().title(" Packets Per Second ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(pps_percent)
        .label(format!("{} PPS", mock_pps));

    f.render_widget(pps_gauge, chunks[0]);

    // --- 2. Signal to Noise Ratio (SNR) Meter ---
    // MOCK: Deriving SNR assuming a noise floor of -95dBm
    // SNR = Signal (RSSI) - Noise (-95)
    let noise_floor = -95;
    let snr = app.last_rssi - noise_floor; // e.g., -60 - (-95) = 35dB
    let snr_percent = (snr as f64 / 50.0 * 100.0).clamp(0.0, 100.0) as u16; // Scale 0-50dB to 0-100%

    let snr_gauge = Gauge::default()
        .block(Block::default().title(" Signal-to-Noise Ratio (SNR) ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(snr_percent)
        .label(format!("{} dB", snr));

    f.render_widget(snr_gauge, chunks[1]);

    // --- 3. RSSI Gauge ---
    // RSSI typically ranges from -90 (bad) to -30 (perfect)
    // We map -100..0 to 0..100%
    let rssi_percent = ((app.last_rssi + 100).max(0) as u16).min(100);
    
    let rssi_gauge = Gauge::default()
        .block(Block::default().title(" RSSI (Signal Strength) ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(app.theme.gauge_color))
        .percent(rssi_percent)
        .label(format!("{} dBm", app.last_rssi));

    f.render_widget(rssi_gauge, chunks[2]);
}
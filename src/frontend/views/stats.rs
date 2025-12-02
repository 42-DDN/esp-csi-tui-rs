// --- File: src/frontend/views/stats.rs ---
// --- Purpose: Dashboard view displaying live statistics and signal quality ---

use ratatui::{prelude::*, widgets::*};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect, is_focused: bool, id: usize) {
    let border_style = if is_focused { app.theme.focused_border } else { app.theme.normal_border };

    // 1. Determine Data Source (Live vs History)
    let mut stats = &app.current_stats;
    let mut status_label = " [LIVE] ".to_string();
    let mut status_style = Style::default().fg(Color::Green).add_modifier(Modifier::BOLD);

    if let Some(state) = app.pane_states.get(&id) {
        if let Some(anchor_id) = state.anchor_packet_id {
            // Find the packet in history that matches our anchor ID
            if let Some(found_packet) = app.history.iter().find(|p| p.packet_count == anchor_id) {
                stats = found_packet;
                status_label = format!(" [REPLAY ID:{}] ", anchor_id);
                status_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
            } else {
                status_label = " [EXPIRED] ".to_string();
                status_style = Style::default().fg(Color::Red);
            }
        }
    }

    // 2. Build Title with Status
    let title = Line::from(vec![
        Span::styled(format!(" [Pane {}] Network Stats", id), app.theme.text_normal),
        Span::styled(status_label, status_style),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border_style)
        .style(app.theme.root);

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // 3. Layout (Centered Vertically)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3), // PPS Meter
            Constraint::Length(1), // Gap
            Constraint::Length(3), // SNR Meter
            Constraint::Length(1), // Gap
            Constraint::Length(3), // RSSI Gauge
            Constraint::Length(1), // Gap
            Constraint::Length(1), // Footer
            Constraint::Min(0),
        ])
        .split(inner_area);

    // --- Packets Per Second (PPS) ---
    // Scale: 0 to 1000 PPS
    let pps_percent = (stats.pps as f64 / 1000.0 * 100.0).clamp(0.0, 100.0) as u16;
    let pps_gauge = Gauge::default()
        .block(Block::default().title(" Packets Per Second ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(pps_percent)
        .label(format!("{} PPS", stats.pps));

    f.render_widget(pps_gauge, chunks[1]);

    // --- Signal-to-Noise Ratio (SNR) ---
    // Scale: 0 to 60 dB
    let snr_percent = (stats.snr as f64 / 60.0 * 100.0).clamp(0.0, 100.0) as u16;
    let snr_gauge = Gauge::default()
        .block(Block::default().title(" Signal-to-Noise Ratio (SNR) ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(snr_percent)
        .label(format!("{} dB", stats.snr));

    f.render_widget(snr_gauge, chunks[3]);

    // --- RSSI ---
    // Scale: -100 dBm to 0 dBm
    let rssi_percent = ((stats.rssi + 100).max(0) as u16).min(100);
    let rssi_gauge = Gauge::default()
        .block(Block::default().title(" RSSI (Signal Strength) ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(app.theme.gauge_color))
        .percent(rssi_percent)
        .label(format!("{} dBm", stats.rssi));

    f.render_widget(rssi_gauge, chunks[5]);

    // --- Footer (MAC & Time) ---
    let mac_str = stats.csi.as_ref().map(|c| c.mac.as_str()).unwrap_or("Waiting...");

    let meta_text = Line::from(vec![
        Span::raw("Time: "),
        Span::styled(format!("{}ms", stats.timestamp), app.theme.text_highlight),
        Span::raw(" | Source: "),
        Span::styled(mac_str, app.theme.text_highlight),
    ]);

    f.render_widget(
        Paragraph::new(meta_text).alignment(Alignment::Center),
        chunks[7]
    );
}
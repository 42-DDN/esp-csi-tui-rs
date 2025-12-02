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
            // REPLAY MODE: We are anchored to a specific packet ID.
            // Search for it in history.
            // Note: Since history is a ring buffer, the packet might have fallen off.
            // In a robust app, you'd handle that. Here we fallback to current or closest.

            if let Some(found_packet) = app.history.iter().find(|p| p.packet_count == anchor_id) {
                stats = found_packet;
                status_label = format!(" [REPLAY ID:{}] ", anchor_id);
                status_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
            } else {
                // Packet fell out of history buffer
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

    // Layout: Vertical Stack with Spacers for Centering
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Top Spacer
            Constraint::Length(3), // PPS Meter
            Constraint::Length(1), // Gap
            Constraint::Length(3), // SNR Meter
            Constraint::Length(1), // Gap
            Constraint::Length(3), // RSSI Gauge
            Constraint::Length(1), // Gap
            Constraint::Length(1), // Metadata Footer
            Constraint::Min(0),    // Bottom Spacer
        ])
        .split(inner_area);

    // --- 1. Packets Per Second (PPS) Meter ---
    let pps_percent = (stats.pps as f64 / 1000.0 * 100.0).clamp(0.0, 100.0) as u16;
    let pps_gauge = Gauge::default()
        .block(Block::default().title(" Packets Per Second ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(Color::Cyan))
        .percent(pps_percent)
        .label(format!("{} PPS", stats.pps));

    f.render_widget(pps_gauge, chunks[1]);

    // --- 2. Signal to Noise Ratio (SNR) Meter ---
    let snr_percent = (stats.snr as f64 / 60.0 * 100.0).clamp(0.0, 100.0) as u16; // 0-60dB scale
    let snr_gauge = Gauge::default()
        .block(Block::default().title(" Signal-to-Noise Ratio (SNR) ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(Color::Green))
        .percent(snr_percent)
        .label(format!("{} dB", stats.snr));

    f.render_widget(snr_gauge, chunks[3]);

    // --- 3. RSSI Gauge ---
    let rssi_percent = ((stats.rssi + 100).max(0) as u16).min(100);
    let rssi_gauge = Gauge::default()
        .block(Block::default().title(" RSSI (Signal Strength) ").borders(Borders::BOTTOM))
        .gauge_style(Style::default().fg(app.theme.gauge_color))
        .percent(rssi_percent)
        .label(format!("{} dBm", stats.rssi));

    f.render_widget(rssi_gauge, chunks[5]);

    // --- 4. Metadata Footer (Timestamp & MAC) ---
    // Extract MAC if available in the CSI packet
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
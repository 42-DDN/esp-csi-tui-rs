// --- File: src/frontend/views/spectrogram.rs ---
// --- Purpose: Doppler Spectrogram (Phase Velocity / Variance Visualization) ---

use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::canvas::{Canvas, Rectangle};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect, is_focused: bool, id: usize) {
    let theme = &app.theme;
    let state = app.pane_states.get(&id).cloned().unwrap_or_else(crate::frontend::view_state::ViewState::new);

    let border_style = if is_focused { theme.focused_border } else { theme.normal_border };
    let history_len = app.history.len();

    // 1. Determine Status & Target Packet
    let mut status_label = " [LIVE] ".to_string();
    let mut status_style = Style::default().fg(Color::Green).add_modifier(Modifier::BOLD);
    let mut target_index = history_len.saturating_sub(1);

    if let Some(anchor) = state.anchor_packet_id {
        if let Some(idx) = app.history.iter().position(|p| p.id == anchor) {
            target_index = idx;
            status_label = format!(" [REPLAY ID:{}] ", anchor);
            status_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
        } else {
            status_label = " [EXPIRED] ".to_string();
            status_style = Style::default().fg(Color::Red);
        }
    }

    if history_len < 2 {
        let block = Block::default()
            .title(format!(" #{} Doppler Spectrogram ", id))
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(theme.root);
        f.render_widget(block, area);
        return;
    }

    let stats = &app.history[target_index];

    // 2. Setup Data Slice
    // We need pairs of packets to calculate phase difference (Doppler).
    // Show last N packets.
    const WINDOW_SIZE: usize = 60;
    let start_index = target_index.saturating_sub(WINDOW_SIZE);
    let slice = &app.history[start_index..=target_index];

    // 3. Build Block
    let title_top = Line::from(vec![
        Span::styled(format!(" #{} Doppler Spectrogram (Phase Variance) ", id), theme.text_normal),
        Span::styled(status_label, status_style),
    ]);

    let footer_text = format!(" Time: {}ms | Window: {} pkts ", stats.timestamp, slice.len());
    let title_bottom = Line::from(Span::styled(footer_text, theme.text_highlight));

    let block = Block::default()
        .title(title_top)
        .title_bottom(title_bottom.alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(theme.root);

    // 4. Calculate Doppler Matrix
    // Matrix[time][subcarrier] = Phase Difference
    let mut matrix: Vec<Vec<f64>> = Vec::with_capacity(slice.len());
    let mut max_subcarriers = 64;

    for i in 1..slice.len() {
        let curr = &slice[i];
        let prev = &slice[i-1];

        let mut row = Vec::new();

        if let (Some(csi_curr), Some(csi_prev)) = (&curr.csi, &prev.csi) {
            let sc_count = csi_curr.csi_raw_data.len() / 2;
            if sc_count > max_subcarriers { max_subcarriers = sc_count; }

            for s in 0..sc_count {
                // Current Phase
                let i_c = csi_curr.csi_raw_data.get(s * 2).copied().unwrap_or(0) as f64;
                let q_c = csi_curr.csi_raw_data.get(s * 2 + 1).copied().unwrap_or(0) as f64;
                let phase_c = q_c.atan2(i_c);

                // Previous Phase
                let i_p = csi_prev.csi_raw_data.get(s * 2).copied().unwrap_or(0) as f64;
                let q_p = csi_prev.csi_raw_data.get(s * 2 + 1).copied().unwrap_or(0) as f64;
                let phase_p = q_p.atan2(i_p);

                // Phase Difference (Doppler Proxy)
                let mut diff = phase_c - phase_p;

                // Unwrap phase
                if diff > std::f64::consts::PI { diff -= 2.0 * std::f64::consts::PI; }
                if diff < -std::f64::consts::PI { diff += 2.0 * std::f64::consts::PI; }

                row.push(diff.abs());
            }
        }
        matrix.push(row);
    }

    // 5. Render Canvas (Heatmap)
    let canvas = Canvas::default()
        .block(block)
        .background_color(theme.root.bg.unwrap_or(Color::Reset))
        .x_bounds([0.0, max_subcarriers as f64])
        .y_bounds([0.0, WINDOW_SIZE as f64])
        .paint(move |ctx| {
            // Draw Heatmap
            // Y=0 is oldest (bottom), Y=WINDOW is newest (top)
            // Or usually Spectrograms scroll. Let's put newest at Top.

            for (t, row) in matrix.iter().enumerate() {
                for (s, &val) in row.iter().enumerate() {
                    // Normalize value for color
                    // Max theoretical phase diff is PI.
                    // In practice, small movements cause small shifts.
                    // Let's saturate at PI/4 for visibility.
                    let intensity = (val / (std::f64::consts::PI / 4.0)).clamp(0.0, 1.0);

                    let color = if intensity > 0.8 {
                        Color::Red
                    } else if intensity > 0.6 {
                        Color::Magenta
                    } else if intensity > 0.4 {
                        Color::Yellow
                    } else if intensity > 0.2 {
                        Color::Green
                    } else if intensity > 0.05 {
                        Color::Blue
                    } else {
                        Color::DarkGray
                    };

                    // Draw a "pixel" (rectangle)
                    // Width = 1.0, Height = 1.0
                    if intensity > 0.05 {
                        ctx.draw(&Rectangle {
                            x: s as f64,
                            y: t as f64,
                            width: 1.0,
                            height: 1.0,
                            color,
                        });
                    }
                }
            }

            // Draw Axes Labels
            ctx.print(0.0, -2.0, "Subcarrier Index");
            ctx.print(-5.0, WINDOW_SIZE as f64 / 2.0, "Time");

            // Draw Legend
            ctx.print(0.0, WINDOW_SIZE as f64 + 1.0, "Color: Phase Velocity (Doppler)");
        });

    f.render_widget(canvas, area);
}
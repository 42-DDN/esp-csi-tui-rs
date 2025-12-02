// --- File: src/frontend/views/spectrogram.rs ---
// --- Purpose: Doppler Spectrogram (Phase Velocity / Variance Visualization) ---
//
// [Graph Description]
// A 2D Heatmap (Spectrogram) showing the rate of change of the signal phase.
// X-Axis: Subcarrier Index
// Y-Axis: Time (History)
// Color: Magnitude of Phase Difference (Delta Phi) between consecutive packets.
//
// [Plotting Logic]
// Calculates the phase difference between packet[t] and packet[t-1] for each subcarrier.
// |Phase[t] - Phase[t-1]| is plotted as color intensity.
// Hot colors (Red/Magenta) indicate rapid phase change.
// Cool colors (Blue/Black) indicate static phase.
//
// [Concepts & Application]
// The rate of phase change is directly proportional to the relative velocity of objects
// in the environment (Doppler Shift).
// This view effectively highlights *motion*. Static objects disappear (black/blue),
// while moving objects create bright streaks.
//
// [Demo]
// Stand still: The plot should be mostly dark.
// Wave your hand quickly: You will see bright "hot" streaks appearing, corresponding
// to the Doppler shift induced by your hand's motion.
//
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
    // Use the actual matrix height for bounds to ensure it fills the area or scales correctly
    let height = matrix.len().max(1) as f64;

    // Add padding for labels
    let x_padding = 8.0;
    let y_padding = 4.0;

    let canvas = Canvas::default()
        .block(block)
        .background_color(theme.root.bg.unwrap_or(Color::Reset))
        .x_bounds([-x_padding, max_subcarriers as f64 + x_padding])
        .y_bounds([-y_padding, height + y_padding])
        .paint(move |ctx| {
            // Draw Heatmap
            for (t, row) in matrix.iter().enumerate() {
                for (s, &val) in row.iter().enumerate() {
                    // Normalize value for color
                    // Max theoretical phase diff is PI.
                    // Saturate at PI/2 for better visibility of subtle motions
                    let intensity = (val / (std::f64::consts::PI / 2.0)).clamp(0.0, 1.0);

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

            // Draw Axes Labels & Ticks
            let axis_color = Color::White;

            // X-Axis Ticks (Subcarriers)
            for s in (0..=max_subcarriers).step_by(16) {
                let x = s as f64;
                ctx.print(x, -2.0, format!("{}", s));
                ctx.draw(&ratatui::widgets::canvas::Line {
                    x1: x, y1: -0.5,
                    x2: x, y2: 0.5,
                    color: axis_color,
                });
            }
            ctx.print(max_subcarriers as f64 / 2.0 - 5.0, -3.5, "Subcarrier Index");

            // Y-Axis Ticks (Time)
            // Top is Newest (0ms ago), Bottom is Oldest
            ctx.print(-x_padding + 1.0, height, "0ms");
            ctx.print(-x_padding + 1.0, 0.0, format!("-{}pkts", height));

            // DC Null Marker (Approximate center)
            let dc_idx = max_subcarriers as f64 / 2.0;
            ctx.print(dc_idx - 1.0, height + 1.0, "DC");
            ctx.draw(&ratatui::widgets::canvas::Line {
                x1: dc_idx, y1: 0.0,
                x2: dc_idx, y2: height,
                color: Color::DarkGray,
            });

            // Legend
            ctx.print(max_subcarriers as f64 - 20.0, height + 2.0, "Color: Phase Delta (rad)");
        });    f.render_widget(canvas, area);
}
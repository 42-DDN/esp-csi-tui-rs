// --- File: src/frontend/views/time_domain_iso.rs ---
// --- Purpose: Channel Impulse Response (CIR) / Multipath View ---
// Visualizes the Time Domain (Delay Profile) via Inverse FFT of CSI data.
// X-axis: Delay (Time Lag), Y-axis: Power, Z-axis: Packet History.

use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine};
use crate::App;
use std::f64::consts::PI;

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

    // 2. Build Block
    let title_top = Line::from(vec![
        Span::styled(format!(" #{} CIR (Multipath) ", id), theme.text_normal),
        Span::styled(status_label, status_style),
    ]);

    let footer_text = format!(" Skew X: {:.1} | Skew Y: {:.1} ", state.camera_x, state.camera_y);
    let title_bottom = Line::from(Span::styled(footer_text, theme.text_highlight));

    let block = Block::default()
        .title(title_top)
        .title_bottom(title_bottom.alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(theme.root);

    if history_len == 0 {
        f.render_widget(block, area);
        return;
    }

    // 3. Prepare Data
    const DEPTH: usize = 30;
    let start_idx = target_index.saturating_sub(DEPTH);
    let end_idx = target_index.min(history_len - 1);
    // Ensure we have a valid range
    let slice = if start_idx <= end_idx {
        &app.history[start_idx..=end_idx]
    } else {
        &[]
    };

    // 4. Projection Parameters
    let skew_x = 0.5 + state.camera_x * 0.1;
    let skew_y = 0.3 + state.camera_y * 0.1;

    let z_spacing = 3.0;
    let max_z = DEPTH as f64 * z_spacing;

    // We will plot 64 delay bins
    let x_bins = 64.0f64;

    // Calculate Bounds dynamically to handle negative skew (rotation)
    let x_min_val = 0.0f64.min(max_z * skew_x);
    let x_max_val = x_bins.max(x_bins + max_z * skew_x);

    let y_min_val = 0.0f64.min(max_z * skew_y);
    let y_max_val = 100.0f64.max(100.0 + max_z * skew_y);

    let canvas = Canvas::default()
        .block(block)
        .background_color(theme.root.bg.unwrap_or(Color::Reset))
        .x_bounds([x_min_val - 20.0, x_max_val + 20.0])
        .y_bounds([y_min_val - 20.0, y_max_val + 20.0])
        .paint(move |ctx| {
            // Draw Grid / Floor
            let z_len = slice.len() as f64 * z_spacing;

            // Left Edge (Delay 0 - LOS)
            ctx.draw(&CanvasLine {
                x1: 0.0, y1: 0.0,
                x2: z_len * skew_x, y2: z_len * skew_y,
                color: Color::DarkGray,
            });
            // Right Edge (Max Delay)
            ctx.draw(&CanvasLine {
                x1: x_bins, y1: 0.0,
                x2: x_bins + z_len * skew_x, y2: z_len * skew_y,
                color: Color::DarkGray,
            });

            // Draw Packets (Back to Front)
            for (i, packet) in slice.iter().enumerate() {
                let z_index = (slice.len() - 1 - i) as f64;
                let z_offset_x = z_index * z_spacing * skew_x;
                let z_offset_y = z_index * z_spacing * skew_y;

                if let Some(csi) = &packet.csi {
                    // Compute Impulse Response (IDFT)
                    let cir = compute_cir(&csi.csi_raw_data);

                    let mut prev_x = 0.0;
                    let mut prev_y = 0.0;

                    for (bin, &power) in cir.iter().enumerate() {
                        // Scale Power for Display
                        let y_val = (power * 0.5).min(80.0);

                        let x_base = bin as f64;
                        let x_screen = x_base + z_offset_x;
                        let y_screen = y_val + z_offset_y;

                        // Color based on Power (Heatmap style)
                        let color = if y_val > 60.0 { Color::Red }
                        else if y_val > 40.0 { Color::Yellow }
                        else if y_val > 20.0 { Color::Green }
                        else if y_val > 5.0 { Color::Cyan }
                        else { Color::Blue };

                        if bin > 0 {
                            ctx.draw(&CanvasLine {
                                x1: prev_x, y1: prev_y,
                                x2: x_screen, y2: y_screen,
                                color,
                            });
                        }

                        prev_x = x_screen;
                        prev_y = y_screen;
                    }
                }
            }
        });

    f.render_widget(canvas, area);

    // Render static labels on top (Outside the Canvas coordinate system)
    let legend_text = vec![
        Line::from(Span::styled("CIR (Multipath)", theme.text_highlight.add_modifier(Modifier::BOLD))),
        Line::from(Span::styled("X: Delay | Y: Power | Z: Time", theme.text_normal)),
        Line::from(Span::styled("LOS: Left Edge (Delay 0)", theme.text_normal)),
    ];

    let legend = Paragraph::new(legend_text)
        .alignment(Alignment::Left)
        .block(Block::default().padding(Padding::new(2, 0, 1, 0))); // Padding from border

    f.render_widget(legend, area);

    let axis_label = Paragraph::new(Span::styled("Delay ->", theme.text_normal))
        .alignment(Alignment::Center)
        .block(Block::default().padding(Padding::new(0, 0, area.height.saturating_sub(2), 0))); // Push to bottom

    f.render_widget(axis_label, area);
}/// Computes the Channel Impulse Response (CIR) magnitude via IDFT
/// Returns a vector of magnitudes (Power Delay Profile)
fn compute_cir(raw_data: &[i32]) -> Vec<f64> {
    let sc_count = raw_data.len() / 2;
    let n = sc_count; // Transform size
    let mut output = Vec::with_capacity(n);

    // Naive IDFT O(N^2) - Fast enough for N=64
    // x[n] = sum(X[k] * e^(j * 2*pi * k * n / N))

    for t in 0..n {
        let mut sum_i = 0.0;
        let mut sum_q = 0.0;

        for k in 0..n {
            // Parse Complex CSI X[k]
            let i_val = raw_data.get(k * 2).copied().unwrap_or(0) as f64;
            let q_val = raw_data.get(k * 2 + 1).copied().unwrap_or(0) as f64;

            // Exponent: e^(j * theta) = cos(theta) + j*sin(theta)
            let theta = 2.0 * PI * (k as f64) * (t as f64) / (n as f64);
            let cos_t = theta.cos();
            let sin_t = theta.sin();

            // Multiply: (a + jb)(c + jd) = (ac - bd) + j(ad + bc)
            // X[k] * e^(...)
            let real = i_val * cos_t - q_val * sin_t;
            let imag = i_val * sin_t + q_val * cos_t;

            sum_i += real;
            sum_q += imag;
        }

        // Magnitude
        let mag = (sum_i.powi(2) + sum_q.powi(2)).sqrt();
        // Normalize by N (optional, but good for scale)
        output.push(mag / n as f64);
    }

    output
}
// --- File: src/frontend/views/phase.rs ---
// --- Purpose: Phase angle visualization (2.5D Wireframe Waterfall) ---

use ratatui::{prelude::*, widgets::*};
use ratatui::widgets::canvas::{Canvas, Line as CanvasLine};
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

    // Handle empty history
    if history_len == 0 {
        let block = Block::default()
            .title(format!(" #{} Phase Wireframe ", id))
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(theme.root);
        f.render_widget(block, area);
        return;
    }

    let stats = &app.history[target_index];

    // 2. Setup Waterfall Constants
    const DEPTH_STEPS: usize = 15; // How many packets to show
    let start_index = target_index.saturating_sub(DEPTH_STEPS);
    let slice = &app.history[start_index..=target_index];

    // 3. Build Block
    let title_top = Line::from(vec![
        Span::styled(format!(" #{} Phase Wireframe ", id), theme.text_normal),
        Span::styled(status_label, status_style),
    ]);
    let timestamp_text = format!(" Time: {}ms ", stats.timestamp);
    let title_bottom = Line::from(Span::styled(timestamp_text, theme.text_highlight));

    let block = Block::default()
        .title(title_top)
        .title_bottom(title_bottom.alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(theme.root);

    // 4. Prepare Data Grid for Canvas
    // grid[time_step][subcarrier_index] = (x, y, phase_val)

    // Projection Constants (Oblique)
    // x_screen = subcarrier + (depth * offset_x)
    // y_screen = phase + (depth * offset_y)
    let offset_x = 0.8;  // Shift right as we go back
    let offset_y = 0.4;  // Shift up as we go back
    let scale_y = 2.0;   // Stretch phase for visibility

    // Pass 1: Find global max subcarriers in the current slice to ensure rectangular grid
    let mut max_subcarriers = 64.0;
    for packet in slice.iter() {
        if let Some(csi) = &packet.csi {
            let sc = (csi.csi_raw_data.len() / 2) as f64;
            if sc > max_subcarriers { max_subcarriers = sc; }
        }
    }

    let mut grid: Vec<Vec<(f64, f64)>> = Vec::with_capacity(slice.len());

    for (i, packet) in slice.iter().enumerate() {
        // 0 is furthest back (oldest in slice), slice.len() is newest
        // We want newest to be at the "front" (no offset), oldest at the "back" (max offset)
        let reverse_depth = (slice.len() - 1 - i) as f64;

        let mut row = Vec::new();
        let mut current_sc_count = 0;

        if let Some(csi) = &packet.csi {
            current_sc_count = csi.csi_raw_data.len() / 2;
            for s in 0..current_sc_count {
                let i_val = csi.csi_raw_data.get(s * 2).copied().unwrap_or(0) as f64;
                let q_val = csi.csi_raw_data.get(s * 2 + 1).copied().unwrap_or(0) as f64;
                let phase = q_val.atan2(i_val); // -PI to PI

                // Project
                let sx = (s as f64) + (reverse_depth * offset_x);
                let sy = (phase * scale_y) + (reverse_depth * offset_y);
                row.push((sx, sy));
            }
        }

        // Pad missing subcarriers with 0.0 phase to maintain wireframe structure
        for s in current_sc_count..(max_subcarriers as usize) {
            let phase = 0.0;
            let sx = (s as f64) + (reverse_depth * offset_x);
            let sy = (phase * scale_y) + (reverse_depth * offset_y);
            row.push((sx, sy));
        }

        grid.push(row);
    }

    // 5. Render Canvas
    // Calculate bounds with extra padding for labels
    let max_x_bound = max_subcarriers + (DEPTH_STEPS as f64 * offset_x) + 10.0; // +10 for right padding
    let min_y_bound = (-std::f64::consts::PI * scale_y) - 2.0; // -2.0 for bottom axis labels
    let max_y_bound = (std::f64::consts::PI * scale_y) + (DEPTH_STEPS as f64 * offset_y) + 4.0; // +4.0 for top padding

    let canvas = Canvas::default()
        .block(block)
        .background_color(theme.root.bg.unwrap_or(Color::Reset))
        .x_bounds([0.0, max_x_bound])
        .y_bounds([min_y_bound, max_y_bound])
        .paint(move |ctx| {
            let axis_color = theme.text_normal.fg.unwrap_or(Color::White);

            // 1. Draw Grid (Wireframe)
            // Draw from back (oldest) to front (newest) so new lines overlap old ones
            for t in 0..grid.len() {
                let row = &grid[t];

                // Color Logic: Heatmap Fade
                // Calculate normalized position (0.0 = Oldest, 1.0 = Newest)
                let normalized_age = t as f64 / (grid.len() - 1) as f64;
                
                // Interpolate color based on age
                // Newest: Theme Gauge Color (e.g., Cyan/Magenta)
                // Middle: Yellow/Green
                // Oldest: Dark Blue/Gray
                let color = if normalized_age > 0.8 {
                    theme.gauge_color // Brightest (Front)
                } else if normalized_age > 0.5 {
                    Color::Cyan // Mid-High
                } else if normalized_age > 0.2 {
                    Color::Blue // Mid-Low
                } else {
                    Color::DarkGray // Oldest (Back)
                };

                for s in 0..row.len() {
                    let (x1, y1) = row[s];

                    // 1. Draw Line to Next Subcarrier (Frequency Domain)
                    if s + 1 < row.len() {
                        let (x2, y2) = row[s+1];
                        ctx.draw(&CanvasLine { x1, y1, x2, y2, color });
                    }

                    // 2. Draw Line to Next Time Step (Time Domain)
                    // Connect to the SAME subcarrier in the NEXT (newer) packet
                    // Note: grid is ordered Oldest -> Newest.
                    // So grid[t+1] is newer (closer to front).
                    if t + 1 < grid.len() {
                        let next_row = &grid[t+1];
                        if s < next_row.len() {
                            let (x2, y2) = next_row[s];
                            // Use the same color logic for time-lines to create a cohesive mesh
                            ctx.draw(&CanvasLine { x1, y1, x2, y2, color });
                        }
                    }
                }
            }

            // 2. Draw Axes & Labels (Relative to the "Front" / Newest Packet)
            // Y-Axis (Phase) at X=0
            let phase_ticks = vec![
                (-std::f64::consts::PI, "-π"),
                (0.0, "0"),
                (std::f64::consts::PI, "+π"),
            ];

            for (val, label) in phase_ticks {
                let y_screen = val * scale_y;
                // Tick line
                ctx.draw(&CanvasLine {
                    x1: 0.0, y1: y_screen,
                    x2: -1.0, y2: y_screen,
                    color: axis_color
                });
                // Label
                ctx.print(0.0, y_screen, label);
            }

            // X-Axis (Subcarrier) at Phase = -PI (Bottom of the front packet)
            let bottom_y = -std::f64::consts::PI * scale_y;

            // Draw Axis Line
            ctx.draw(&CanvasLine {
                x1: 0.0, y1: bottom_y,
                x2: max_subcarriers, y2: bottom_y,
                color: axis_color,
            });

            // Ticks every 16 subcarriers
            for s in (0..=max_subcarriers as usize).step_by(16) {
                let x_screen = s as f64;
                ctx.draw(&CanvasLine {
                    x1: x_screen, y1: bottom_y,
                    x2: x_screen, y2: bottom_y - 0.5,
                    color: axis_color,
                });
                ctx.print(x_screen, bottom_y - 1.5, format!("{}", s));
            }

            // Axis Titles
            ctx.print(max_subcarriers / 2.0, bottom_y - 2.5, "Subcarrier");
            ctx.print(0.0, max_y_bound - 1.0, "Phase (Rad)");
        });

    f.render_widget(canvas, area);
}
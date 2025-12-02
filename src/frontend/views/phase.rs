use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders};
use ratatui::widgets::canvas::{Canvas, Line};
use crate::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect, is_focused: bool, id: usize) {
    let theme = &app.theme;
    let state = app.pane_states.get(&id).cloned().unwrap_or_else(crate::frontend::view_state::ViewState::new);

    let border_style = if is_focused { theme.focused_border } else { theme.normal_border };

    let block = Block::default()
        .title(format!(" #{} Phase (3D) ", id))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(theme.root);

    // 1. Determine Data Window
    let history_len = app.history.len();
    if history_len == 0 {
        f.render_widget(block, area);
        return;
    }

    // Find the end index based on anchor
    let end_index = if let Some(anchor) = state.anchor_packet_id {
        app.history.iter().position(|p| p.id == anchor).unwrap_or(history_len - 1)
    } else {
        history_len - 1
    };

    // How many packets to show?
    let window_size = 30;
    let start_index = end_index.saturating_sub(window_size);
    let data_slice = &app.history[start_index..=end_index];

    // 2. Setup Canvas
    let canvas_x_bound = 100.0;
    let canvas_y_bound = 100.0;

    let canvas = Canvas::default()
        .block(block)
        .x_bounds([-canvas_x_bound, canvas_x_bound])
        .y_bounds([-canvas_y_bound, canvas_y_bound])
        .paint(|ctx| {
            let scale = 2.0 * state.zoom;
            let offset_x = state.camera_x;
            let offset_y = state.camera_y;

            // Projection Function (Isometric-ish)
            let project = |x: f64, y: f64, z: f64| -> (f64, f64) {
                let iso_x = x - y;
                let iso_y = (x + y) * 0.5 - z;
                (iso_x * scale + offset_x, iso_y * scale + offset_y)
            };

            // --- Draw Axes ---
            // Define bounds in 3D space
            // Time: [-window_size/2, window_size/2] * 2.0
            // Subcarrier: [-num_subcarriers/2, num_subcarriers/2] * 1.0
            // Phase: [-PI, PI] * 5.0

            // We need to know num_subcarriers to draw the box correctly.
            // We'll guess based on the first packet or default to 64 if empty.
            let num_subcarriers = data_slice.first()
                .and_then(|s| s.csi.as_ref())
                .map(|c| c.csi_raw_data.len() / 2)
                .unwrap_or(64);

            let t_max = (window_size as f64 / 2.0) * 2.0;
            let t_min = -t_max;
            let s_max = (num_subcarriers as f64 / 2.0) * 1.0;
            let s_min = -s_max;
            let p_max = std::f64::consts::PI * 5.0;
            let p_min = -p_max;

            // Axis Lines
            let axes = [
                // Time Axis (along X)
                ((t_min, s_min, p_min), (t_max, s_min, p_min), Color::Red),
                // Subcarrier Axis (along Y)
                ((t_min, s_min, p_min), (t_min, s_max, p_min), Color::Green),
                // Phase Axis (along Z)
                ((t_min, s_min, p_min), (t_min, s_min, p_max), Color::Yellow),
            ];

            for ((x1, y1, z1), (x2, y2, z2), color) in axes {
                let (sx1, sy1) = project(x1, y1, z1);
                let (sx2, sy2) = project(x2, y2, z2);
                ctx.draw(&Line { x1: sx1, y1: sy1, x2: sx2, y2: sy2, color });
            }

            // Labels
            let (tx, ty) = project(t_max + 5.0, s_min, p_min);
            ctx.print(tx, ty, "Time");

            let (sx, sy) = project(t_min, s_max + 5.0, p_min);
            ctx.print(sx, sy, "Subcarrier");

            let (px, py) = project(t_min, s_min, p_max + 5.0);
            ctx.print(px, py, "Phase");


            // --- Draw Data ---
            // grid[time_idx][subcarrier_idx] = (screen_x, screen_y)
            let mut grid: Vec<Vec<(f64, f64)>> = Vec::new();

            for (t_idx, stats) in data_slice.iter().enumerate() {
                let mut row_points = Vec::new();
                if let Some(csi) = &stats.csi {
                    let current_subcarriers = csi.csi_raw_data.len() / 2;

                    for s_idx in 0..current_subcarriers {
                        let i_val = csi.csi_raw_data.get(s_idx * 2).copied().unwrap_or(0) as f64;
                        let q_val = csi.csi_raw_data.get(s_idx * 2 + 1).copied().unwrap_or(0) as f64;
                        let phase = q_val.atan2(i_val); // -PI to PI

                        // Map to 3D Space
                        let x = (t_idx as f64 - window_size as f64 / 2.0) * 2.0;
                        let y = (s_idx as f64 - num_subcarriers as f64 / 2.0) * 1.0;
                        let z = phase * 5.0;

                        row_points.push(project(x, y, z));
                    }
                }
                grid.push(row_points);
            }

            // Draw Wireframe
            for t in 0..grid.len() {
                for s in 0..grid[t].len() {
                    let (x1, y1) = grid[t][s];

                    // Draw line to next time step (Time evolution)
                    if t + 1 < grid.len() && s < grid[t+1].len() {
                        let (x2, y2) = grid[t+1][s];
                        ctx.draw(&Line {
                            x1, y1, x2, y2,
                            color: Color::Cyan,
                        });
                    }

                    // Draw line to next subcarrier (Frequency evolution)
                    if s + 1 < grid[t].len() {
                        let (x2, y2) = grid[t][s+1];
                        ctx.draw(&Line {
                            x1, y1, x2, y2,
                            color: Color::Blue,
                        });
                    }
                }
            }
        });

    f.render_widget(canvas, area);
}
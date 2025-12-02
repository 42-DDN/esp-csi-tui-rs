// --- File: src/frontend/views/raw_scatter.rs ---
// --- Purpose: 3D Histogram Wireframe of Complex (I/Q) Distribution ---

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

    // Determine the end index for our data window
    let end_index = if let Some(anchor) = state.anchor_packet_id {
        if let Some(idx) = app.history.iter().position(|p| p.id == anchor) {
            status_label = format!(" [REPLAY ID:{}] ", anchor);
            status_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);
            idx
        } else {
            status_label = " [EXPIRED] ".to_string();
            status_style = Style::default().fg(Color::Red);
            history_len.saturating_sub(1)
        }
    } else {
        history_len.saturating_sub(1)
    };

    if history_len == 0 {
        let block = Block::default()
            .title(format!(" #{} I/Q Distribution ", id))
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(theme.root);
        f.render_widget(block, area);
        return;
    }

    let stats = &app.history[end_index];

    // 2. Data Processing: 2D Histogram
    // Grid: 24x24 bins covering -128 to 128
    const GRID_SIZE: usize = 24;

    // Use the pre-calculated cumulative grid from the target packet
    // This allows "rewinding" to see the distribution state at that point in time.
    let grid = stats.distribution_grid;

    let mut max_count: f32 = 1.0; // Avoid div by zero
    for row in grid.iter() {
        for &val in row.iter() {
            if val > max_count { max_count = val; }
        }
    }

    // 3. Camera / View Controls
    // Azimuth (Rotation around Z) - A/D keys
    let azimuth = (std::f64::consts::PI / 4.0) + (state.camera_x * 0.1);
    let sin_a = azimuth.sin();
    let cos_a = azimuth.cos();

    // Elevation (Tilt) - W/S keys (camera_y)
    // Default 45 degrees. W (negative y) -> Increase tilt (Top view). S (positive y) -> Decrease tilt (Side view).
    let elevation = (std::f64::consts::PI / 4.0) - (state.camera_y * 0.05);
    let elevation = elevation.clamp(0.1, std::f64::consts::PI / 2.0 - 0.1);
    let sin_e = elevation.sin();
    let cos_e = elevation.cos();

    // 4. Build Block
    let title_top = Line::from(vec![
        Span::styled(format!(" #{} I/Q Distribution (Wireframe) ", id), theme.text_normal),
        Span::styled(status_label, status_style),
    ]);

    let az_deg = (azimuth.to_degrees() % 360.0 + 360.0) % 360.0;
    let el_deg = elevation.to_degrees();
    let footer_text = format!(" Rot: {:.0}° | Tilt: {:.0}° | Max: {:.0} | Time: {}ms ", az_deg, el_deg, max_count, stats.timestamp);
    let title_bottom = Line::from(Span::styled(footer_text, theme.text_highlight));

    let block = Block::default()
        .title(title_top)
        .title_bottom(title_bottom.alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(theme.root);

    // 5. Render Canvas (Isometric Wireframe)

    // Scale factors
    let scale = 5.0;
    let z_scale = 80.0 / max_count as f64;

    // Helper to project (grid_x, grid_y, count) -> (screen_x, screen_y)
    // Center the grid around 0,0
    let project = |gx: f64, gy: f64, z: f64| -> (f64, f64) {
        let x0 = gx - GRID_SIZE as f64 / 2.0;
        let y0 = gy - GRID_SIZE as f64 / 2.0;

        // 1. Rotate around Z (Azimuth)
        let x1 = x0 * cos_a - y0 * sin_a;
        let y1 = x0 * sin_a + y0 * cos_a;

        // 2. Project to Screen (Orthographic with Elevation)
        // x_screen = x1
        // y_screen = y1 * sin(e) + z * cos(e)
        // Note: In Ratatui Canvas, Y goes UP.

        let sx = x1 * scale;
        let sy = (y1 * sin_e + (z * z_scale) * cos_e) * scale;
        (sx, sy)
    };

    let canvas = Canvas::default()
        .block(block)
        .background_color(theme.root.bg.unwrap_or(Color::Reset))
        .x_bounds([-100.0, 100.0])
        .y_bounds([-80.0, 80.0])
        .paint(move |ctx| {
            // Draw Axes & Labels
            let axis_color = Color::DarkGray;

            // Base corners (z=0)
            let min_idx = 0.0;
            let max_idx = (GRID_SIZE - 1) as f64;

            let c00 = project(min_idx, min_idx, 0.0); // -128, -128
            let c10 = project(max_idx, min_idx, 0.0); // 128, -128
            let c11 = project(max_idx, max_idx, 0.0); // 128, 128
            let c01 = project(min_idx, max_idx, 0.0); // -128, 128

            // Draw Base Box
            ctx.draw(&CanvasLine { x1: c00.0, y1: c00.1, x2: c10.0, y2: c10.1, color: axis_color });
            ctx.draw(&CanvasLine { x1: c10.0, y1: c10.1, x2: c11.0, y2: c11.1, color: axis_color });
            ctx.draw(&CanvasLine { x1: c11.0, y1: c11.1, x2: c01.0, y2: c01.1, color: axis_color });
            ctx.draw(&CanvasLine { x1: c01.0, y1: c01.1, x2: c00.0, y2: c00.1, color: axis_color });

            // Axis Labels
            // Real (I) Axis: along y=min_idx (c00 -> c10)
            ctx.print(c00.0, c00.1 - 5.0, "-128");
            ctx.print(c10.0, c10.1 - 5.0, "128");
            let mid_i = project(GRID_SIZE as f64 / 2.0, min_idx, 0.0);
            ctx.print(mid_i.0, mid_i.1 - 8.0, "Real (I)");

            // Imaginary (Q) Axis: along x=min_idx (c00 -> c01)
            ctx.print(c01.0, c01.1 - 5.0, "128");
            let mid_q = project(min_idx, GRID_SIZE as f64 / 2.0, 0.0);
            ctx.print(mid_q.0 - 15.0, mid_q.1, "Imag (Q)");

            // Draw Grid Lines
            for x in 0..GRID_SIZE {
                for y in 0..GRID_SIZE {
                    let z = grid[x][y] as f64;
                    let (sx, sy) = project(x as f64, y as f64, z);

                    // Color based on height (Heatmap) - Vibrant Gradient
                    let intensity = (z / max_count as f64) as f32;
                    let color = if intensity > 0.8 {
                        Color::Magenta
                    } else if intensity > 0.6 {
                        Color::Red
                    } else if intensity > 0.4 {
                        Color::Yellow
                    } else if intensity > 0.2 {
                        Color::Green
                    } else if intensity > 0.05 {
                        Color::Cyan
                    } else {
                        Color::DarkGray
                    };

                    // Draw line to X+1
                    if x + 1 < GRID_SIZE {
                        let z_next = grid[x+1][y] as f64;
                        let (sx2, sy2) = project((x+1) as f64, y as f64, z_next);
                        ctx.draw(&CanvasLine { x1: sx, y1: sy, x2: sx2, y2: sy2, color });
                    }

                    // Draw line to Y+1
                    if y + 1 < GRID_SIZE {
                        let z_next = grid[x][y+1] as f64;
                        let (sx2, sy2) = project(x as f64, (y+1) as f64, z_next);
                        ctx.draw(&CanvasLine { x1: sx, y1: sy, x2: sx2, y2: sy2, color });
                    }
                }
            }
        });

    f.render_widget(canvas, area);
}
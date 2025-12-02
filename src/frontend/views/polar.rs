// --- File: src/frontend/views/polar.rs ---
// --- Purpose: 3D Cylindrical "Tunnel" View of Amplitude vs Subcarrier History ---
// conic plot

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

    if history_len == 0 {
        let block = Block::default()
            .title(format!(" #{} Polar Amplitude Tunnel ", id))
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(theme.root);
        f.render_widget(block, area);
        return;
    }

    let stats = &app.history[target_index];

    // 2. Setup Data Slice (Tunnel Depth)
    const DEPTH_STEPS: usize = 20;
    let start_index = target_index.saturating_sub(DEPTH_STEPS);
    let slice = &app.history[start_index..=target_index];

    // 3. Build Block
    let title_top = Line::from(vec![
        Span::styled(format!(" #{} Polar Amplitude Tunnel ", id), theme.text_normal),
        Span::styled(status_label, status_style),
    ]);

    // Footer Info
    let az_deg = (state.camera_x.to_degrees() % 360.0 + 360.0) % 360.0;
    let el_deg = state.camera_y.to_degrees();
    let footer_text = format!(" Rot: {:.0}° | Tilt: {:.0}° | Time: {}ms ", az_deg, el_deg, stats.timestamp);
    let title_bottom = Line::from(Span::styled(footer_text, theme.text_highlight));

    let block = Block::default()
        .title(title_top)
        .title_bottom(title_bottom.alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(theme.root);

    // 4. Prepare 3D Points
    // We want to map:
    // - Angle (Theta) = Subcarrier Index
    // - Radius (R) = Amplitude
    // - Depth (Z) = Time (Packet Index)

    let mut points: Vec<Vec<(f64, f64, f64)>> = Vec::with_capacity(slice.len());
    let mut max_amp: f64 = 1.0;

    for (i, packet) in slice.iter().enumerate() {
        let mut row = Vec::new();
        // Z-coordinate: 0 is newest (front), negative is older (back)
        // slice.len()-1 is the newest packet.
        // i goes from 0 (oldest) to len-1 (newest).
        // Let's make Z positive going into the screen? Or negative?
        // Let's say Z = 0 is center of rotation.
        // We want the tunnel to extend along Z.

        // Let's map i=0 (oldest) to Z = -Depth
        // i=len-1 (newest) to Z = 0
        let z_step = 15.0; // Distance between rings
        let z = (i as f64 - (slice.len() as f64 - 1.0)) * z_step;

        if let Some(csi) = &packet.csi {
            let sc_count = csi.csi_raw_data.len() / 2;
            for s in 0..sc_count {
                let i_val = csi.csi_raw_data.get(s * 2).copied().unwrap_or(0) as f64;
                let q_val = csi.csi_raw_data.get(s * 2 + 1).copied().unwrap_or(0) as f64;
                let amp = (i_val.powi(2) + q_val.powi(2)).sqrt();

                if amp > max_amp { max_amp = amp; }

                // Map Subcarrier to Angle (0 to 2PI)
                // We leave a small gap to distinguish start/end
                let theta = (s as f64 / sc_count as f64) * 2.0 * std::f64::consts::PI;

                // Convert Polar (r, theta) to Cartesian (x, y)
                // r = amp
                let x = amp * theta.cos();
                let y = amp * theta.sin();

                row.push((x, y, z));
            }
        }
        points.push(row);
    }

    // 5. Render Canvas
    // Camera State
    // camera_x -> Rotation around Z (Spinning the tunnel)
    // camera_y -> Tilt around X (Looking up/down the tunnel)

    let rot_z = state.camera_x * 0.1; // Scale sensitivity
    let tilt_x = state.camera_y * 0.05;

    let sin_rz = rot_z.sin();
    let cos_rz = rot_z.cos();
    let sin_tx = tilt_x.sin();
    let cos_tx = tilt_x.cos();

    let scale = 100.0 / max_amp; // Normalize to fit screen roughly

    // Projection Helper
    let project = |x: f64, y: f64, z: f64| -> (f64, f64) {
        // 1. Rotate around Z (Spin)
        let x1 = x * cos_rz - y * sin_rz;
        let y1 = x * sin_rz + y * cos_rz;
        let z1 = z;

        // 2. Rotate around X (Tilt)
        // y' = y*cos - z*sin
        // z' = y*sin + z*cos
        let x2 = x1;
        let y2 = y1 * cos_tx - z1 * sin_tx;
        let z2 = y1 * sin_tx + z1 * cos_tx;

        // 3. Perspective Projection
        // Simple weak perspective: x_screen = x / (1 + z/depth)
        // But for now, let's stick to Orthographic + Z-offset for "Isometric-like" look
        // or just raw Orthographic (x2, y2).
        // Since we want a "Tunnel", we need perspective.

        let depth_offset = 500.0; // Push camera back
        let factor = depth_offset / (depth_offset - z2); // z2 is negative for older packets

        let sx = x2 * factor * scale;
        let sy = y2 * factor * scale;

        (sx, sy)
    };

    let canvas = Canvas::default()
        .block(block)
        .background_color(theme.root.bg.unwrap_or(Color::Reset))
        .x_bounds([-180.0, 180.0])
        .y_bounds([-140.0, 140.0])
        .paint(move |ctx| {
            // Draw Center Cross (Origin)
            let (cx, cy) = project(0.0, 0.0, 0.0);
            ctx.print(cx, cy, "+");

            // Draw Data
            for t in 0..points.len() {
                let row = &points[t];

                // Color based on age (t=0 is oldest, t=len-1 is newest)
                let age = t as f64 / (points.len() as f64 - 1.0);
                let color = if age > 0.9 {
                    theme.gauge_color
                } else if age > 0.7 {
                    Color::Cyan
                } else if age > 0.5 {
                    Color::Blue
                } else if age > 0.3 {
                    Color::DarkGray
                } else {
                    Color::Black
                };

                for s in 0..row.len() {
                    let (x, y, z) = row[s];
                    let (sx, sy) = project(x, y, z);

                    // 1. Draw Ring (Frequency Domain)
                    if s + 1 < row.len() {
                        let (nx, ny, nz) = row[s+1];
                        let (nsx, nsy) = project(nx, ny, nz);
                        ctx.draw(&CanvasLine { x1: sx, y1: sy, x2: nsx, y2: nsy, color });
                    }

                    // 2. Draw Spine (Time Domain)
                    // Connect to same subcarrier in NEXT (newer) packet
                    if t + 1 < points.len() {
                        let next_row = &points[t+1];
                        if s < next_row.len() {
                            let (nx, ny, nz) = next_row[s];
                            let (nsx, nsy) = project(nx, ny, nz);
                            ctx.draw(&CanvasLine { x1: sx, y1: sy, x2: nsx, y2: nsy, color });
                        }
                    }
                }
            }

            // 3. Draw Reference Rings (Amplitude Orbits)
            // Draw concentric circles at fixed amplitude intervals to serve as a scale
            let ring_count = 4;
            let grid_color = Color::DarkGray;

            for r in 1..=ring_count {
                let radius_norm = r as f64 / ring_count as f64;
                let radius_val = radius_norm * max_amp;

                // Draw circle at Z=0 (Front)
                let segments = 64;
                for i in 0..segments {
                    let theta1 = (i as f64 / segments as f64) * 2.0 * std::f64::consts::PI;
                    let theta2 = ((i + 1) as f64 / segments as f64) * 2.0 * std::f64::consts::PI;

                    let x1 = radius_val * theta1.cos();
                    let y1 = radius_val * theta1.sin();
                    let x2 = radius_val * theta2.cos();
                    let y2 = radius_val * theta2.sin();

                    let (sx1, sy1) = project(x1, y1, 0.0);
                    let (sx2, sy2) = project(x2, y2, 0.0);

                    ctx.draw(&CanvasLine { x1: sx1, y1: sy1, x2: sx2, y2: sy2, color: grid_color });
                }

                // Label the orbit with its amplitude value
                // Place label at the top of the ring (Angle = PI/2)
                let lx_raw = 0.0;
                let ly_raw = radius_val;
                let (lx, ly) = project(lx_raw, ly_raw, 0.0);
                ctx.print(lx, ly, format!("{:.1} dB", radius_val));
            }

            // 4. Draw Angle Spread (Subcarrier Indices)
            // Draw lines radiating from center to max radius at specific subcarrier intervals
            let max_radius = max_amp * 1.1; // Extend slightly beyond max amplitude
            let subcarrier_step = 8;
            // Assuming 64 subcarriers for standard WiFi CSI
            let total_subcarriers = 64;

            for s in (0..total_subcarriers).step_by(subcarrier_step) {
                let theta = (s as f64 / total_subcarriers as f64) * 2.0 * std::f64::consts::PI;

                let x_end = max_radius * theta.cos();
                let y_end = max_radius * theta.sin();

                let (sx_start, sy_start) = project(0.0, 0.0, 0.0);
                let (sx_end, sy_end) = project(x_end, y_end, 0.0);

                // Draw faint line
                ctx.draw(&CanvasLine { x1: sx_start, y1: sy_start, x2: sx_end, y2: sy_end, color: Color::DarkGray });

                // Label at the end
                ctx.print(sx_end, sy_end, format!("SC{}", s));
            }

            // Draw Labels
            ctx.print(-170.0, -130.0, "Polar Amplitude Tunnel");
            ctx.print(-170.0, -138.0, "Angle: Subcarrier | Radius: Amplitude | Depth: Time");
        });

    f.render_widget(canvas, area);
}
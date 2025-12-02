// --- File: src/frontend/views/phase.rs ---
// --- Purpose: Phase angle visualization over subcarriers (2D Chart) ---

use ratatui::{prelude::*, widgets::*};
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
            .title(format!(" #{} Phase vs Subcarrier ", id))
            .borders(Borders::ALL)
            .border_style(border_style)
            .style(theme.root);
        f.render_widget(block, area);
        return;
    }

    let stats = &app.history[target_index];

    // 2. Calculate Adaptive Max Subcarriers (Decaying Peak)
    // Simulate a peak detector over recent history to smooth axis transitions
    let lookback = 150;
    let start_scan = target_index.saturating_sub(lookback);
    let mut adaptive_max = 64.0;
    let decay_rate = 0.98; // Slow decay

    for i in start_scan..=target_index {
        if let Some(packet) = app.history.get(i) {
            let current_count = packet.csi.as_ref()
                .map(|c| c.csi_raw_data.len() / 2)
                .unwrap_or(64) as f64;

            // Peak detector logic: Jump up immediately, decay down slowly
            if current_count > adaptive_max {
                adaptive_max = current_count;
            } else {
                adaptive_max = adaptive_max * decay_rate;
                // Never decay below the current packet's width (or 64)
                let min_width = current_count.max(64.0);
                if adaptive_max < min_width {
                    adaptive_max = min_width;
                }
            }
        }
    }

    let mut data_points: Vec<(f64, f64)> = Vec::new();

    if let Some(csi) = &stats.csi {
        let num_subcarriers = csi.csi_raw_data.len() / 2;
        for s_idx in 0..num_subcarriers {
            let i_val = csi.csi_raw_data.get(s_idx * 2).copied().unwrap_or(0) as f64;
            let q_val = csi.csi_raw_data.get(s_idx * 2 + 1).copied().unwrap_or(0) as f64;
            let phase = q_val.atan2(i_val); // -PI to PI
            data_points.push((s_idx as f64, phase));
        }
    }

    // 3. Build Block with Status & Timestamp
    let title_top = Line::from(vec![
        Span::styled(format!(" #{} Phase per Subcarrier ", id), theme.text_normal),
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

    // Create Dataset
    let datasets = vec![
        Dataset::default()
            .name("") // Empty name to hide legend text
            .marker(symbols::Marker::Braille)
            .style(Style::default().fg(theme.gauge_color))
            .graph_type(GraphType::Line)
            .data(&data_points),
    ];

    // Create Chart
    let chart = Chart::new(datasets)
        .block(block)
        .style(theme.root)
        .x_axis(Axis::default()
            .title("Subcarrier")
            .style(theme.text_normal)
            .bounds([0.0, adaptive_max])
            .labels(vec![
                Span::styled("0", theme.text_normal),
                Span::styled(format!("{:.0}", adaptive_max / 2.0), theme.text_normal),
                Span::styled(format!("{:.0}", adaptive_max), theme.text_normal),
            ]))
        .y_axis(Axis::default()
            .title("Phase (rad)")
            .style(theme.text_normal)
            .bounds([-3.2, 3.2]) // Slightly larger than PI
            .labels(vec![
                Span::styled("-π", theme.text_normal),
                Span::styled("0", theme.text_normal),
                Span::styled("+π", theme.text_normal),
            ]));

    f.render_widget(chart, area);
}
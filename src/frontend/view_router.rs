// --- File: src/view_router.rs ---
// --- Purpose: Recursively renders the UI based on the Layout Tree structure ---

use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::App;
use crate::layout_tree::{LayoutNode, ViewType, SplitDirection};
use crate::frontend::views::*;
use crate::frontend::overlays::*;

pub fn ui(f: &mut Frame, app: &App) {
    // 0. Reset Interaction Caches
    app.pane_regions.borrow_mut().clear();
    app.splitter_regions.borrow_mut().clear();

    // 1. Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Tiling Area
            Constraint::Length(1), // Footer
        ])
        .split(f.area());

    // 2. Draw Header
    draw_header(f, app, chunks[0]);

    // 3. Draw Main Area
    if let Some(fs_id) = app.fullscreen_pane_id {
        let view_type = find_view_type(&app.tiling.root, fs_id).unwrap_or(ViewType::Empty);
        render_pane(f, app, chunks[1], fs_id, view_type, true);
    } else {
        // Pass initial empty path
        draw_tree(f, app, &app.tiling.root, chunks[1], Vec::new());
    }

    // 4. Draw Footer
    draw_footer(f, app, chunks[2]);

    // 5. Draw Overlays
    if app.show_help { help::draw(f, app, f.area()); }
    if app.show_view_selector { view_selector::draw(f, app, f.area()); }
    if app.show_main_menu { main_menu::draw(f, app, f.area()); }
    if app.show_save_input { save_template::draw(f, app, f.area()); }
    if app.show_load_selector { load_template::draw(f, app, f.area()); }
    if app.show_export_input { export_data::draw(f, app, f.area()); }
    if app.show_theme_selector { theme_selector::draw(f, app, f.area()); }
    if app.show_quit_popup { quit::draw(f, app, f.area()); }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    // Build status indicators
    let mut status_parts = Vec::new();

    // Rerun status
    if let Some(ref streamer) = app.rerun_streamer {
        if let Ok(s) = streamer.lock() {
            if s.is_connected() {
                status_parts.push(Span::styled(" 🔴LIVE ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)));
            }
            if s.is_recording() {
                status_parts.push(Span::styled(" ⏺REC ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
            }
        }
    }

    let hotkeys = if app.fullscreen_pane_id.is_some() {
        " [Space] Exit Fullscreen | [Arrows] Playback | [WASD] Move Camera | [R] Reset Live | [Q] Quit "
    } else {
        " [Shift+Arrow] Split | [Del] Close | [Drag] Resize | [0-9] Focus | [Enter] View | [M] Menu | [Shift+R] Stream | [Shift+L] Record "
    };

    // Use theme colors for the header
    // Background: Normal Border Color (usually a muted gray)
    // Foreground: Root Text Color (usually high contrast against background)
    let bg_color = app.theme.normal_border.fg.unwrap_or(Color::DarkGray);
    let fg_color = app.theme.root.fg.unwrap_or(Color::White);

    let header = Paragraph::new(hotkeys)
        .style(Style::default().bg(bg_color).fg(fg_color).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(header, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let text = "esp-csi-tui-rs,DDN@2025";

    // Dimmer, not highlighted: Use root background and DarkGray text
    let bg_color = app.theme.root.bg.unwrap_or(Color::Reset);
    let fg_color = Color::DarkGray;

    let footer = Paragraph::new(text)
        .style(Style::default().bg(bg_color).fg(fg_color).add_modifier(Modifier::ITALIC))
        .alignment(Alignment::Center);
    f.render_widget(footer, area);
}

fn draw_tree(f: &mut Frame, app: &App, node: &LayoutNode, area: Rect, path: Vec<usize>) {
    match node {
        LayoutNode::Pane { id, view } => {
            app.pane_regions.borrow_mut().push((*id, area));
            let is_focused = *id == app.tiling.focused_pane_id;
            render_pane(f, app, area, *id, *view, is_focused);
        }
        LayoutNode::Split { direction, ratio, children } => {
            let constraints = [
                Constraint::Percentage(*ratio),
                Constraint::Percentage(100 - *ratio),
            ];
            let chunks = Layout::default()
                .direction(direction.to_ratatui())
                .constraints(constraints)
                .split(area);

            // CALCULATE SPLITTER HITBOX
            let splitter_rect = match direction {
                SplitDirection::Vertical => Rect {
                    x: area.x,
                    y: chunks[0].bottom(),
                    width: area.width,
                    height: 1,
                },
                SplitDirection::Horizontal => Rect {
                    x: chunks[0].right(),
                    y: area.y,
                    width: 1,
                    height: area.height,
                },
            };
            let container_size = match direction {
                SplitDirection::Horizontal => area.width,
                SplitDirection::Vertical => area.height,
            };
            app.splitter_regions.borrow_mut().push((path.clone(), splitter_rect, *direction, *ratio, container_size));

            for (i, child) in children.iter().enumerate() {
                if let Some(chunk) = chunks.get(i) {
                    let mut child_path = path.clone();
                    child_path.push(i);
                    draw_tree(f, app, child, *chunk, child_path);
                }
            }
        }
    }
}

fn render_pane(f: &mut Frame, app: &App, area: Rect, id: usize, view: ViewType, is_focused: bool) {
    match view {
        ViewType::Dashboard => stats::draw(f, app, area, is_focused, id),
        ViewType::Phase => phase::draw(f, app, area, is_focused, id),
        ViewType::RawScatter => raw_scatter::draw(f, app, area, is_focused, id),
        ViewType::Polar => polar::draw(f, app, area, is_focused, id),
        ViewType::Spectrogram => spectrogram::draw(f, app, area, is_focused, id),
        ViewType::Isometric => time_domain_iso::draw(f, app, area, is_focused, id),
        _ => draw_empty(f, app, area, is_focused, &view, id),
    }
}

fn find_view_type(node: &LayoutNode, target_id: usize) -> Option<ViewType> {
    match node {
        LayoutNode::Pane { id, view } => {
            if *id == target_id { Some(*view) } else { None }
        }
        LayoutNode::Split { children, .. } => {
            for child in children {
                if let Some(v) = find_view_type(child, target_id) {
                    return Some(v);
                }
            }
            None
        }
    }
}

fn draw_empty(f: &mut Frame, app: &App, area: Rect, is_focused: bool, view_type: &ViewType, id: usize) {
    let border_style = if is_focused { app.theme.focused_border } else { app.theme.normal_border };
    let block = Block::default()
        .title(format!(" #{} Empty ", id))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(app.theme.root);
    let text = Paragraph::new(format!("{}\n\n[Enter]", view_type.as_str()))
        .block(block)
        .style(app.theme.text_normal)
        .alignment(Alignment::Center);
    f.render_widget(text, area);
}
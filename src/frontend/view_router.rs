// --- File: src/view_router.rs ---
// --- Purpose: Recursively renders the UI based on the Layout Tree structure ---

use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::App;
use crate::layout_tree::{LayoutNode, ViewType};
use crate::frontend::views::*;
use crate::frontend::overlays::*;

pub fn ui(f: &mut Frame, app: &App) {
    // 0. Reset Interaction Cache for this frame
    app.pane_regions.borrow_mut().clear();

    // 1. Layout: Header (Top) vs Main Tiling Area
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header (Hotkeys)
            Constraint::Min(0),    // Tiling Area
        ])
        .split(f.area());

    // 2. Draw Header
    draw_header(f, app, chunks[0]);

    // 3. Draw Main Area (Fullscreen vs Tiling)
    if let Some(fs_id) = app.fullscreen_pane_id {
        // FULLSCREEN MODE
        // We need to find the view type for this ID to render it
        let view_type = find_view_type(&app.tiling.root, fs_id).unwrap_or(ViewType::Empty);

        // Render it taking up the whole space
        render_pane(f, app, chunks[1], fs_id, view_type, true); // true = is_focused (implicitly)
    } else {
        // TILING MODE
        draw_tree(f, app, &app.tiling.root, chunks[1]);
    }

    // 4. Draw Overlays (in z-order)
    if app.show_help { help::draw(f, app, f.area()); }
    if app.show_view_selector { view_selector::draw(f, app, f.area()); }
    if app.show_main_menu { main_menu::draw(f, app, f.area()); }
    if app.show_save_input { save_template::draw(f, app, f.area()); }
    if app.show_load_selector { load_template::draw(f, app, f.area()); }
    if app.show_theme_selector { theme_selector::draw(f, app, f.area()); }
    if app.show_quit_popup { quit::draw(f, app, f.area()); }
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    // Update Hotkeys list based on mode
    let hotkeys = if app.fullscreen_pane_id.is_some() {
        " [Space] Exit Fullscreen | [Arrows] Playback | [WASD] Move Camera | [R] Reset Live | [Q] Quit "
    } else {
        " [Shift+Arrow] Split | [Del] Close | [0-9/Click] Focus | [Enter] View | [R] Reset | [M] Menu | [Q] Quit "
    };

    let header = Paragraph::new(hotkeys)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(header, area);
}

fn draw_tree(f: &mut Frame, app: &App, node: &LayoutNode, area: Rect) {
    match node {
        LayoutNode::Pane { id, view } => {
            // Register Hitbox for mouse
            app.pane_regions.borrow_mut().push((*id, area));
            let is_focused = *id == app.tiling.focused_pane_id;

            // Delegate to render_pane
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

            for (i, child) in children.iter().enumerate() {
                if let Some(chunk) = chunks.get(i) {
                    draw_tree(f, app, child, *chunk);
                }
            }
        }
    }
}

// Helper to consolidate rendering logic
fn render_pane(f: &mut Frame, app: &App, area: Rect, id: usize, view: ViewType, is_focused: bool) {
    match view {
        ViewType::Dashboard => stats::draw(f, app, area, is_focused, id),
        // Add other views here:
        // ViewType::Polar => polar::draw(f, app, area, is_focused, id),
        _ => draw_empty(f, app, area, is_focused, &view, id),
    }
}

// Helper to find ViewType by ID (needed for Fullscreen mode where we don't traverse the whole tree)
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
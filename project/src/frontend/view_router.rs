// --- File: src/view_router.rs ---
// --- Purpose: Recursively renders the UI based on the Layout Tree structure ---

use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::App;
use crate::layout_tree::{LayoutNode, ViewType};
use crate::frontend::views::*;
use crate::frontend::overlays::*;

pub fn ui(f: &mut Frame, app: &App) {
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

    // 3. Draw Main Tiling Area
    draw_tree(f, app, &app.tiling.root, chunks[1]);

    // 4. Draw Overlays (in z-order)
    if app.show_help {
        help::draw(f, app, f.area());
    }
    if app.show_view_selector {
        view_selector::draw(f, app, f.area());
    }
    if app.show_main_menu {
        main_menu::draw(f, app, f.area());
    }
    if app.show_quit_popup {
        quit::draw(f, app, f.area());
    }
}

// HEADER
fn draw_header(f: &mut Frame, _app: &App, area: Rect) {
    let hotkeys = " [Shift+Arrow] Split | [Tab] Focus | [Enter] Select View | [M] Menu | [Q] Quit ";
    let header = Paragraph::new(hotkeys)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(header, area);
}

// THE RECURSIVE RENDERER
fn draw_tree(f: &mut Frame, app: &App, node: &LayoutNode, area: Rect) {
    match node {
        LayoutNode::Pane { id, view } => {
            let is_focused = *id == app.tiling.focused_pane_id;

            match view {
                ViewType::Dashboard => stats::draw(f, app, area, is_focused),
                // ViewType::Polar => polar::draw(f, app, area, is_focused),
                _ => draw_empty(f, app, area, is_focused, view),
            }
        }
        LayoutNode::Split { direction, ratio, children } => {
            let constraints = [
                Constraint::Percentage(*ratio),
                Constraint::Percentage(100 - *ratio),
            ];
            let chunks = Layout::default()
                .direction(*direction)
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

// Empty Pane
fn draw_empty(f: &mut Frame, app: &App, area: Rect, is_focused: bool, view_type: &ViewType) {
    let border_style = if is_focused { app.theme.focused_border } else { app.theme.normal_border };
    let block = Block::default()
        .title(" Empty Pane ")
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(app.theme.root);

    let text = Paragraph::new(format!("{}\n\n[Enter] to Select View", view_type.as_str()))
        .block(block)
        .style(app.theme.text_normal)
        .alignment(Alignment::Center);

    f.render_widget(text, area);
}
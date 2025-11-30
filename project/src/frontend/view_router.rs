// --- File: src/view_router.rs ---
// --- Purpose: Recursively renders the UI based on the Layout Tree structure ---

use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::App;
use crate::layout_tree::{LayoutNode, ViewType};
use crate::frontend::views::*;
use crate::frontend::overlays::*;
use crate::frontend::responsive::{get_density, LayoutDensity}; // <--- Import

pub fn ui(f: &mut Frame, app: &App) {
    // 0. Reset Interaction Cache
    app.pane_regions.borrow_mut().clear();

    // 1. Layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(f.area());

    // 2. Draw Header
    draw_header(f, app, chunks[0]);

    // 3. Draw Tree
    draw_tree(f, app, &app.tiling.root, chunks[1]);

    // 4. Overlays
    if app.show_help { help::draw(f, app, f.area()); }
    if app.show_view_selector { view_selector::draw(f, app, f.area()); }
    if app.show_main_menu { main_menu::draw(f, app, f.area()); }
    if app.show_quit_popup { quit::draw(f, app, f.area()); }
}

fn draw_header(f: &mut Frame, _app: &App, area: Rect) {
    let hotkeys = " [Shift+Arrow] Split | [Del] Close | [0-9/Click] Focus | [Enter] View | [M] Menu | [Q] Quit ";
    let header = Paragraph::new(hotkeys)
        .style(Style::default().bg(Color::DarkGray).fg(Color::White).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(header, area);
}

fn draw_tree(f: &mut Frame, app: &App, node: &LayoutNode, area: Rect) {
    match node {
        LayoutNode::Pane { id, view } => {
            // Register Hitbox
            app.pane_regions.borrow_mut().push((*id, area));

            let is_focused = *id == app.tiling.focused_pane_id;

            // --- RESPONSIVE CHECK ---
            let density = get_density(area);

            // If TINY, the Router handles it globally (Generic "Too Small" UI)
            if density == LayoutDensity::Tiny {
                draw_tiny_fallback(f, app, area, is_focused, *id);
                return;
            }

            // If Compact or Normal, pass that info to the View
            // (Views now accept the `density` enum)
            match view {
                ViewType::Dashboard => stats::draw(f, app, area, is_focused, *id, density),
                // ViewType::Polar => polar::draw(f, app, area, is_focused, *id, density),
                _ => draw_empty(f, app, area, is_focused, view, *id),
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

// Global "Too Small" Renderer
fn draw_tiny_fallback(f: &mut Frame, app: &App, area: Rect, is_focused: bool, id: usize) {
    let border_style = if is_focused { app.theme.focused_border } else { app.theme.normal_border };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(app.theme.root);

    // Just render the ID
    let text = Paragraph::new(format!("#{}", id))
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(text, area);
}

fn draw_empty(f: &mut Frame, app: &App, area: Rect, is_focused: bool, view_type: &ViewType, id: usize) {
    let border_style = if is_focused { app.theme.focused_border } else { app.theme.normal_border };
    let block = Block::default()
        .title(format!(" #{} Empty ", id))
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(app.theme.root);

    let text = Paragraph::new(format!("{}\n[Enter]", view_type.as_str()))
        .block(block)
        .style(app.theme.text_normal)
        .alignment(Alignment::Center);

    f.render_widget(text, area);
}
// --- File: src/view_router.rs ---
// --- Purpose: Recursively renders the UI based on the Layout Tree structure ---

use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::App;
use crate::layout_tree::{LayoutNode, ViewType};
use crate::frontend::views::*;
use crate::frontend::overlays::*;

pub fn ui(f: &mut Frame, app: &App) {
    // 1. Root Layout: Sidebar vs Tiling Area
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(25), // Sidebar width
            Constraint::Min(0),     // Tiling Area
        ])
        .split(f.area());

    // 2. Draw Sidebar
    draw_sidebar(f, app, chunks[0]);

    // 3. Recursive Draw of Tiling Area
    draw_tree(f, app, &app.tiling.root, chunks[1]);

    // 4. Overlays
    if app.show_help {
        help::draw(f, app, f.area());
    }
    if app.show_options {
        options::draw(f, app, f.area());
    }
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

fn draw_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = vec![
        "Dashboard", "Polar Scatter", "3D Isometric", "Spectrogram", "Phase Plot", "Camera Feed"
    ]
    .iter()
    .enumerate()
    .map(|(i, &t)| {
        let style = if app.sidebar_active && i == app.sidebar_index {
            app.theme.sidebar_selected
        } else {
            app.theme.sidebar_normal
        };
        ListItem::new(t).style(style)
    })
    .collect();

    let border_style = if app.sidebar_active {
        app.theme.focused_border
    } else {
        app.theme.normal_border
    };

    let list = List::new(items)
        .block(Block::default().title(" Menu ").borders(Borders::ALL).border_style(border_style).style(app.theme.root));

    f.render_widget(list, area);
}

// Empty Pane
fn draw_empty(f: &mut Frame, app: &App, area: Rect, is_focused: bool, view_type: &ViewType) {
    let border_style = if is_focused { app.theme.focused_border } else { app.theme.normal_border };
    let block = Block::default()
        .title(" Pane ")
        .borders(Borders::ALL)
        .border_style(border_style)
        .style(app.theme.root);

    let text = Paragraph::new(format!("{}\n\n[Enter] to Select View", view_type.as_str()))
        .block(block)
        .style(app.theme.text_normal)
        .alignment(Alignment::Center);

    f.render_widget(text, area);
}
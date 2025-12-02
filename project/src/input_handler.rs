// --- File: src/input_handler.rs ---
// --- Purpose: Handles keyboard input events and maps them to App actions (Controller Logic) ---

use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use std::io;
use ratatui::layout::Direction;
use crate::App;
use crate::frontend::overlays::view_selector::AVAILABLE_VIEWS;
use crate::frontend::overlays::main_menu::MENU_ITEMS;
use crate::config_manager;
use crate::frontend::view_traits::ViewBehavior;

pub fn handle_event(app: &mut App) -> io::Result<()> {
    match event::read()? {
        Event::Key(key) => {
            // --- PRIORITY 0: Popups (Save/Load/Quit/Menus) ---
            if handle_popups(app, key)? {
                return Ok(());
            }

            // --- FULLSCREEN MODE NAVIGATION ---
            // If we are in fullscreen, input handling changes completely
            if let Some(fs_id) = app.fullscreen_pane_id {
                let current_view_type = get_view_type_for_pane(app, fs_id);
                let state = app.get_pane_state_mut(fs_id);

                match key.code {
                    // EXIT Fullscreen
                    KeyCode::Char(' ') | KeyCode::Esc => {
                        app.fullscreen_pane_id = None;
                    }

                    // RESET Live
                    KeyCode::Char('r') => {
                        state.reset_live();
                    }

                    // TEMPORAL NAVIGATION (Arrows)
                    KeyCode::Left if current_view_type.is_temporal() => {
                        state.step_back();
                    }
                    KeyCode::Right if current_view_type.is_temporal() => {
                        state.step_forward();
                    }

                    // SPATIAL NAVIGATION (WASD)
                    KeyCode::Char('w') if current_view_type.is_spatial() => {
                        state.move_camera(0.0, -1.0); // Up
                    }
                    KeyCode::Char('s') if current_view_type.is_spatial() => {
                        state.move_camera(0.0, 1.0); // Down
                    }
                    KeyCode::Char('a') if current_view_type.is_spatial() => {
                        state.move_camera(-1.0, 0.0); // Left
                    }
                    KeyCode::Char('d') if current_view_type.is_spatial() => {
                        state.move_camera(1.0, 0.0); // Right
                    }

                    _ => {}
                }
                return Ok(());
            }

            // --- STANDARD TILING NAVIGATION ---
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                match key.code {
                    KeyCode::Left | KeyCode::Right => app.tiling.split(Direction::Horizontal),
                    KeyCode::Up | KeyCode::Down => app.tiling.split(Direction::Vertical),
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => app.show_quit_popup = true,
                    KeyCode::Char('h') => app.show_help = !app.show_help,
                    KeyCode::Char('m') => app.show_main_menu = !app.show_main_menu,
                    KeyCode::Char('t') => app.next_theme(),
                    KeyCode::Tab => app.tiling.focus_next(),
                    KeyCode::Delete => app.tiling.close_focused_pane(),

                    // ENTER FULLSCREEN
                    KeyCode::Char(' ') => {
                        app.fullscreen_pane_id = Some(app.tiling.focused_pane_id);
                    }

                    KeyCode::Char(c) if c.is_digit(10) => {
                        let id = if c == '0' { 10 } else { c.to_digit(10).unwrap() as usize };
                        let exists = app.pane_regions.borrow().iter().any(|(pid, _)| *pid == id);
                        if exists { app.tiling.focused_pane_id = id; }
                    }
                    KeyCode::Enter => {
                        app.show_view_selector = true;
                        app.view_selector_index = 0;
                    }
                    _ => {}
                }
            }
        },

        Event::Mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
            // Only handle mouse click focus in Tiling mode
            if app.fullscreen_pane_id.is_none() {
                let regions = app.pane_regions.borrow();
                for (id, rect) in regions.iter() {
                    if rect.contains(ratatui::layout::Position { x: column, y: row }) {
                        app.tiling.focused_pane_id = *id;
                        break;
                    }
                }
            }
        },
        _ => {}
    }
    Ok(())
}

// Helper to look up view type from tree (could be optimized)
fn get_view_type_for_pane(app: &App, id: usize) -> crate::frontend::layout_tree::ViewType {
    find_view_type_recursive(&app.tiling.root, id).unwrap_or(crate::frontend::layout_tree::ViewType::Empty)
}

fn find_view_type_recursive(node: &crate::frontend::layout_tree::LayoutNode, target: usize) -> Option<crate::frontend::layout_tree::ViewType> {
    match node {
        crate::frontend::layout_tree::LayoutNode::Pane { id, view } => {
            if *id == target { Some(*view) } else { None }
        }
        crate::frontend::layout_tree::LayoutNode::Split { children, .. } => {
            for child in children {
                if let Some(v) = find_view_type_recursive(child, target) {
                    return Some(v);
                }
            }
            None
        }
    }
}

// Helper to clean up main handle function
fn handle_popups(app: &mut App, key: crossterm::event::KeyEvent) -> io::Result<bool> {
    if app.show_save_input {
        match key.code {
            KeyCode::Enter => {
                if !app.input_buffer.is_empty() {
                    app.tiling.theme_variant = Some(app.theme.variant);
                    app.tiling.is_default = false;
                    let _ = config_manager::save_template(&app.input_buffer, &app.tiling);
                    app.show_save_input = false;
                    app.input_buffer.clear();
                }
            }
            KeyCode::Esc => {
                app.show_save_input = false;
                app.input_buffer.clear();
            }
            KeyCode::Backspace => { app.input_buffer.pop(); }
            KeyCode::Char(c) => { app.input_buffer.push(c); }
            _ => {}
        }
        return Ok(true);
    }

    if app.show_load_selector {
        match key.code {
            KeyCode::Up => {
                if app.load_selector_index > 0 { app.load_selector_index -= 1; }
                else if !app.available_templates.is_empty() { app.load_selector_index = app.available_templates.len() - 1; }
            }
            KeyCode::Down => {
                if !app.available_templates.is_empty() { app.load_selector_index = (app.load_selector_index + 1) % app.available_templates.len(); }
            }
            KeyCode::Char('d') => {
                if !app.available_templates.is_empty() {
                    let (filename, _) = &app.available_templates[app.load_selector_index];
                    let _ = config_manager::set_default_template(filename);
                    if let Ok(list) = config_manager::list_templates() { app.available_templates = list; }
                }
            }
            KeyCode::Enter => {
                if !app.available_templates.is_empty() {
                    let (filename, _) = &app.available_templates[app.load_selector_index];
                    if let Ok(new_tiling) = config_manager::load_template(filename) {
                        if let Some(variant) = new_tiling.theme_variant { app.theme = crate::theme::Theme::new(variant); }
                        app.tiling = new_tiling;
                    }
                    app.show_load_selector = false;
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => app.show_load_selector = false,
            _ => {}
        }
        return Ok(true);
    }

    if app.show_quit_popup {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => app.should_quit = true,
            KeyCode::Char('n') | KeyCode::Char('q') | KeyCode::Esc => app.show_quit_popup = false,
            _ => {}
        }
        return Ok(true);
    }

    if app.show_view_selector {
        match key.code {
            KeyCode::Up => {
                if app.view_selector_index > 0 { app.view_selector_index -= 1; } else { app.view_selector_index = AVAILABLE_VIEWS.len() - 1; }
            }
            KeyCode::Down => { app.view_selector_index = (app.view_selector_index + 1) % AVAILABLE_VIEWS.len(); }
            KeyCode::Enter => {
                let (selected_view, _) = AVAILABLE_VIEWS[app.view_selector_index];
                app.tiling.set_current_view(selected_view);
                app.show_view_selector = false;
            }
            KeyCode::Esc | KeyCode::Char('q') => app.show_view_selector = false,
            _ => {}
        }
        return Ok(true);
    }

    if app.show_main_menu {
        match key.code {
            KeyCode::Up => {
                if app.main_menu_index > 0 { app.main_menu_index -= 1; } else { app.main_menu_index = MENU_ITEMS.len() - 1; }
            }
            KeyCode::Down => { app.main_menu_index = (app.main_menu_index + 1) % MENU_ITEMS.len(); }
            KeyCode::Enter => {
                match app.main_menu_index {
                    0 => app.next_theme(),
                    1 => { app.show_main_menu = false; app.show_save_input = true; app.input_buffer.clear(); },
                    2 => { app.show_main_menu = false; if let Ok(list) = config_manager::list_templates() { app.available_templates = list; } app.load_selector_index = 0; app.show_load_selector = true; },
                    4 => app.show_main_menu = false,
                    _ => {}
                }
            }
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('m') => app.show_main_menu = false,
            _ => {}
        }
        return Ok(true);
    }

    Ok(false)
}
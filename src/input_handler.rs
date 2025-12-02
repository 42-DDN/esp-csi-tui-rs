// --- File: src/input_handler.rs ---
// --- Purpose: Handles keyboard input events and maps them to App actions (Controller Logic) ---

use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind, MouseButton, KeyEventKind};
use std::io;
use ratatui::layout::Direction;
use crate::App;
use crate::frontend::overlays::view_selector::AVAILABLE_VIEWS;
use crate::frontend::overlays::main_menu::MENU_ITEMS;
use crate::frontend::overlays::theme_selector::AVAILABLE_THEMES;
use crate::config_manager;
use crate::frontend::view_traits::ViewBehavior;
use crate::frontend::theme::Theme;

/// Returns Ok(true) if the state changed and a redraw is needed.
pub fn handle_event(app: &mut App) -> io::Result<bool> {
    match event::read()? {
        Event::Key(key) => {
            // FIX 1: Ignore Release events to prevent stuttering/double-input
            if key.kind == KeyEventKind::Release {
                return Ok(false);
            }

            // --- PRIORITY 0: Popups ---
            if handle_popups(app, key)? {
                return Ok(true);
            }

            // --- FULLSCREEN MODE NAVIGATION ---
            if let Some(fs_id) = app.fullscreen_pane_id {
                let current_view_type = get_view_type_for_pane(app, fs_id);
                let current_live_id = app.current_stats.packet_count;
                let state = app.get_pane_state_mut(fs_id);

                match key.code {
                    KeyCode::Char('q') => { app.show_quit_popup = true; return Ok(true); }
                    KeyCode::Char(' ') | KeyCode::Esc => { app.fullscreen_pane_id = None; return Ok(true); }
                    KeyCode::Char('r') => { state.reset_live(); return Ok(true); }

                    KeyCode::Left if current_view_type.is_temporal() => {
                        state.step_back(current_live_id);
                        return Ok(true);
                    }
                    KeyCode::Right if current_view_type.is_temporal() => {
                        state.step_forward(current_live_id);
                        return Ok(true);
                    }

                    KeyCode::Char('w') if current_view_type.is_spatial() => { state.move_camera(0.0, -1.0); return Ok(true); }
                    KeyCode::Char('s') if current_view_type.is_spatial() => { state.move_camera(0.0, 1.0); return Ok(true); }
                    KeyCode::Char('a') if current_view_type.is_spatial() => { state.move_camera(-1.0, 0.0); return Ok(true); }
                    KeyCode::Char('d') if current_view_type.is_spatial() => { state.move_camera(1.0, 0.0); return Ok(true); }

                    _ => return Ok(false),
                }
            }

            // --- STANDARD NAVIGATION ---
            if key.modifiers.contains(KeyModifiers::SHIFT) {
                match key.code {
                    KeyCode::Left | KeyCode::Right => { app.tiling.split(Direction::Horizontal); return Ok(true); }
                    KeyCode::Up | KeyCode::Down => { app.tiling.split(Direction::Vertical); return Ok(true); }
                    _ => return Ok(false),
                }
            } else {
                match key.code {
                    KeyCode::Char('q') => { app.show_quit_popup = true; return Ok(true); }
                    KeyCode::Char('h') => { app.show_help = !app.show_help; return Ok(true); }
                    KeyCode::Char('m') => { app.show_main_menu = !app.show_main_menu; return Ok(true); }
                    KeyCode::Char('t') => { app.next_theme(); return Ok(true); }
                    KeyCode::Tab => { app.tiling.focus_next(); return Ok(true); }
                    KeyCode::Delete => { app.tiling.close_focused_pane(); return Ok(true); }
                    KeyCode::Char(' ') => { app.fullscreen_pane_id = Some(app.tiling.focused_pane_id); return Ok(true); }

                    KeyCode::Char('r') => {
                        let id = app.tiling.focused_pane_id;
                        app.get_pane_state_mut(id).reset_live();
                        return Ok(true);
                    }

                    KeyCode::Char(c) if c.is_digit(10) => {
                        let id = if c == '0' { 10 } else { c.to_digit(10).unwrap() as usize };
                        if app.pane_regions.borrow().iter().any(|(pid, _)| *pid == id) {
                            app.tiling.focused_pane_id = id;
                            return Ok(true);
                        }
                    }
                    KeyCode::Enter => {
                        app.show_view_selector = true;
                        app.view_selector_index = 0;
                        return Ok(true);
                    }
                    _ => return Ok(false),
                }
            }
        },

        Event::Mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
            if app.fullscreen_pane_id.is_none() {
                let regions = app.pane_regions.borrow();
                for (id, rect) in regions.iter() {
                    if rect.contains(ratatui::layout::Position { x: column, y: row }) {
                        app.tiling.focused_pane_id = *id;
                        return Ok(true);
                    }
                }
            }
        },
        _ => {}
    }
    Ok(false)
}

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

// Handles all popup overlays
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
            KeyCode::Esc => { app.show_save_input = false; app.input_buffer.clear(); }
            KeyCode::Backspace => { app.input_buffer.pop(); }
            KeyCode::Char(c) => { app.input_buffer.push(c); }
            _ => {}
        }
        return Ok(true);
    }

    if app.show_theme_selector {
        match key.code {
            KeyCode::Up => {
                if app.theme_selector_index > 0 { app.theme_selector_index -= 1; }
                else { app.theme_selector_index = AVAILABLE_THEMES.len() - 1; }
            }
            KeyCode::Down => {
                app.theme_selector_index = (app.theme_selector_index + 1) % AVAILABLE_THEMES.len();
            }
            // Use Space OR Enter to select, but KEEP OPEN
            KeyCode::Enter | KeyCode::Char(' ') => {
                let (variant, _) = AVAILABLE_THEMES[app.theme_selector_index];
                app.theme = Theme::new(variant);
                // Removed: app.show_theme_selector = false;
            }
            KeyCode::Esc | KeyCode::Char('q') => app.show_theme_selector = false,
            _ => {}
        }
        return Ok(true);
    }

    if app.show_load_selector {
        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Char('d') | KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char(' ') => {
                if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') { app.show_load_selector = false; }
                else {
                    if key.code == KeyCode::Up {
                        if app.load_selector_index > 0 { app.load_selector_index -= 1; }
                        else if !app.available_templates.is_empty() { app.load_selector_index = app.available_templates.len() - 1; }
                    }
                    if key.code == KeyCode::Down {
                        if !app.available_templates.is_empty() { app.load_selector_index = (app.load_selector_index + 1) % app.available_templates.len(); }
                    }
                    if (key.code == KeyCode::Enter || key.code == KeyCode::Char(' ')) && !app.available_templates.is_empty() {
                        let (filename, _) = &app.available_templates[app.load_selector_index];
                        if let Ok(new_tiling) = config_manager::load_template(filename) {
                            if let Some(variant) = new_tiling.theme_variant { app.theme = crate::theme::Theme::new(variant); }
                            app.tiling = new_tiling;
                        }
                        app.show_load_selector = false;
                    }
                    if key.code == KeyCode::Char('d') && !app.available_templates.is_empty() {
                         let (filename, _) = &app.available_templates[app.load_selector_index];
                         let _ = config_manager::set_default_template(filename);
                         if let Ok(list) = config_manager::list_templates() { app.available_templates = list; }
                    }
                }
                return Ok(true);
            }
            _ => return Ok(false),
        }
    }

    if app.show_quit_popup {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter | KeyCode::Char(' ') => app.should_quit = true,
            KeyCode::Char('n') | KeyCode::Char('q') | KeyCode::Esc => app.show_quit_popup = false,
            _ => {}
        }
        return Ok(true);
    }

    if app.show_view_selector || app.show_main_menu {
        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Enter | KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('m') | KeyCode::Char(' ') => {
                if app.show_view_selector {
                    if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                        let (selected_view, _) = AVAILABLE_VIEWS[app.view_selector_index];
                        app.tiling.set_current_view(selected_view);
                        app.show_view_selector = false;
                    } else if key.code == KeyCode::Up {
                         if app.view_selector_index > 0 { app.view_selector_index -= 1; } else { app.view_selector_index = AVAILABLE_VIEWS.len() - 1; }
                    } else if key.code == KeyCode::Down {
                         app.view_selector_index = (app.view_selector_index + 1) % AVAILABLE_VIEWS.len();
                    } else {
                        app.show_view_selector = false;
                    }
                } else if app.show_main_menu {
                    if key.code == KeyCode::Enter || key.code == KeyCode::Char(' ') {
                        match app.main_menu_index {
                            0 => {
                                app.show_main_menu = false;
                                app.show_theme_selector = true;
                                app.theme_selector_index = 0;
                            },
                            1 => { app.show_main_menu = false; app.show_save_input = true; app.input_buffer.clear(); },
                            2 => { app.show_main_menu = false; if let Ok(list) = config_manager::list_templates() { app.available_templates = list; } app.load_selector_index = 0; app.show_load_selector = true; },
                            4 => app.show_main_menu = false,
                            _ => {}
                        }
                    } else if key.code == KeyCode::Up {
                        if app.main_menu_index > 0 { app.main_menu_index -= 1; } else { app.main_menu_index = MENU_ITEMS.len() - 1; }
                    } else if key.code == KeyCode::Down {
                        app.main_menu_index = (app.main_menu_index + 1) % MENU_ITEMS.len();
                    } else {
                        app.show_main_menu = false;
                    }
                }
                return Ok(true);
            }
            _ => return Ok(false),
        }
    }

    Ok(false)
}
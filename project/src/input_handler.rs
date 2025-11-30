// --- File: src/input_handler.rs ---
// --- Purpose: Handles keyboard input events and maps them to App actions (Controller Logic) ---

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io;
use ratatui::layout::Direction;
use crate::App;
use crate::frontend::overlays::view_selector::AVAILABLE_VIEWS;
use crate::frontend::overlays::main_menu::MENU_ITEMS;

pub fn handle_event(app: &mut App) -> io::Result<()> {
    if let Event::Key(key) = event::read()? {

        // --- PRIORITY 1: Quit Popup ---
        if app.show_quit_popup {
            match key.code {
                KeyCode::Char('y') | KeyCode::Enter => app.should_quit = true,
                KeyCode::Char('n') | KeyCode::Char('q') | KeyCode::Esc => app.show_quit_popup = false,
                _ => {}
            }
            return Ok(());
        }

        // --- PRIORITY 2: View Selector Popup ---
        if app.show_view_selector {
            match key.code {
                KeyCode::Up => {
                    if app.view_selector_index > 0 {
                        app.view_selector_index -= 1;
                    } else {
                        app.view_selector_index = AVAILABLE_VIEWS.len() - 1;
                    }
                }
                KeyCode::Down => {
                    app.view_selector_index = (app.view_selector_index + 1) % AVAILABLE_VIEWS.len();
                }
                KeyCode::Enter => {
                    let (selected_view, _) = AVAILABLE_VIEWS[app.view_selector_index];
                    app.tiling.set_current_view(selected_view);
                    app.show_view_selector = false;
                }
                KeyCode::Esc | KeyCode::Char('q') => app.show_view_selector = false,
                _ => {}
            }
            return Ok(());
        }

        // --- PRIORITY 3: Main Menu Popup ---
        if app.show_main_menu {
            match key.code {
                KeyCode::Up => {
                    if app.main_menu_index > 0 {
                        app.main_menu_index -= 1;
                    } else {
                        app.main_menu_index = MENU_ITEMS.len() - 1;
                    }
                }
                KeyCode::Down => {
                    app.main_menu_index = (app.main_menu_index + 1) % MENU_ITEMS.len();
                }
                KeyCode::Enter => {
                    match app.main_menu_index {
                        0 => app.next_theme(), // Change Theme
                        1 => {}, // Export (TODO)
                        2 => app.show_main_menu = false, // Close
                        _ => {}
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('m') => app.show_main_menu = false,
                _ => {}
            }
            return Ok(());
        }

        // --- Standard Inputs ---

        // 1. Tiling Management (Shift + Arrows)
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            match key.code {
                KeyCode::Left | KeyCode::Right => app.tiling.split(Direction::Horizontal),
                KeyCode::Up | KeyCode::Down => app.tiling.split(Direction::Vertical),
                _ => {}
            }
        }
        // 2. Global Navigation & Actions
        else {
            match key.code {
                KeyCode::Char('q') => app.show_quit_popup = true,
                KeyCode::Char('h') => app.show_help = !app.show_help,
                KeyCode::Char('m') => app.show_main_menu = !app.show_main_menu,
                KeyCode::Char('t') => app.next_theme(), // Quick toggle

                // Focus Navigation
                KeyCode::Tab => app.tiling.focus_next(),

                // Open View Selector for current pane
                KeyCode::Enter => {
                    app.show_view_selector = true;
                    app.view_selector_index = 0; // Reset selection to top
                }

                _ => {}
            }
        }
    }
    Ok(())
}
// --- File: src/input_handler.rs ---
// --- Purpose: Handles keyboard input events and maps them to App actions (Controller Logic) ---

use crossterm::event::{self, Event, KeyCode, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use std::io;
use ratatui::layout::Direction;
use crate::App;
use crate::frontend::overlays::view_selector::AVAILABLE_VIEWS;
use crate::frontend::overlays::main_menu::MENU_ITEMS;

pub fn handle_event(app: &mut App) -> io::Result<()> {
    match event::read()? {
        Event::Key(key) => {
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
                            0 => app.next_theme(),
                            1 => {},
                            2 => app.show_main_menu = false,
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
                    KeyCode::Char('t') => app.next_theme(),

                    // Focus Navigation
                    KeyCode::Tab => app.tiling.focus_next(),

                    // Close Pane
                    KeyCode::Delete => app.tiling.close_focused_pane(),

                    // Numeric Selection (0-9)
                    KeyCode::Char(c) if c.is_digit(10) => {
                        if let Some(digit) = c.to_digit(10) {
                            let id = digit as usize;
                            let exists = app.pane_regions.borrow().iter().any(|(pid, _)| *pid == id);
                            if exists {
                                app.tiling.focused_pane_id = id;
                            }
                        }
                    }

                    // Open View Selector
                    KeyCode::Enter => {
                        app.show_view_selector = true;
                        app.view_selector_index = 0;
                    }

                    _ => {}
                }
            }
        },

        // --- MOUSE INPUT ---
        Event::Mouse(MouseEvent { kind: MouseEventKind::Down(MouseButton::Left), column, row, .. }) => {
            let regions = app.pane_regions.borrow();
            for (id, rect) in regions.iter() {
                if rect.contains(ratatui::layout::Position { x: column, y: row }) {
                    app.tiling.focused_pane_id = *id;
                    break;
                }
            }
        },
        _ => {}
    }
    Ok(())
}
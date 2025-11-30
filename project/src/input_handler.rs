// --- File: src/input_handler.rs ---
// --- Purpose: Handles keyboard input events and maps them to App actions (Controller Logic) ---

use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::io;
use ratatui::layout::Direction;
use crate::App;
use crate::layout_tree::ViewType;

/// Handles a single event from crossterm.
/// Returns Ok(true) if the app should quit, Ok(false) otherwise.
pub fn handle_event(app: &mut App) -> io::Result<()> {
    if let Event::Key(key) = event::read()? {

        // 1. Tiling Management (Shift + Arrows)
        if key.modifiers.contains(KeyModifiers::SHIFT) {
            match key.code {
                KeyCode::Left | KeyCode::Right => {
                    app.tiling.split(Direction::Horizontal);
                }
                KeyCode::Up | KeyCode::Down => {
                    app.tiling.split(Direction::Vertical);
                }
                _ => {}
            }
        }
        // 2. Standard Navigation & Shortcuts
        else {
            match key.code {
                // Global Shortcuts
                KeyCode::Char('q') => app.should_quit = true,
                KeyCode::Char('h') => app.show_help = !app.show_help,
                KeyCode::Char('o') => app.show_options = !app.show_options,
                KeyCode::Char('t') => app.next_theme(),

                // Focus Navigation
                KeyCode::Tab => app.tiling.focus_next(),

                // Sidebar Navigation
                KeyCode::Down => {
                    if app.sidebar_active {
                        app.sidebar_index = (app.sidebar_index + 1) % 6; // 6 menu items
                    }
                }
                KeyCode::Up => {
                    if app.sidebar_active && app.sidebar_index > 0 {
                        app.sidebar_index -= 1;
                    }
                }

                // Selection Logic (Enter)
                KeyCode::Enter => handle_enter_key(app),

                _ => {}
            }
        }
    }
    Ok(())
}

/// Extracted logic for the Enter key to keep the main match block clean
fn handle_enter_key(app: &mut App) {
    if app.sidebar_active {
        // Apply selected view to the currently focused pane
        let view = match app.sidebar_index {
            0 => ViewType::Dashboard,
            1 => ViewType::Polar,
            2 => ViewType::Isometric,
            3 => ViewType::Spectrogram,
            4 => ViewType::Phase,
            5 => ViewType::Camera,
            _ => ViewType::Empty,
        };
        app.tiling.set_current_view(view);

        // Exit sidebar mode
        app.sidebar_active = false;
    } else {
        // Enter sidebar mode to choose a view for this pane
        app.sidebar_active = true;
    }
}
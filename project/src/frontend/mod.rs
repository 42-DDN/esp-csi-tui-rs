// --- File: src/frontend/mod.rs ---
// --- Purpose: Entry point for the frontend folder. Registers submodules. ---

pub mod layout_tree;
pub mod theme;
pub mod view_router;
pub mod view_traits; // <--- Added
pub mod view_state;  // <--- Added

pub mod views;
pub mod overlays;
// --- File: src/frontend/mod.rs ---
// --- Purpose: Entry point for the frontend folder. Registers submodules. ---

// 1. Register sibling files in src/frontend/
pub mod layout_tree;
pub mod theme;
pub mod view_router;
pub mod responsive; // <--- Added this

// 2. Register sub-directories
pub mod views;
pub mod overlays;
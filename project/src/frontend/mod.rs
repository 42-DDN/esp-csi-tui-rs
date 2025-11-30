// --- File: src/frontend/mod.rs ---
// --- Purpose: Entry point for the frontend folder. Registers submodules. ---

// 1. Register sibling files in src/frontend/
pub mod layout_tree;
pub mod theme;
pub mod view_router;

// 2. Register sub-directories
// Rust automatically looks for src/frontend/views/mod.rs
pub mod views;
pub mod overlays;
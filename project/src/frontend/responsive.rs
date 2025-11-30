// --- File: src/frontend/responsive.rs ---
// --- Purpose: Determines layout density based on available screen area ---

use ratatui::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LayoutDensity {
    Tiny,    // Too small for content (e.g., < 10x5). Show ID only.
    Compact, // Small (e.g., < 30x15). Hide labels, legends, borders.
    Normal,  // Full UI.
}

pub fn get_density(area: Rect) -> LayoutDensity {
    // Thresholds can be tuned based on your TUI's font aspect ratio
    if area.width < 20 || area.height < 6 {
        return LayoutDensity::Tiny;
    }

    if area.width < 45 || area.height < 15 {
        return LayoutDensity::Compact;
    }

    LayoutDensity::Normal
}
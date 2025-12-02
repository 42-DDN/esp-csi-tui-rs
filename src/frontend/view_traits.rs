// --- File: src/frontend/view_traits.rs ---
// --- Purpose: Traits to categorize views (Temporal vs Spatial) ---

use crate::layout_tree::ViewType;

pub trait ViewBehavior {
    fn is_temporal(&self) -> bool;
    fn is_spatial(&self) -> bool;
}

impl ViewBehavior for ViewType {
    fn is_temporal(&self) -> bool {
        match self {
            ViewType::Dashboard |
            ViewType::Spectrogram |
            ViewType::Phase => true,
            _ => false,
        }
    }

    fn is_spatial(&self) -> bool {
        match self {
            ViewType::Polar |
            ViewType::Isometric => true,
            _ => false,
        }
    }
}
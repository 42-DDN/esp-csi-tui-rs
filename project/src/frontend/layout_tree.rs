// --- File: src/layout_tree.rs ---
// --- Purpose: Defines the Tiling Window Manager (TWM) logic, pane splitting, and focus management ---

use ratatui::prelude::*;

// What can be inside a pane?
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ViewType {
    Empty,
    Dashboard,
    Polar,
    Isometric,
    Spectrogram,
    Phase,
    Camera,
}

impl ViewType {
    pub fn as_str(&self) -> &str {
        match self {
            ViewType::Empty => "Empty Pane",
            ViewType::Dashboard => "Dashboard Stats",
            ViewType::Polar => "Polar Scatter",
            ViewType::Isometric => "3D Isometric",
            ViewType::Spectrogram => "Spectrogram",
            ViewType::Phase => "Phase Plot",
            ViewType::Camera => "Camera Feed",
        }
    }
}

// The Tree Node
#[derive(Clone)]
pub enum LayoutNode {
    // A Leaf is a visible pane
    Pane {
        id: usize,
        view: ViewType
    },
    // A Split divides space for children
    Split {
        direction: Direction,
        ratio: u16, // usually 50%
        children: Vec<LayoutNode>, // In binary split, usually 2
    },
}

// The Manager
pub struct TilingManager {
    pub root: LayoutNode,
    pub focused_pane_id: usize,
    pub next_id: usize,
}

impl TilingManager {
    pub fn new() -> Self {
        Self {
            // Start with one empty pane
            root: LayoutNode::Pane { id: 0, view: ViewType::Empty },
            focused_pane_id: 0,
            next_id: 1,
        }
    }

    // SPLIT LOGIC
    // Recursively find the focused node and replace it with a Split
    pub fn split(&mut self, direction: Direction) {
        self.root = self.split_recursive(self.root.clone(), direction);
    }

    fn split_recursive(&mut self, node: LayoutNode, dir: Direction) -> LayoutNode {
        match node {
            LayoutNode::Pane { id, view } => {
                if id == self.focused_pane_id {
                    // FOUND IT: Create a split here
                    let new_id = self.next_id;
                    self.next_id += 1;

                    // The new pane starts as Empty
                    let new_pane = LayoutNode::Pane { id: new_id, view: ViewType::Empty };
                    let old_pane = LayoutNode::Pane { id, view };

                    // Switch focus to the new pane
                    self.focused_pane_id = new_id;

                    return LayoutNode::Split {
                        direction: dir,
                        ratio: 50,
                        children: vec![old_pane, new_pane], // Old one first, new one second
                    };
                }
                LayoutNode::Pane { id, view } // Not the focused one, return as is
            }
            LayoutNode::Split { direction, ratio, children } => {
                // Keep searching down the tree
                let new_children: Vec<LayoutNode> = children
                    .into_iter()
                    .map(|c| self.split_recursive(c, dir))
                    .collect();

                LayoutNode::Split { direction, ratio, children: new_children }
            }
        }
    }

    // SET VIEW LOGIC
    pub fn set_current_view(&mut self, new_view: ViewType) {
        self.root = self.set_view_recursive(self.root.clone(), new_view);
    }

    fn set_view_recursive(&self, node: LayoutNode, new_view: ViewType) -> LayoutNode {
        match node {
            LayoutNode::Pane { id, view } => {
                if id == self.focused_pane_id {
                    LayoutNode::Pane { id, view: new_view }
                } else {
                    LayoutNode::Pane { id, view }
                }
            }
            LayoutNode::Split { direction, ratio, children } => {
                let new_children = children.into_iter().map(|c| self.set_view_recursive(c, new_view)).collect();
                LayoutNode::Split { direction, ratio, children: new_children }
            }
        }
    }

    // NAVIGATION
    pub fn focus_next(&mut self) {
        // Simple sequential focus logic for Hackathon
        let start_id = self.focused_pane_id;
        let mut check_id = start_id + 1;
        let max_id = self.next_id;

        // Loop to find next valid pane ID
        for _ in 0..max_id {
            if check_id >= max_id { check_id = 0; }
            if self.node_exists(check_id, &self.root) {
                self.focused_pane_id = check_id;
                return;
            }
            check_id += 1;
        }
    }

    fn node_exists(&self, target_id: usize, node: &LayoutNode) -> bool {
        match node {
            LayoutNode::Pane { id, .. } => *id == target_id,
            LayoutNode::Split { children, .. } => {
                children.iter().any(|c| self.node_exists(target_id, c))
            }
        }
    }
}
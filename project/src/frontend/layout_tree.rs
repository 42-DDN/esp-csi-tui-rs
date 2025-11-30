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
            // Start with one Empty pane, ID 1
            root: LayoutNode::Pane { id: 1, view: ViewType::Empty },
            focused_pane_id: 1,
            next_id: 2,
        }
    }

    // SPLIT LOGIC
    // Recursively find the focused node and replace it with a Split
    pub fn split(&mut self, direction: Direction) {
        // LIMIT: Prevent creating more than 10 panes
        if self.get_pane_count() >= 10 {
            return;
        }
        self.root = self.split_recursive(self.root.clone(), direction);
    }

    fn split_recursive(&mut self, node: LayoutNode, dir: Direction) -> LayoutNode {
        match node {
            LayoutNode::Pane { id, view } => {
                if id == self.focused_pane_id {
                    let new_id = self.next_id;
                    self.next_id += 1;

                    // NEW PANE IS ALWAYS EMPTY
                    let new_pane = LayoutNode::Pane { id: new_id, view: ViewType::Empty };
                    let old_pane = LayoutNode::Pane { id, view };

                    // Switch focus to the new pane
                    self.focused_pane_id = new_id;

                    return LayoutNode::Split {
                        direction: dir,
                        ratio: 50,
                        children: vec![old_pane, new_pane],
                    };
                }
                LayoutNode::Pane { id, view }
            }
            LayoutNode::Split { direction, ratio, children } => {
                let new_children: Vec<LayoutNode> = children
                    .into_iter()
                    .map(|c| self.split_recursive(c, dir))
                    .collect();

                LayoutNode::Split { direction, ratio, children: new_children }
            }
        }
    }

    // CLOSE LOGIC
    pub fn close_focused_pane(&mut self) {
        // Don't close the last remaining pane
        if self.get_pane_count() <= 1 {
            return;
        }

        // 1. Remove the pane from the tree
        let removed_root = self.remove_recursive(self.root.clone(), self.focused_pane_id);

        // 2. Update root (unwrap safe because we checked count > 1)
        if let Some(node) = removed_root {
            self.root = node;
        }

        // 3. If focused ID is gone, reset focus to the first available pane
        if !self.node_exists(self.focused_pane_id, &self.root) {
            self.focused_pane_id = self.find_first_id(&self.root);
        }
    }

    // Returns Option<LayoutNode>. If None, the node is removed.
    fn remove_recursive(&self, node: LayoutNode, target_id: usize) -> Option<LayoutNode> {
        match node {
            LayoutNode::Pane { id, .. } => {
                if id == target_id {
                    None // Delete this pane
                } else {
                    Some(node) // Keep this pane
                }
            }
            LayoutNode::Split { direction, ratio, children } => {
                let mut new_children = Vec::new();
                for child in children {
                    if let Some(n) = self.remove_recursive(child, target_id) {
                        new_children.push(n);
                    }
                }

                if new_children.is_empty() {
                    return None;
                } else if new_children.len() == 1 {
                    // Split collapses to the single remaining child
                    return Some(new_children[0].clone());
                }

                Some(LayoutNode::Split { direction, ratio, children: new_children })
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
        let start_id = self.focused_pane_id;
        let mut check_id = start_id + 1;
        let max_id = self.next_id;

        for _ in 0..max_id {
            if check_id >= max_id { check_id = 1; } // Wrap back to 1
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

    // UTILITIES
    pub fn get_pane_count(&self) -> usize {
        self.count_recursive(&self.root)
    }

    fn count_recursive(&self, node: &LayoutNode) -> usize {
        match node {
            LayoutNode::Pane { .. } => 1,
            LayoutNode::Split { children, .. } => {
                children.iter().map(|c| self.count_recursive(c)).sum()
            }
        }
    }

    fn find_first_id(&self, node: &LayoutNode) -> usize {
        match node {
            LayoutNode::Pane { id, .. } => *id,
            LayoutNode::Split { children, .. } => self.find_first_id(&children[0]),
        }
    }
}
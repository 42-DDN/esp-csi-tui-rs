/*### Tiling Manager*/

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
            ViewType::Empty => "Empty Pane (Press Enter to Select)",
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
    // We need to recursively find the focused node and replace it with a Split
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

                    // Switch focus to the new pane? (Optional, let's say yes)
                    self.focused_pane_id = new_id;

                    return LayoutNode::Split {
                        direction: dir,
                        ratio: 50,
                        children: vec![old_pane, new_pane], // Old one first, new one second
                    };
                }
                LayoutNode::Pane { id, view } // Not the focused one
            }
            LayoutNode::Split { direction, ratio, mut children } => {
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

    // NAVIGATION (Simplified for brevity: Cycle through IDs)
    pub fn focus_next(&mut self) {
        // A real implementation needs tree traversal to find the "geometric next"
        // For hackathon speed, just cycle ID + 1 until we find a valid ID or wrap around.
        // (Placeholder logic)
    }
}
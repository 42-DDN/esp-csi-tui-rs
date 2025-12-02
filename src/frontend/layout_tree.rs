// --- File: src/layout_tree.rs ---
// --- Purpose: Defines the Tiling Window Manager (TWM) logic, pane splitting, and focus management ---

use ratatui::prelude::*;
use serde::{Serialize, Deserialize};
use crate::frontend::theme::ThemeType;

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

impl SplitDirection {
    pub fn to_ratatui(&self) -> Direction {
        match self {
            SplitDirection::Horizontal => Direction::Horizontal,
            SplitDirection::Vertical => Direction::Vertical,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum ViewType {
    Empty,
    Dashboard,
    Polar,
    Isometric,
    Spectrogram,
    Phase,
    Camera,
    RawScatter,
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
            ViewType::RawScatter => "Multipath Scatter",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum LayoutNode {
    Pane {
        id: usize,
        view: ViewType
    },
    Split {
        direction: SplitDirection,
        ratio: u16,
        children: Vec<LayoutNode>,
    },
}

impl LayoutNode {
    pub fn set_ratio_recursive(&mut self, path: &[usize], new_ratio: u16) {
        if path.is_empty() {
            if let LayoutNode::Split { ratio, .. } = self {
                *ratio = new_ratio.clamp(10, 90);
            }
            return;
        }
        if let LayoutNode::Split { children, .. } = self {
            let child_idx = path[0];
            if let Some(child) = children.get_mut(child_idx) {
                child.set_ratio_recursive(&path[1..], new_ratio);
            }
        }
    }

    pub fn adjust_ratio_recursive(&mut self, path: &[usize], delta: i16) {
        if path.is_empty() {
            if let LayoutNode::Split { ratio, .. } = self {
                let new_ratio = (*ratio as i16 + delta).clamp(10, 90);
                *ratio = new_ratio as u16;
            }
            return;
        }
        if let LayoutNode::Split { children, .. } = self {
            let child_idx = path[0];
            if let Some(child) = children.get_mut(child_idx) {
                child.adjust_ratio_recursive(&path[1..], delta);
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TilingManager {
    pub root: LayoutNode,
    pub focused_pane_id: usize,
    pub next_id: usize,

    #[serde(default)]
    pub is_default: bool,

    #[serde(default)]
    pub theme_variant: Option<ThemeType>,
}

impl TilingManager {
    pub fn new() -> Self {
        Self {
            root: LayoutNode::Pane { id: 1, view: ViewType::Empty },
            focused_pane_id: 1,
            next_id: 2,
            is_default: false,
            theme_variant: None,
        }
    }

    pub fn set_split_ratio(&mut self, path: &[usize], ratio: u16) {
        self.root.set_ratio_recursive(path, ratio);
    }

    pub fn adjust_split_ratio(&mut self, path: &[usize], delta: i16) {
        self.root.adjust_ratio_recursive(path, delta);
    }

    pub fn split(&mut self, direction: Direction) {
        if self.get_pane_count() >= 10 { return; }

        let local_dir = match direction {
            Direction::Horizontal => SplitDirection::Horizontal,
            Direction::Vertical => SplitDirection::Vertical,
        };

        self.root = self.split_recursive(self.root.clone(), local_dir);
    }

    fn split_recursive(&mut self, node: LayoutNode, dir: SplitDirection) -> LayoutNode {
        match node {
            LayoutNode::Pane { id, view } => {
                if id == self.focused_pane_id {
                    let new_id = self.next_id;
                    self.next_id += 1;
                    let new_pane = LayoutNode::Pane { id: new_id, view: ViewType::Empty };
                    let old_pane = LayoutNode::Pane { id, view };
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
                let new_children: Vec<LayoutNode> = children.into_iter().map(|c| self.split_recursive(c, dir)).collect();
                LayoutNode::Split { direction, ratio, children: new_children }
            }
        }
    }

    pub fn close_focused_pane(&mut self) {
        if self.get_pane_count() <= 1 { return; }
        let removed_root = self.remove_recursive(self.root.clone(), self.focused_pane_id);
        if let Some(node) = removed_root { self.root = node; }
        if !self.node_exists(self.focused_pane_id, &self.root) {
            self.focused_pane_id = self.find_first_id(&self.root);
        }
        self.reindex_ids();
    }

    fn reindex_ids(&mut self) {
        let mut counter = 1;
        let mut new_focus = self.focused_pane_id;
        self.root = self.reindex_recursive(self.root.clone(), &mut counter, &mut new_focus);
        self.focused_pane_id = new_focus;
        self.next_id = counter;
    }

    fn reindex_recursive(&self, node: LayoutNode, counter: &mut usize, new_focus: &mut usize) -> LayoutNode {
        match node {
            LayoutNode::Pane { id, view } => {
                let new_id = *counter;
                *counter += 1;
                if id == *new_focus { *new_focus = new_id; }
                LayoutNode::Pane { id: new_id, view }
            }
            LayoutNode::Split { direction, ratio, children } => {
                let new_children = children.into_iter().map(|c| self.reindex_recursive(c, counter, new_focus)).collect();
                LayoutNode::Split { direction, ratio, children: new_children }
            }
        }
    }

    fn remove_recursive(&self, node: LayoutNode, target_id: usize) -> Option<LayoutNode> {
        match node {
            LayoutNode::Pane { id, .. } => if id == target_id { None } else { Some(node) },
            LayoutNode::Split { direction, ratio, children } => {
                let mut new_children = Vec::new();
                for child in children {
                    if let Some(n) = self.remove_recursive(child, target_id) { new_children.push(n); }
                }
                if new_children.is_empty() { return None; }
                else if new_children.len() == 1 { return Some(new_children[0].clone()); }
                Some(LayoutNode::Split { direction, ratio, children: new_children })
            }
        }
    }

    pub fn set_current_view(&mut self, new_view: ViewType) {
        self.root = self.set_view_recursive(self.root.clone(), new_view);
    }

    fn set_view_recursive(&self, node: LayoutNode, new_view: ViewType) -> LayoutNode {
        match node {
            LayoutNode::Pane { id, view } => {
                if id == self.focused_pane_id { LayoutNode::Pane { id, view: new_view } } else { LayoutNode::Pane { id, view } }
            }
            LayoutNode::Split { direction, ratio, children } => {
                let new_children = children.into_iter().map(|c| self.set_view_recursive(c, new_view)).collect();
                LayoutNode::Split { direction, ratio, children: new_children }
            }
        }
    }

    pub fn focus_next(&mut self) {
        let start_id = self.focused_pane_id;
        let mut check_id = start_id + 1;
        let max_id = self.next_id;
        for _ in 0..max_id {
            if check_id >= max_id { check_id = 1; }
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
            LayoutNode::Split { children, .. } => children.iter().any(|c| self.node_exists(target_id, c))
        }
    }

    pub fn get_pane_count(&self) -> usize { self.count_recursive(&self.root) }
    fn count_recursive(&self, node: &LayoutNode) -> usize {
        match node {
            LayoutNode::Pane { .. } => 1,
            LayoutNode::Split { children, .. } => children.iter().map(|c| self.count_recursive(c)).sum()
        }
    }
    fn find_first_id(&self, node: &LayoutNode) -> usize {
        match node {
            LayoutNode::Pane { id, .. } => *id,
            LayoutNode::Split { children, .. } => self.find_first_id(&children[0]),
        }
    }
}
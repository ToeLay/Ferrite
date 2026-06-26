use crate::{DrawCommand, KeyEvent, Widget};
use ferrite_layout::{LayoutTree, NodeId, Rect};

pub struct App {
    tree: LayoutTree,
    root: Box<dyn Widget>,
    focused: Option<NodeId>,
    active_drag: Option<NodeId>,
    last_frame: Option<std::time::Instant>,
}

impl App {
    pub fn new(tree: LayoutTree, root: Box<dyn Widget>) -> Self {
        ferrite_reactive::animation::set_wake_up(crate::dirty::request_repaint);
        App { tree, root, focused: None, active_drag: None, last_frame: None }
    }

    pub fn layout_tree(&self) -> &LayoutTree { &self.tree }
    pub fn root_node_id(&self) -> NodeId { self.root.node_id() }
    pub fn root(&self) -> &dyn Widget { self.root.as_ref() }

    pub fn absolute_rect(&self, target: NodeId) -> Option<Rect> {
        fn walk(w: &dyn Widget, tree: &LayoutTree, ox: f32, oy: f32, target: NodeId) -> Option<Rect> {
            let r = tree.layout(w.node_id());
            let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
            if w.node_id() == target { return Some(abs); }
            for child in w.children() {
                if let Some(f) = walk(child.as_ref(), tree, abs.x, abs.y, target) { return Some(f); }
            }
            None
        }
        walk(self.root.as_ref(), &self.tree, 0.0, 0.0, target)
    }

    pub fn render(&mut self, width: f32, height: f32) -> Vec<DrawCommand> {
        let now = std::time::Instant::now();
        let dt = if let Some(last) = self.last_frame {
            now.duration_since(last).as_secs_f32()
        } else {
            1.0 / 60.0
        };
        self.last_frame = Some(now);

        // Tick animations (this might request repaint if animations are still running)
        ferrite_reactive::animation::tick_animations(dt);

        self.root.update(&mut self.tree);

        let dirty_nodes = crate::dirty::take_dirty_nodes();
        for node in dirty_nodes {
            self.tree.mark_dirty(node);
        }
        self.tree.compute(self.root.node_id(), width, height);
        let mut out = Vec::new();
        self.root.paint(&self.tree, 0.0, 0.0, &mut out);
        out
    }

    pub fn click(&mut self, x: f32, y: f32) -> bool {
        let new_focus = self.root.find_focusable_at(&self.tree, 0.0, 0.0, x, y);
        if new_focus != self.focused {
            if let Some(old) = self.focused { self.root.dispatch_focus(old, false); }
            if let Some(new) = new_focus  { self.root.dispatch_focus(new, true);  }
            self.focused = new_focus;
        }
        let clicked = self.root.click_at(&self.tree, 0.0, 0.0, x, y);
        self.active_drag = clicked;
        clicked.is_some()
    }

    pub fn double_click(&mut self, x: f32, y: f32) -> bool {
        let clicked = self.root.double_click_at(&self.tree, 0.0, 0.0, x, y);
        // Cancel active drag on double click so small mouse jitters
        // don't immediately override the word selection with a normal drag.
        self.active_drag = None;
        clicked.is_some()
    }

    pub fn triple_click(&mut self, x: f32, y: f32) -> bool {
        let clicked = self.root.triple_click_at(&self.tree, 0.0, 0.0, x, y);
        self.active_drag = None;
        clicked.is_some()
    }

    pub fn drag(&mut self, x: f32, y: f32) -> bool {
        if let Some(target) = self.active_drag {
            self.root.dispatch_drag(target, &self.tree, 0.0, 0.0, x, y)
        } else {
            false
        }
    }

    pub fn release_drag(&mut self) {
        self.active_drag = None;
    }

    pub fn scroll(&mut self, px: f32, py: f32, dx: f32, dy: f32) -> bool {
        self.root.scroll_at(&self.tree, 0.0, 0.0, px, py, dx, dy)
    }

    pub fn key_event(&mut self, event: KeyEvent) -> bool {
        if let Some(focused) = self.focused {
            self.root.dispatch_key(focused, &event)
        } else {
            false
        }
    }

    pub fn blur(&mut self) {
        if let Some(old) = self.focused.take() {
            self.root.dispatch_focus(old, false);
        }
    }

    pub fn collect_focusable(&self) -> Vec<NodeId> {
        fn walk(w: &dyn crate::widget::Widget, out: &mut Vec<NodeId>) {
            if w.is_focusable() { out.push(w.node_id()); }
            for child in w.children() { walk(child.as_ref(), out); }
        }
        let mut out = Vec::new();
        walk(self.root.as_ref(), &mut out);
        out
    }

    pub fn focus_next(&mut self) {
        let nodes = self.collect_focusable();
        if nodes.is_empty() { return; }
        let next = match self.focused {
            None => nodes[0],
            Some(cur) => {
                let idx = nodes.iter().position(|&n| n == cur).unwrap_or(0);
                nodes[(idx + 1) % nodes.len()]
            }
        };
        if let Some(old) = self.focused { self.root.dispatch_focus(old, false); }
        self.root.dispatch_focus(next, true);
        self.focused = Some(next);
        crate::dirty::request_repaint();
    }

    pub fn focus_prev(&mut self) {
        let nodes = self.collect_focusable();
        if nodes.is_empty() { return; }
        let next = match self.focused {
            None => nodes[nodes.len() - 1],
            Some(cur) => {
                let idx = nodes.iter().position(|&n| n == cur).unwrap_or(0);
                nodes[(idx + nodes.len() - 1) % nodes.len()]
            }
        };
        if let Some(old) = self.focused { self.root.dispatch_focus(old, false); }
        self.root.dispatch_focus(next, true);
        self.focused = Some(next);
        crate::dirty::request_repaint();
    }
}

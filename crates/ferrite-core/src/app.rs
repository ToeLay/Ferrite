use crate::{DrawCommand, KeyEvent, Widget};
use ferrite_layout::{LayoutTree, NodeId, Rect};

pub struct App {
    tree: LayoutTree,
    root: Box<dyn Widget>,
    focused: Option<NodeId>,
}

impl App {
    pub fn new(tree: LayoutTree, root: Box<dyn Widget>) -> Self {
        App { tree, root, focused: None }
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
        self.root.click_at(&self.tree, 0.0, 0.0, x, y)
    }

    pub fn drag(&mut self, x: f32, y: f32) -> bool {
        self.root.drag_at(&self.tree, 0.0, 0.0, x, y)
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
}

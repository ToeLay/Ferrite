use crate::{DrawCommand, KeyEvent};
use ferrite_layout::{LayoutTree, NodeId, Rect};

/// Every widget owns exactly one layout node and knows how to paint itself,
/// respond to clicks, and (optionally) receive keyboard focus and key events.
///
/// Almost every method here has a sensible default. A leaf widget only needs
/// `node_id` + `paint_self`; a container only needs `node_id` + `children`/
/// `children_mut`. Focus and keyboard are opt-in via `is_focusable` + `on_key`.
pub trait Widget {
    fn node_id(&self) -> NodeId;

    fn children(&self) -> &[Box<dyn Widget>] { &[] }
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] { &mut [] }

    fn paint_self(&self, _rect: Rect, _out: &mut Vec<DrawCommand>) {}

    fn on_click(&mut self) -> bool { false }

    /// Whether this widget can receive keyboard focus (default: false).
    fn is_focusable(&self) -> bool { false }

    /// Called when focus is gained (`focused = true`) or lost (`false`).
    fn on_focus_change(&mut self, _focused: bool) {}

    /// Handle a key event. Only called when this widget has focus.
    /// Return `true` to consume the event.
    fn on_key(&mut self, _event: &KeyEvent) -> bool { false }

    /// Optional tooltip text to display when the widget is hovered.
    fn tooltip(&self) -> Option<&str> { None }

    /// Optional signal to track hover state.
    fn hover_signal(&self) -> Option<ferrite_reactive::Signal<bool>> { None }

    /// Optional signal to track press state.
    fn press_signal(&self) -> Option<ferrite_reactive::Signal<bool>> { None }

    /// Called before layout to process reactive state changes or dynamic children
    fn update(&mut self, _tree: &mut LayoutTree) {}

    /// Called when the widget is permanently removed from the tree
    fn destroy(&mut self, tree: &mut LayoutTree) {
        for child in self.children_mut() {
            child.destroy(tree);
        }
        tree.remove(self.node_id());
    }


    fn paint(&self, tree: &LayoutTree, ox: f32, oy: f32, out: &mut Vec<DrawCommand>) {
        let r = tree.layout(self.node_id());
        let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        self.paint_self(abs, out);
        for child in self.children() { child.paint(tree, abs.x, abs.y, out); }
    }

    fn click_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<NodeId> {
        let r = tree.layout(self.node_id());
        let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        
        for child in self.children_mut().iter_mut().rev() {
            if let Some(clicked) = child.click_at(tree, abs.x, abs.y, px, py) { return Some(clicked); }
        }
        
        if px < abs.x || py < abs.y || px > abs.x + abs.width || py > abs.y + abs.height { return None; }
        let clicked = self.on_click();
        if clicked { Some(self.node_id()) } else { None }
    }

    fn double_click_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<NodeId> {
        let r = tree.layout(self.node_id());
        let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        
        for child in self.children_mut().iter_mut().rev() {
            if let Some(clicked) = child.double_click_at(tree, abs.x, abs.y, px, py) { return Some(clicked); }
        }
        
        if px < abs.x || py < abs.y || px > abs.x + abs.width || py > abs.y + abs.height { return None; }
        if self.on_double_click() { Some(self.node_id()) } else { None }
    }

    fn on_double_click(&mut self) -> bool { self.on_click() }

    fn triple_click_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<NodeId> {
        let r = tree.layout(self.node_id());
        let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        
        for child in self.children_mut().iter_mut().rev() {
            if let Some(clicked) = child.triple_click_at(tree, abs.x, abs.y, px, py) { return Some(clicked); }
        }
        
        if px < abs.x || py < abs.y || px > abs.x + abs.width || py > abs.y + abs.height { return None; }
        if self.on_triple_click() { Some(self.node_id()) } else { None }
    }

    fn on_triple_click(&mut self) -> bool { self.on_double_click() }

    fn drag_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> bool {
        let r = tree.layout(self.node_id());
        let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        
        for child in self.children_mut().iter_mut().rev() {
            if child.drag_at(tree, abs.x, abs.y, px, py) { return true; }
        }
        
        if px < abs.x || py < abs.y || px > abs.x + abs.width || py > abs.y + abs.height { return false; }
        false
    }

    fn dispatch_drag(&mut self, target: NodeId, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> bool {
        if self.node_id() == target {
            return true;
        }
        let r = tree.layout(self.node_id());
        let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        for child in self.children_mut().iter_mut().rev() {
            if child.dispatch_drag(target, tree, abs.x, abs.y, px, py) { return true; }
        }
        false
    }

    fn scroll_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32, dx: f32, dy: f32) -> bool {
        let r = tree.layout(self.node_id());
        let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        
        for child in self.children_mut().iter_mut().rev() {
            if child.scroll_at(tree, abs.x, abs.y, px, py, dx, dy) { return true; }
        }
        
        if px < abs.x || py < abs.y || px > abs.x + abs.width || py > abs.y + abs.height { return false; }
        self.on_scroll(dx, dy)
    }

    fn on_scroll(&mut self, _dx: f32, _dy: f32) -> bool { false }

    fn find_focusable_at(&self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<NodeId> {
        let r = tree.layout(self.node_id());
        let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        
        for child in self.children().iter().rev() {
            if let Some(focused) = child.find_focusable_at(tree, abs.x, abs.y, px, py) { return Some(focused); }
        }
        
        if px < abs.x || py < abs.y || px > abs.x + abs.width || py > abs.y + abs.height { return None; }
        if self.is_focusable() { Some(self.node_id()) } else { None }
    }

    fn dispatch_focus(&mut self, target: NodeId, focused: bool) {
        if self.node_id() == target { self.on_focus_change(focused); return; }
        for child in self.children_mut() { child.dispatch_focus(target, focused); }
    }

    fn dispatch_key(&mut self, target: NodeId, event: &KeyEvent) -> bool {
        if self.node_id() == target { return self.on_key(event); }
        for child in self.children_mut() {
            if child.dispatch_key(target, event) { return true; }
        }
        false
    }
}

use crate::{DrawCommand, Widget};
use ferrite_layout::{LayoutTree, NodeId, Rect};

/// The whole UI: a layout tree and the widget tree that owns nodes in it.
/// This is intentionally the entire "application" concept in `ferrite-core` —
/// windowing, event loops, and presenting pixels are a windowing backend's
/// job (see `ferrite-window`), not this crate's.
pub struct App {
    tree: LayoutTree,
    root: Box<dyn Widget>,
}

impl App {
    pub fn new(tree: LayoutTree, root: Box<dyn Widget>) -> Self {
        App { tree, root }
    }

    /// Read access to the underlying layout tree — useful for tests and
    /// tooling that need a widget's resolved rect (e.g. to simulate a click
    /// at its center) without `ferrite-core` having to expose a bespoke
    /// "find widget by path" API.
    pub fn layout_tree(&self) -> &LayoutTree {
        &self.tree
    }

    /// The root widget's own node id, the starting point for any layout query.
    pub fn root_node_id(&self) -> NodeId {
        self.root.node_id()
    }

    /// Find a widget's resolved, absolute (window-space) rect by its layout
    /// node id, by walking the tree the same way `render`/`click` do. Meant
    /// for tests and UI-automation tooling — "click the button" shouldn't
    /// require hand-deriving flexbox math.
    pub fn absolute_rect(&self, target: NodeId) -> Option<Rect> {
        fn walk(widget: &dyn Widget, tree: &LayoutTree, ox: f32, oy: f32, target: NodeId) -> Option<Rect> {
            let r = tree.layout(widget.node_id());
            let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
            if widget.node_id() == target {
                return Some(abs);
            }
            for child in widget.children() {
                if let Some(found) = walk(child.as_ref(), tree, abs.x, abs.y, target) {
                    return Some(found);
                }
            }
            None
        }
        walk(self.root.as_ref(), &self.tree, 0.0, 0.0, target)
    }

    /// Recompute layout for the given viewport size, then walk the widget
    /// tree into a flat draw command list a renderer backend can consume.
    pub fn render(&mut self, width: f32, height: f32) -> Vec<DrawCommand> {
        self.tree.compute(self.root.node_id(), width, height);
        let mut out = Vec::new();
        self.root.paint(&self.tree, 0.0, 0.0, &mut out);
        out
    }

    /// Dispatch a click at the given window-space point. Returns whether any
    /// widget handled it. Assumes `render` was already called at least once
    /// for the current size (layout must be up to date for hit-testing to be correct).
    pub fn click(&mut self, x: f32, y: f32) -> bool {
        self.root.click_at(&self.tree, 0.0, 0.0, x, y)
    }
}

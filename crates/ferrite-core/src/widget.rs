use crate::DrawCommand;
use ferrite_layout::{LayoutTree, NodeId, Rect};

/// Every widget owns exactly one node in the shared [`LayoutTree`] (containers
/// own a node with children attached; leaves own a childless node) and knows
/// how to paint itself and respond to a click.
///
/// Almost everything here has a default implementation. A leaf widget like
/// [`crate::widgets::Text`] only needs to implement `node_id` and `paint_self`.
/// A container only needs `node_id`, `children`, and `children_mut`. The walk
/// — turning the tree into draw commands, or finding which widget a click
/// landed on — is written once, here, instead of once per widget.
pub trait Widget {
    /// This widget's node in the layout tree, used to look up its resolved box.
    fn node_id(&self) -> NodeId;

    /// Sub-widgets, in paint order (back to front). Leaves return `&[]`.
    fn children(&self) -> &[Box<dyn Widget>] {
        &[]
    }
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        &mut []
    }

    /// Emit this widget's own draw commands (not its children's) given its
    /// already-resolved absolute rect. Default: draw nothing (a plain layout
    /// container with no background, for instance).
    fn paint_self(&self, _rect: Rect, _out: &mut Vec<DrawCommand>) {}

    /// Called when a click lands inside this widget and no child consumed it
    /// first. Return `true` if handled. Default: not interactive.
    fn on_click(&mut self) -> bool {
        false
    }

    /// Walk this widget and its children, resolving each one's absolute
    /// position (layout boxes are parent-relative, so positions accumulate
    /// as we descend) and appending draw commands in paint order.
    fn paint(&self, tree: &LayoutTree, origin_x: f32, origin_y: f32, out: &mut Vec<DrawCommand>) {
        let r = tree.layout(self.node_id());
        let abs = Rect { x: origin_x + r.x, y: origin_y + r.y, width: r.width, height: r.height };
        self.paint_self(abs, out);
        for child in self.children() {
            child.paint(tree, abs.x, abs.y, out);
        }
    }

    /// Hit-test a point (in the same absolute space as `paint`'s output)
    /// against this widget and its children, topmost (last-painted) first.
    /// Returns `true` as soon as something handles it.
    fn click_at(&mut self, tree: &LayoutTree, origin_x: f32, origin_y: f32, px: f32, py: f32) -> bool {
        let r = tree.layout(self.node_id());
        let abs_x = origin_x + r.x;
        let abs_y = origin_y + r.y;
        let inside = px >= abs_x && py >= abs_y && px <= abs_x + r.width && py <= abs_y + r.height;
        if !inside {
            return false;
        }
        for child in self.children_mut().iter_mut().rev() {
            if child.click_at(tree, abs_x, abs_y, px, py) {
                return true;
            }
        }
        self.on_click()
    }
}

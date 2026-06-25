//! Layout for Ferrite, built directly on [`taffy`]'s flexbox implementation
//! rather than reinventing one. Taffy is a solved problem done well; what
//! Ferrite adds is a [`Style`] type expressed in terms a widget author
//! actually writes (logical pixels, `Option<f32>` for "unset") instead of
//! taffy's CSS-shaped types, plus a `NodeId` that round-trips cleanly through
//! the widget tree above it.

use taffy::prelude::{auto, length, percent};
use taffy::{AvailableSpace, Dimension, FlexDirection as TaffyFlexDirection, TaffyTree};
use std::rc::Rc;
use std::cell::RefCell;

/// Which axis children are laid out along.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Row,
    Column,
}

impl From<Direction> for TaffyFlexDirection {
    fn from(d: Direction) -> Self {
        match d {
            Direction::Row => TaffyFlexDirection::Row,
            Direction::Column => TaffyFlexDirection::Column,
        }
    }
}

/// A size value: either an exact pixel amount, a percentage of the parent, or
/// left to the layout algorithm to figure out from content/flex.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Size {
    #[default]
    Auto,
    Px(f32),
    Percent(f32),
}

impl From<Size> for Dimension {
    fn from(s: Size) -> Self {
        match s {
            Size::Auto => auto(),
            Size::Px(v) => length(v),
            Size::Percent(v) => percent(v / 100.0),
        }
    }
}

/// Edge values (padding/margin/gap), in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Edges {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Edges {
    pub fn all(v: f32) -> Self {
        Edges { top: v, right: v, bottom: v, left: v }
    }
    pub fn symmetric(horizontal: f32, vertical: f32) -> Self {
        Edges { top: vertical, right: horizontal, bottom: vertical, left: horizontal }
    }
}

/// Controls how overflowing content is handled.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
    Scroll,
}

impl From<Overflow> for taffy::Overflow {
    fn from(o: Overflow) -> Self {
        match o {
            Overflow::Visible => taffy::Overflow::Visible,
            Overflow::Hidden => taffy::Overflow::Hidden,
            Overflow::Scroll => taffy::Overflow::Scroll,
        }
    }
}

/// The subset of flexbox a widget actually needs to set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub direction: Direction,
    pub width: Size,
    pub height: Size,
    pub padding: Edges,
    pub margin: Edges,
    /// Gap between children along the main axis, in logical pixels.
    pub gap: f32,
    /// How eagerly this node grows to fill leftover space in its parent.
    pub flex_grow: f32,
    /// How eagerly this node shrinks when its parent is too small.
    pub flex_shrink: f32,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub overflow_x: Overflow,
    pub overflow_y: Overflow,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            // Ferrite defaults to column layout, not row: app UI is
            // overwhelmingly "stack of things top to bottom" (forms, lists,
            // panels), whereas CSS's row default exists for inline text flow,
            // which doesn't apply here. Widgets that want a row opt in explicitly.
            direction: Direction::Column,
            width: Size::Auto,
            height: Size::Auto,
            padding: Edges::default(),
            margin: Edges::default(),
            gap: 0.0,
            flex_grow: 0.0,
            flex_shrink: 1.0,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            overflow_x: Overflow::Visible,
            overflow_y: Overflow::Visible,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum JustifyContent {
    #[default]
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AlignItems {
    Start,
    End,
    Center,
    #[default]
    Stretch,
}

fn to_taffy_style(s: &Style) -> taffy::Style {
    use taffy::{AlignItems as TAlign, JustifyContent as TJustify, Rect};

    taffy::Style {
        display: taffy::Display::Flex,
        flex_direction: s.direction.into(),
        size: taffy::Size { width: s.width.into(), height: s.height.into() },
        padding: Rect {
            left: length(s.padding.left),
            right: length(s.padding.right),
            top: length(s.padding.top),
            bottom: length(s.padding.bottom),
        },
        margin: Rect {
            left: length(s.margin.left),
            right: length(s.margin.right),
            top: length(s.margin.top),
            bottom: length(s.margin.bottom),
        },
        gap: taffy::Size { width: length(s.gap), height: length(s.gap) },
        overflow: taffy::Point { x: s.overflow_x.into(), y: s.overflow_y.into() },
        flex_grow: s.flex_grow,
        flex_shrink: s.flex_shrink,
        justify_content: Some(match s.justify_content {
            JustifyContent::Start => TJustify::Start,
            JustifyContent::End => TJustify::End,
            JustifyContent::Center => TJustify::Center,
            JustifyContent::SpaceBetween => TJustify::SpaceBetween,
            JustifyContent::SpaceAround => TJustify::SpaceAround,
        }),
        align_items: Some(match s.align_items {
            AlignItems::Start => TAlign::Start,
            AlignItems::End => TAlign::End,
            AlignItems::Center => TAlign::Center,
            AlignItems::Stretch => TAlign::Stretch,
        }),
        ..Default::default()
    }
}

/// A resolved, post-layout box in logical pixels, relative to its parent's
/// content area.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// Handle to a node in a [`LayoutTree`]. Just a thin wrapper around taffy's
/// own id so the rest of Ferrite doesn't take a direct dependency on taffy's types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(taffy::NodeId);

pub enum NodeKind {
    Text { content: Rc<RefCell<String>>, font_size: f32 },
    Other,
}

/// Owns the flexbox tree and the cached result of the last `compute` call.
pub struct LayoutTree {
    inner: TaffyTree<NodeKind>,
    measure_fn: Option<Box<dyn Fn(&str, f32, Option<f32>) -> (f32, f32)>>,
    hooks: Vec<Box<dyn Fn() -> Vec<(NodeId, Style)>>>,
}

impl Default for LayoutTree {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutTree {
    pub fn new() -> Self {
        Self {
            inner: TaffyTree::new(),
            measure_fn: None,
            hooks: Vec::new(),
        }
    }

    pub fn set_text_measure(&mut self, f: impl Fn(&str, f32, Option<f32>) -> (f32, f32) + 'static) {
        self.measure_fn = Some(Box::new(f));
    }

    pub fn add_pre_layout_hook(&mut self, hook: impl Fn() -> Vec<(NodeId, Style)> + 'static) {
        self.hooks.push(Box::new(hook));
    }

    pub fn new_leaf(&mut self, style: Style) -> NodeId {
        NodeId(self.inner.new_leaf_with_context(to_taffy_style(&style), NodeKind::Other).expect("ferrite-layout: arena allocation failed"))
    }

    pub fn new_text_leaf(&mut self, style: Style, content: Rc<RefCell<String>>, font_size: f32) -> NodeId {
        NodeId(self.inner.new_leaf_with_context(to_taffy_style(&style), NodeKind::Text { content, font_size }).expect("ferrite-layout: arena allocation failed"))
    }

    pub fn new_with_children(&mut self, style: Style, children: &[NodeId]) -> NodeId {
        let children: Vec<taffy::NodeId> = children.iter().map(|n| n.0).collect();
        NodeId(
            self.inner
                .new_with_children(to_taffy_style(&style), &children)
                .expect("ferrite-layout: arena allocation failed"),
        )
    }

    pub fn set_style(&mut self, node: NodeId, style: Style) {
        self.inner
            .set_style(node.0, to_taffy_style(&style))
            .expect("ferrite-layout: set_style on missing node");
    }

    pub fn set_children(&mut self, node: NodeId, children: &[NodeId]) {
        let children: Vec<taffy::NodeId> = children.iter().map(|n| n.0).collect();
        self.inner
            .set_children(node.0, &children)
            .expect("ferrite-layout: set_children on missing node");
    }

    pub fn mark_dirty(&mut self, node: NodeId) {
        let _ = self.inner.mark_dirty(node.0);
    }

    /// Run the flexbox algorithm with the given viewport/container size, in
    /// logical pixels. Call this once per frame (or once per signal-driven
    /// layout invalidation, via the effect that wraps your render loop).
    pub fn compute(&mut self, root: NodeId, available_width: f32, available_height: f32) {
        let changes: Vec<(NodeId, Style)> = self.hooks.iter().flat_map(|h| h()).collect();
        for (node, style) in changes {
            self.set_style(node, style);
        }

        let size = taffy::Size {
            width: AvailableSpace::Definite(available_width),
            height: AvailableSpace::Definite(available_height),
        };

        if let Some(ref measure) = self.measure_fn {
            let measure_fn = measure.as_ref();
            self.inner.compute_layout_with_measure(root.0, size, |known, available, _, ctx, _| {
                match ctx {
                    Some(NodeKind::Text { content, font_size }) => {
                        let max_w = known.width.or(available.width.into_option());
                        let (w, h) = measure_fn(&content.borrow(), *font_size, max_w);
                        taffy::Size { width: w, height: h }
                    }
                    _ => taffy::Size { width: 0.0, height: 0.0 },
                }
            }).expect("ferrite-layout: compute_layout_with_measure failed");
        } else {
            self.inner.compute_layout(root.0, size).expect("ferrite-layout: compute_layout failed");
        }
    }

    /// The resolved box for `node` after the last `compute` call. Coordinates
    /// are relative to the node's parent, matching taffy's convention — sum
    /// them up the tree if you need absolute screen coordinates.
    pub fn layout(&self, node: NodeId) -> Rect {
        let l = self.inner.layout(node.0).expect("ferrite-layout: layout() before compute()");
        Rect { x: l.location.x, y: l.location.y, width: l.size.width, height: l.size.height }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn column_stack_with_gap() {
        let mut tree = LayoutTree::new();
        let child_style = Style { height: Size::Px(20.0), ..Default::default() };
        let a = tree.new_leaf(child_style);
        let b = tree.new_leaf(child_style);
        let root = tree.new_with_children(
            Style { width: Size::Px(100.0), height: Size::Px(200.0), gap: 10.0, ..Default::default() },
            &[a, b],
        );
        tree.compute(root, 100.0, 200.0);

        let a_box = tree.layout(a);
        let b_box = tree.layout(b);
        assert_eq!(a_box.y, 0.0);
        assert_eq!(a_box.height, 20.0);
        assert_eq!(b_box.y, 30.0); // 20 (a's height) + 10 (gap)
        assert_eq!(b_box.width, 100.0); // stretch is the default align_items
    }

    #[test]
    fn flex_grow_fills_remaining_space() {
        let mut tree = LayoutTree::new();
        let fixed = tree.new_leaf(Style { height: Size::Px(50.0), ..Default::default() });
        let grow = tree.new_leaf(Style { flex_grow: 1.0, ..Default::default() });
        let root = tree.new_with_children(Style { height: Size::Px(200.0), ..Default::default() }, &[fixed, grow]);
        tree.compute(root, 100.0, 200.0);

        assert_eq!(tree.layout(fixed).height, 50.0);
        assert_eq!(tree.layout(grow).height, 150.0);
    }
}

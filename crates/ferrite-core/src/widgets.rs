use crate::{request_repaint, Color, DrawCommand, Widget};
use ferrite_layout::{Direction, LayoutTree, NodeId, Rect, Size, Style};
use std::cell::RefCell;
use std::rc::Rc;

/// A layout box that stacks its children and, optionally, paints a flat
/// background behind them. This is the only container type Ferrite ships
/// with — `column` and `row` are just `Container` with `direction` set,
/// the same way most CSS layouts are "a div with `flex-direction`" rather
/// than distinct element types.
pub struct Container {
    node: NodeId,
    children: Vec<Box<dyn Widget>>,
    background: Option<Color>,
}

impl Container {
    pub fn background(mut self, color: Color) -> Self {
        self.background = Some(color);
        self
    }
}

impl Widget for Container {
    fn node_id(&self) -> NodeId {
        self.node
    }
    fn children(&self) -> &[Box<dyn Widget>] {
        &self.children
    }
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        &mut self.children
    }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        if let Some(color) = self.background {
            out.push(DrawCommand::Rect { rect, color, corner_radius: 0.0 });
        }
    }
}

fn container(tree: &mut LayoutTree, mut style: Style, direction: Direction, children: Vec<Box<dyn Widget>>) -> Container {
    style.direction = direction;
    let ids: Vec<NodeId> = children.iter().map(|c| c.node_id()).collect();
    let node = tree.new_with_children(style, &ids);
    Container { node, children, background: None }
}

/// Children stacked top to bottom.
pub fn column(tree: &mut LayoutTree, style: Style, children: Vec<Box<dyn Widget>>) -> Container {
    container(tree, style, Direction::Column, children)
}

/// Children laid out left to right.
pub fn row(tree: &mut LayoutTree, style: Style, children: Vec<Box<dyn Widget>>) -> Container {
    container(tree, style, Direction::Row, children)
}

/// A run of text. Note on sizing: Ferrite v0.1 doesn't yet wire a text
/// shaper into taffy's measure-function hook (see ARCHITECTURE.md), so a
/// `Text` node's box is *estimated* from character count against the bundled
/// font's known monospace advance width, not measured from real glyph
/// metrics. It's accurate for the shipped font and single-line content;
/// don't rely on it once a proportional font or text wrapping enters the picture.
pub struct Text {
    node: NodeId,
    content: Rc<RefCell<String>>,
    color: Color,
    size: f32,
}

impl Text {
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// Sets the font size and re-derives this node's layout box from it.
    /// Takes `&mut LayoutTree` because, unlike `color`, this has to reach
    /// into layout — see the sizing note on [`Text`] for why it's an
    /// estimate rather than a measurement.
    pub fn font_size(mut self, tree: &mut LayoutTree, size: f32) -> Self {
        self.size = size;
        let char_count = self.content.borrow().chars().count() as f32;
        tree.set_style(self.node, text_node_style(char_count, size));
        self
    }
}

impl Widget for Text {
    fn node_id(&self) -> NodeId {
        self.node
    }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        out.push(DrawCommand::Text {
            x: rect.x,
            y: rect.y,
            content: self.content.borrow().clone(),
            size: self.size,
            color: self.color,
        });
    }
}

/// IBM Plex Mono's advance width is a fixed ~0.62em (it's monospace, so this
/// isn't an average, it's exact for the bundled font). Ferrite v0.1 doesn't
/// wire a text shaper into taffy's measure-function hook yet (see the
/// module-level doc comment), so this is the bridge: real per-glyph metrics
/// from `ferrite-render-skia`'s font would replace it once that hook exists,
/// without changing the `Text` API.
fn text_node_style(char_count: f32, size: f32) -> Style {
    Style { width: Size::Px(char_count * size * 0.62), height: Size::Px(size * 1.4), ..Default::default() }
}

fn new_text_node(tree: &mut LayoutTree, char_count: usize, size: f32) -> NodeId {
    tree.new_leaf(text_node_style(char_count as f32, size))
}

const DEFAULT_TEXT_SIZE: f32 = 16.0;

/// Static text that never changes.
pub fn text(tree: &mut LayoutTree, content: impl Into<String>) -> Text {
    let content = content.into();
    let node = new_text_node(tree, content.chars().count(), DEFAULT_TEXT_SIZE);
    Text { node, content: Rc::new(RefCell::new(content)), color: Color::BLACK, size: DEFAULT_TEXT_SIZE }
}

/// Text driven by a reactive computation. `f` re-runs whenever a signal it
/// reads changes (it's just an effect under the hood — see
/// `ferrite_reactive::create_effect`), and each re-run both updates this
/// node's content and requests a repaint, with no diffing or virtual tree
/// involved: the widget *is* the persistent state, the effect just keeps it current.
///
/// The layout box is sized from the *initial* call to `f`, not re-measured
/// on every change (see the sizing note above) — fine for "Count: 7" style
/// labels that stay roughly the same length, less fine for text whose length
/// swings wildly; call `.font_size()` again after a big known change if needed.
pub fn text_dyn(tree: &mut LayoutTree, f: impl Fn() -> String + 'static) -> Text {
    let initial = f();
    let node = new_text_node(tree, initial.chars().count(), DEFAULT_TEXT_SIZE);
    let content = Rc::new(RefCell::new(initial));
    let content2 = content.clone();
    ferrite_reactive::create_effect(move || {
        *content2.borrow_mut() = f();
        request_repaint();
    });
    Text { node, content, color: Color::BLACK, size: DEFAULT_TEXT_SIZE }
}

/// A clickable, labeled box. `on_click` runs on the click that lands inside
/// its bounds and isn't consumed by a child first (buttons are leaves, so in
/// practice: any click inside it).
pub struct Button {
    node: NodeId,
    label: String,
    on_click: Box<dyn FnMut()>,
    background: Color,
    foreground: Color,
}

impl Button {
    pub fn background(mut self, color: Color) -> Self {
        self.background = color;
        self
    }
    pub fn foreground(mut self, color: Color) -> Self {
        self.foreground = color;
        self
    }
}

impl Widget for Button {
    fn node_id(&self) -> NodeId {
        self.node
    }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        out.push(DrawCommand::Rect { rect, color: self.background, corner_radius: 10.0 });
        out.push(DrawCommand::Text {
            x: rect.x + 18.0,
            y: rect.y + rect.height / 2.0 - 9.0,
            content: self.label.clone(),
            size: 18.0,
            color: self.foreground,
        });
    }
    fn on_click(&mut self) -> bool {
        (self.on_click)();
        true
    }
}

pub fn button(tree: &mut LayoutTree, label: impl Into<String>, on_click: impl FnMut() + 'static) -> Button {
    let node = tree.new_leaf(Style { width: Size::Px(56.0), height: Size::Px(56.0), ..Default::default() });
    Button {
        node,
        label: label.into(),
        on_click: Box::new(on_click),
        background: Color::rgb(0.21, 0.43, 0.86),
        foreground: Color::WHITE,
    }
}

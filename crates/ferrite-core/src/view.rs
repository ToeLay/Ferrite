use crate::widgets::{self, Container, Text, Button, TextInput, Spacer, Divider, Checkbox, Slider, Scroll};
use crate::{request_repaint, Color, DrawCommand, Widget};
use ferrite_layout::{AlignItems, Direction, Edges, JustifyContent, LayoutTree, NodeId, Rect, Size, Style};
use ferrite_reactive::{create_effect, Signal};
use crate::theme::Theme;
use std::cell::RefCell;
use std::rc::Rc;

// ── StyleOverrides ───────────────────────────────────────────────────────────

#[derive(Default)]
struct StyleOverrides {
    width: Option<Size>,
    height: Option<Size>,
    padding: Option<Edges>,
    margin: Option<Edges>,
    gap: Option<f32>,
    flex_grow: Option<f32>,
    flex_shrink: Option<f32>,
    align: Option<AlignItems>,
    justify: Option<JustifyContent>,
    pub overflow_x: Option<ferrite_layout::Overflow>,
    pub overflow_y: Option<ferrite_layout::Overflow>,
    pub position_type: Option<ferrite_layout::PositionType>,
    pub inset: Option<ferrite_layout::Inset>,
}

impl StyleOverrides {
    fn apply_to(&self, style: &mut Style) {
        if let Some(w) = self.width { style.width = w; }
        if let Some(h) = self.height { style.height = h; }
        if let Some(p) = self.padding { style.padding = p; }
        if let Some(m) = self.margin { style.margin = m; }
        if let Some(g) = self.gap { style.gap = g; }
        if let Some(g) = self.flex_grow { style.flex_grow = g; }
        if let Some(s) = self.flex_shrink { style.flex_shrink = s; }
        if let Some(a) = self.align { style.align_items = a; }
        if let Some(j) = self.justify { style.justify_content = j; }
        if let Some(ox) = self.overflow_x { style.overflow_x = ox; }
        if let Some(oy) = self.overflow_y { style.overflow_y = oy; }
        if let Some(pt) = self.position_type { style.position_type = pt; }
        if let Some(i) = self.inset { style.inset = i; }
    }
}

// ── ViewDescriptor trait ─────────────────────────────────────────────────────

trait ViewDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget>;
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides;
    
    fn set_font_size(&mut self, _size: f32) {
        debug_assert!(false, "This widget does not support setting font size");
    }
    fn set_text_color(&mut self, _color: Color) {
        debug_assert!(false, "This widget does not support setting text color");
    }
    fn set_background(&mut self, _color: Color) {
        debug_assert!(false, "This widget does not support setting background");
    }
    fn set_foreground(&mut self, _color: Color) {
        debug_assert!(false, "This widget does not support setting foreground");
    }
    fn set_corner_radius(&mut self, _r: f32) {
        debug_assert!(false, "This widget does not support setting corner radius");
    }
}

// ── AnyView ──────────────────────────────────────────────────────────────────

pub struct AnyView {
    inner: Box<dyn ViewDescriptor>,
}

impl AnyView {
    pub fn build(self, tree: &mut LayoutTree) -> Box<dyn Widget> {
        self.inner.build(tree)
    }

    // Layout modifiers
    pub fn padding(mut self, p: f32) -> Self {
        self.inner.style_overrides_mut().padding = Some(Edges::all(p));
        self
    }
    pub fn padding_xy(mut self, h: f32, v: f32) -> Self {
        self.inner.style_overrides_mut().padding = Some(Edges::symmetric(h, v));
        self
    }
    pub fn margin(mut self, m: f32) -> Self {
        self.inner.style_overrides_mut().margin = Some(Edges::all(m));
        self
    }
    pub fn gap(mut self, g: f32) -> Self {
        self.inner.style_overrides_mut().gap = Some(g);
        self
    }
    pub fn width(mut self, w: f32) -> Self {
        self.inner.style_overrides_mut().width = Some(Size::Px(w));
        self
    }
    pub fn height_percent(mut self, h: f32) -> Self {
        self.inner.style_overrides_mut().height = Some(Size::Percent(h));
        self
    }
    
    pub fn position_type(mut self, pt: ferrite_layout::PositionType) -> Self {
        self.inner.style_overrides_mut().position_type = Some(pt);
        self
    }
    
    pub fn inset(mut self, inset: ferrite_layout::Inset) -> Self {
        self.inner.style_overrides_mut().inset = Some(inset);
        self
    }

    pub fn width_percent(mut self, w: f32) -> Self {
        self.inner.style_overrides_mut().width = Some(Size::Percent(w));
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.inner.style_overrides_mut().height = Some(Size::Px(h));
        self
    }
    pub fn fill(mut self) -> Self {
        self.inner.style_overrides_mut().width = Some(Size::Percent(100.0));
        self.inner.style_overrides_mut().height = Some(Size::Percent(100.0));
        self
    }
    pub fn flex_grow(mut self, g: f32) -> Self {
        self.inner.style_overrides_mut().flex_grow = Some(g);
        self
    }
    pub fn flex_shrink(mut self, s: f32) -> Self {
        self.inner.style_overrides_mut().flex_shrink = Some(s);
        self
    }
    pub fn align(mut self, a: AlignItems) -> Self {
        self.inner.style_overrides_mut().align = Some(a);
        self
    }
    pub fn justify(mut self, j: JustifyContent) -> Self {
        self.inner.style_overrides_mut().justify = Some(j);
        self
    }

    // Widget-specific modifiers
    pub fn size(mut self, s: f32) -> Self {
        self.inner.set_font_size(s);
        self
    }
    pub fn color(mut self, c: Color) -> Self {
        self.inner.set_text_color(c);
        self
    }
    pub fn background(mut self, c: Color) -> Self {
        self.inner.set_background(c);
        self
    }
    pub fn foreground(mut self, c: Color) -> Self {
        self.inner.set_foreground(c);
        self
    }
    pub fn corner_radius(mut self, r: f32) -> Self {
        self.inner.set_corner_radius(r);
        self
    }

    // Dynamic modifier
    pub fn visible_when(self, signal: impl Fn() -> bool + 'static) -> Self {
        AnyView {
            inner: Box::new(VisibleWhenDescriptor {
                inner: self,
                signal: Rc::new(signal),
                overrides: StyleOverrides::default(),
            }),
        }
    }
    
    pub fn tooltip(self, text: &str) -> Self {
        AnyView {
            inner: Box::new(TooltipDescriptor {
                inner: self,
                text: text.to_string(),
                overrides: StyleOverrides::default(),
            }),
        }
    }

    pub fn on_hover(self, callback: impl FnMut(bool) + 'static) -> Self {
        AnyView {
            inner: Box::new(TrackHoverDescriptor {
                inner: self,
                callback: Box::new(callback),
                overrides: StyleOverrides::default(),
            }),
        }
    }

    pub fn on_press(self, callback: impl FnMut(bool) + 'static) -> Self {
        AnyView {
            inner: Box::new(TrackPressDescriptor {
                inner: self,
                callback: Box::new(callback),
                overrides: StyleOverrides::default(),
            }),
        }
    }

    pub fn anchor(self, token: Anchor) -> Self {
        AnyView {
            inner: Box::new(AnchorDescriptor {
                inner: self,
                token,
                overrides: StyleOverrides::default(),
            }),
        }
    }
}

// ── View trait ───────────────────────────────────────────────────────────────

pub trait View {
    fn view(self) -> AnyView;
}

impl View for AnyView {
    fn view(self) -> AnyView { self }
}

// ── ViewIteratorExt ──────────────────────────────────────────────────────────

pub trait ViewIteratorExt: Iterator<Item = AnyView> + Sized {
    /// Collect views into a vertical column.
    fn collect_col(self) -> AnyView {
        col(self.collect::<Vec<_>>())
    }

    /// Collect views into a horizontal row.
    fn collect_row(self) -> AnyView {
        row(self.collect::<Vec<_>>())
    }

    /// Insert a separator view between each item.
    fn intersperse_with<F: FnMut() -> AnyView>(self, separator: F) -> IntersperseWith<Self, F> {
        IntersperseWith { iter: self.peekable(), separator, needs_sep: false }
    }
}

impl<I: Iterator<Item = AnyView> + Sized> ViewIteratorExt for I {}

pub struct IntersperseWith<I: Iterator, F> {
    iter: std::iter::Peekable<I>,
    separator: F,
    needs_sep: bool,
}

impl<I: Iterator<Item = AnyView>, F: FnMut() -> AnyView> Iterator for IntersperseWith<I, F> {
    type Item = AnyView;
    fn next(&mut self) -> Option<AnyView> {
        if self.needs_sep {
            if self.iter.peek().is_some() {
                self.needs_sep = false;
                return Some((self.separator)());
            }
        }
        self.iter.next().map(|item| {
            self.needs_sep = true;
            item
        })
    }
}

// ── ScopedWidget ─────────────────────────────────────────────────────────────

struct ScopedWidget {
    inner: Box<dyn crate::Widget>,
    scope: Option<ferrite_reactive::Scope>,
}

impl crate::Widget for ScopedWidget {
    fn node_id(&self) -> ferrite_layout::NodeId { self.inner.node_id() }
    fn children(&self) -> &[Box<dyn crate::Widget>] { self.inner.children() }
    fn children_mut(&mut self) -> &mut [Box<dyn crate::Widget>] { self.inner.children_mut() }
    fn paint_self(&self, rect: ferrite_layout::Rect, out: &mut Vec<crate::DrawCommand>) { self.inner.paint_self(rect, out) }
    fn on_click(&mut self) -> bool { self.inner.on_click() }
    fn is_focusable(&self) -> bool { self.inner.is_focusable() }
    fn on_focus_change(&mut self, focused: bool) { self.inner.on_focus_change(focused) }
    fn on_key(&mut self, event: &crate::KeyEvent) -> bool { self.inner.on_key(event) }
    fn tooltip(&self) -> Option<&str> { self.inner.tooltip() }
    fn hover_signal(&self) -> Option<ferrite_reactive::Signal<bool>> { self.inner.hover_signal() }
    fn press_signal(&self) -> Option<ferrite_reactive::Signal<bool>> { self.inner.press_signal() }
    fn update(&mut self, tree: &mut ferrite_layout::LayoutTree) { self.inner.update(tree) }
    fn destroy(&mut self, tree: &mut ferrite_layout::LayoutTree) {
        self.inner.destroy(tree);
        // The drop implementation handles scope disposal, but we could also do it here.
    }
    
    // We must forward default-provided methods to preserve behavior
    fn paint(&self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, out: &mut Vec<crate::DrawCommand>) {
        self.inner.paint(tree, ox, oy, out)
    }
    fn click_at(&mut self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<ferrite_layout::NodeId> {
        self.inner.click_at(tree, ox, oy, px, py)
    }
    fn double_click_at(&mut self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<ferrite_layout::NodeId> {
        self.inner.double_click_at(tree, ox, oy, px, py)
    }
    fn on_double_click(&mut self) -> bool { self.inner.on_double_click() }
    fn triple_click_at(&mut self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<ferrite_layout::NodeId> {
        self.inner.triple_click_at(tree, ox, oy, px, py)
    }
    fn on_triple_click(&mut self) -> bool { self.inner.on_triple_click() }
    fn drag_at(&mut self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> bool {
        self.inner.drag_at(tree, ox, oy, px, py)
    }
    fn dispatch_drag(&mut self, target: ferrite_layout::NodeId, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> bool {
        self.inner.dispatch_drag(target, tree, ox, oy, px, py)
    }
    fn scroll_at(&mut self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, px: f32, py: f32, dx: f32, dy: f32) -> bool {
        self.inner.scroll_at(tree, ox, oy, px, py, dx, dy)
    }
    fn find_focusable_at(&self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<ferrite_layout::NodeId> {
        self.inner.find_focusable_at(tree, ox, oy, px, py)
    }
    fn dispatch_focus(&mut self, target: ferrite_layout::NodeId, focused: bool) {
        self.inner.dispatch_focus(target, focused)
    }
    fn dispatch_key(&mut self, target: ferrite_layout::NodeId, event: &crate::KeyEvent) -> bool {
        self.inner.dispatch_key(target, event)
    }
}

impl Drop for ScopedWidget {
    fn drop(&mut self) {
        if let Some(scope) = self.scope.take() {
            scope.dispose();
        }
    }
}

// ── TooltipDescriptor ──────────────────────────────────────────────────────────

struct TooltipDescriptor {
    inner: AnyView,
    text: String,
    overrides: StyleOverrides,
}

struct TooltipWidget {
    node: ferrite_layout::NodeId,
    child: Box<dyn crate::Widget>,
    text: String,
}

impl crate::Widget for TooltipWidget {
    fn node_id(&self) -> ferrite_layout::NodeId { self.node }
    fn children(&self) -> &[Box<dyn crate::Widget>] { std::slice::from_ref(&self.child) }
    fn children_mut(&mut self) -> &mut [Box<dyn crate::Widget>] { std::slice::from_mut(&mut self.child) }
    fn tooltip(&self) -> Option<&str> { Some(&self.text) }
    fn update(&mut self, tree: &mut ferrite_layout::LayoutTree) {
        self.child.update(tree);
    }
    fn paint(&self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, out: &mut Vec<crate::DrawCommand>) {
        let r = tree.layout(self.node);
        let abs = ferrite_layout::Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        out.push(crate::DrawCommand::TooltipRegion {
            rect: abs,
            text: self.text.clone(),
        });
        self.child.paint(tree, ox + r.x, oy + r.y, out);
    }
}

impl ViewDescriptor for TooltipDescriptor {
    fn build(self: Box<Self>, tree: &mut ferrite_layout::LayoutTree) -> Box<dyn crate::Widget> {
        let TooltipDescriptor { inner, text, overrides } = *self;
        let child = inner.build(tree);
        let mut style = ferrite_layout::Style::default();
        overrides.apply_to(&mut style);
        let node = tree.new_with_children(style, &[child.node_id()]);
        Box::new(TooltipWidget { node, child, text })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
}

// ── TrackHoverDescriptor ───────────────────────────────────────────────────────

struct TrackHoverDescriptor {
    inner: AnyView,
    callback: Box<dyn FnMut(bool)>,
    overrides: StyleOverrides,
}

struct TrackHoverWidget {
    node: ferrite_layout::NodeId,
    child: Box<dyn crate::Widget>,
    signal: Signal<bool>,
}

impl crate::Widget for TrackHoverWidget {
    fn node_id(&self) -> ferrite_layout::NodeId { self.node }
    fn children(&self) -> &[Box<dyn crate::Widget>] { std::slice::from_ref(&self.child) }
    fn children_mut(&mut self) -> &mut [Box<dyn crate::Widget>] { std::slice::from_mut(&mut self.child) }
    fn hover_signal(&self) -> Option<Signal<bool>> { Some(self.signal) }
    fn update(&mut self, tree: &mut ferrite_layout::LayoutTree) { self.child.update(tree); }
    fn paint(&self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, out: &mut Vec<crate::DrawCommand>) {
        let r = tree.layout(self.node);
        self.child.paint(tree, ox + r.x, oy + r.y, out);
    }
}

impl ViewDescriptor for TrackHoverDescriptor {
    fn build(mut self: Box<Self>, tree: &mut ferrite_layout::LayoutTree) -> Box<dyn crate::Widget> {
        let scope = ferrite_reactive::Scope::new();
        let widget = scope.run(|| {
            let signal = ferrite_reactive::create_signal(false);
            let mut callback = self.callback;
            ferrite_reactive::create_effect(move || {
                if let Some(val) = signal.try_get() {
                    callback(val);
                }
            });
            let TrackHoverDescriptor { inner, overrides, .. } = *self;
            let child = inner.build(tree);
            let mut style = ferrite_layout::Style::default();
            overrides.apply_to(&mut style);
            let node = tree.new_with_children(style, &[child.node_id()]);
            Box::new(TrackHoverWidget { node, child, signal }) as Box<dyn crate::Widget>
        });
        Box::new(ScopedWidget { inner: widget, scope: Some(scope) })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
}

// ── TrackPressDescriptor ───────────────────────────────────────────────────────

struct TrackPressDescriptor {
    inner: AnyView,
    callback: Box<dyn FnMut(bool)>,
    overrides: StyleOverrides,
}

struct TrackPressWidget {
    node: ferrite_layout::NodeId,
    child: Box<dyn crate::Widget>,
    signal: Signal<bool>,
}

impl crate::Widget for TrackPressWidget {
    fn node_id(&self) -> ferrite_layout::NodeId { self.node }
    fn children(&self) -> &[Box<dyn crate::Widget>] { std::slice::from_ref(&self.child) }
    fn children_mut(&mut self) -> &mut [Box<dyn crate::Widget>] { std::slice::from_mut(&mut self.child) }
    fn press_signal(&self) -> Option<Signal<bool>> { Some(self.signal) }
    fn update(&mut self, tree: &mut ferrite_layout::LayoutTree) { self.child.update(tree); }
    fn paint(&self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, out: &mut Vec<crate::DrawCommand>) {
        let r = tree.layout(self.node);
        self.child.paint(tree, ox + r.x, oy + r.y, out);
    }
}

impl ViewDescriptor for TrackPressDescriptor {
    fn build(mut self: Box<Self>, tree: &mut ferrite_layout::LayoutTree) -> Box<dyn crate::Widget> {
        let scope = ferrite_reactive::Scope::new();
        let widget = scope.run(|| {
            let signal = ferrite_reactive::create_signal(false);
            let mut callback = self.callback;
            ferrite_reactive::create_effect(move || {
                if let Some(val) = signal.try_get() {
                    callback(val);
                }
            });
            let TrackPressDescriptor { inner, overrides, .. } = *self;
            let child = inner.build(tree);
            let mut style = ferrite_layout::Style::default();
            overrides.apply_to(&mut style);
            let node = tree.new_with_children(style, &[child.node_id()]);
            Box::new(TrackPressWidget { node, child, signal }) as Box<dyn crate::Widget>
        });
        Box::new(ScopedWidget { inner: widget, scope: Some(scope) })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
}

// ── Anchor ───────────────────────────────────────────────────────────────────

#[derive(Clone, Default)]
pub struct Anchor(pub std::rc::Rc<std::cell::Cell<Option<ferrite_layout::Rect>>>);

impl Anchor {
    pub fn new() -> Self { Self::default() }
    pub fn get(&self) -> Option<ferrite_layout::Rect> { self.0.get() }
}

struct AnchorDescriptor {
    inner: AnyView,
    token: Anchor,
    overrides: StyleOverrides,
}

struct AnchorWidget {
    node: ferrite_layout::NodeId,
    child: Box<dyn crate::Widget>,
    token: Anchor,
}

impl crate::Widget for AnchorWidget {
    fn node_id(&self) -> ferrite_layout::NodeId { self.node }
    fn children(&self) -> &[Box<dyn crate::Widget>] { std::slice::from_ref(&self.child) }
    fn children_mut(&mut self) -> &mut [Box<dyn crate::Widget>] { std::slice::from_mut(&mut self.child) }
    fn update(&mut self, tree: &mut ferrite_layout::LayoutTree) {
        self.child.update(tree);
    }
    fn paint(&self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, out: &mut Vec<crate::DrawCommand>) {
        let r = tree.layout(self.node);
        let abs = ferrite_layout::Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        self.token.0.set(Some(abs));
        self.child.paint(tree, ox + r.x, oy + r.y, out);
    }
}

impl ViewDescriptor for AnchorDescriptor {
    fn build(self: Box<Self>, tree: &mut ferrite_layout::LayoutTree) -> Box<dyn crate::Widget> {
        let AnchorDescriptor { inner, token, overrides } = *self;
        let child = inner.build(tree);
        let mut style = ferrite_layout::Style::default();
        overrides.apply_to(&mut style);
        let node = tree.new_with_children(style, &[child.node_id()]);
        Box::new(AnchorWidget { node, child, token })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
}

// ── TextDescriptor ───────────────────────────────────────────────────────────

struct TextDescriptor {
    content: String,
    font_size: f32,
    color: Color,
    overrides: StyleOverrides,
}

impl ViewDescriptor for TextDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let TextDescriptor { content, font_size, color, overrides } = *self;
        let mut style = Style::default();
        overrides.apply_to(&mut style);
        let content_rc = Rc::new(RefCell::new(content));
        let node = tree.new_text_leaf(style, content_rc.clone(), font_size);
        Box::new(Text {
            node,
            content: content_rc,
            color,
            size: font_size,
        })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
    fn set_font_size(&mut self, s: f32) { self.font_size = s; }
    fn set_text_color(&mut self, c: Color) { self.color = c; }
}

pub fn text(content: &str) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    AnyView {
        inner: Box::new(TextDescriptor {
            content: content.to_string(),
            font_size: widgets::DEFAULT_TEXT_SIZE,
            color: theme.on_surface,
            overrides: StyleOverrides::default(),
        }),
    }
}

// ── LabelDescriptor (reactive text) ──────────────────────────────────────────

struct LabelDescriptor {
    compute: Box<dyn Fn() -> String>,
    font_size: f32,
    color: Color,
    overrides: StyleOverrides,
}

impl ViewDescriptor for LabelDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let scope = ferrite_reactive::Scope::new();
        let widget = scope.run(|| {
            let LabelDescriptor { compute, font_size, color, overrides } = *self;
            let initial = compute();
            let mut style = Style::default();
            overrides.apply_to(&mut style);
            let content = Rc::new(RefCell::new(initial));
            let node = tree.new_text_leaf(style, content.clone(), font_size);
            let c2 = content.clone();
            create_effect(move || { 
                *c2.borrow_mut() = compute(); 
                crate::dirty::request_layout(node); 
            });
            Box::new(Text { node, content, color, size: font_size }) as Box<dyn Widget>
        });
        Box::new(ScopedWidget { inner: widget, scope: Some(scope) })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
    fn set_font_size(&mut self, s: f32) { self.font_size = s; }
    fn set_text_color(&mut self, c: Color) { self.color = c; }
}

pub fn label(f: impl Fn() -> String + 'static) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    AnyView {
        inner: Box::new(LabelDescriptor {
            compute: Box::new(f),
            font_size: widgets::DEFAULT_TEXT_SIZE,
            color: theme.on_surface,
            overrides: StyleOverrides::default(),
        }),
    }
}

// ── ButtonDescriptor ─────────────────────────────────────────────────────────

struct ButtonDescriptor {
    label: String,
    on_click: Box<dyn FnMut()>,
    background: Color,
    foreground: Color,
    overrides: StyleOverrides,
}

impl ViewDescriptor for ButtonDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let scope = ferrite_reactive::Scope::new();
        let widget = scope.run(|| {
            let ButtonDescriptor { label, on_click, background, foreground, overrides } = *self;
            let char_w = label.chars().count() as f32 * 18.0 * 0.62;
            let mut style = Style {
                width: Size::Px((char_w + 36.0).max(56.0)),
                height: Size::Px(42.0),
                ..Default::default()
            };
            overrides.apply_to(&mut style);
            let node = tree.new_leaf(style);
            let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
            
            let hovered = ferrite_reactive::create_signal(false);
            let pressed = ferrite_reactive::create_signal(false);
            
            let anim = ferrite_reactive::use_spring(
                move || {
                    if pressed.get() { 2.0 }
                    else if hovered.get() { 1.0 }
                    else { 0.0 }
                },
                ferrite_reactive::SpringConfig::stiff()
            );
            
            Box::new(widgets::Button { 
                node, label, on_click, background, foreground, theme, focused: false,
                hovered, pressed, anim
            }) as Box<dyn Widget>
        });
        Box::new(ScopedWidget { inner: widget, scope: Some(scope) })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
    fn set_background(&mut self, c: Color) { self.background = c; }
    fn set_foreground(&mut self, c: Color) { self.foreground = c; }
}

pub fn button(label: &str, on_click: impl FnMut() + 'static) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    AnyView {
        inner: Box::new(ButtonDescriptor {
            label: label.to_string(),
            on_click: Box::new(on_click),
            background: theme.primary,
            foreground: theme.on_primary,
            overrides: StyleOverrides::default(),
        }),
    }
}

// ── InputDescriptor ──────────────────────────────────────────────────────────

struct InputDescriptor {
    value: Signal<String>,
    placeholder: String,
    font_size: f32,
    input_width: f32,
    overrides: StyleOverrides,
}

impl ViewDescriptor for InputDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let InputDescriptor { value, placeholder, font_size, input_width, overrides } = *self;
        let mut style = widgets::text_input_style(input_width, font_size);
        overrides.apply_to(&mut style);
        let node = tree.new_leaf(style);
        let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
        Box::new(TextInput {
            node, value, placeholder, focused: false, cursor: 0, selection_start: None, scroll_x: 0.0,
            cursor_px: 0.0, selection_start_px: None,
            font_size, width: input_width, theme,
            layout_dirty: true, last_val: String::new(),
            last_cursor: 0, last_selection: None, last_width: 0.0,
            undo_stack: Vec::new(), redo_stack: Vec::new(),
        })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
    fn set_font_size(&mut self, s: f32) { self.font_size = s; }
}

pub fn input(value: Signal<String>, placeholder: &str) -> AnyView {
    AnyView {
        inner: Box::new(InputDescriptor {
            value,
            placeholder: placeholder.to_string(),
            font_size: 16.0,
            input_width: 280.0,
            overrides: StyleOverrides::default(),
        }),
    }
}

// ── TextAreaDescriptor ───────────────────────────────────────────────────────

struct TextAreaDescriptor {
    value: Signal<String>,
    placeholder: String,
    font_size: f32,
    overrides: StyleOverrides,
}

impl ViewDescriptor for TextAreaDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let TextAreaDescriptor { value, placeholder, font_size, overrides } = *self;
        // Default textarea style: flex growing block
        let mut style = Style {
            width: Size::Percent(100.0),
            height: Size::Px(120.0),
            flex_grow: 1.0,
            ..Default::default()
        };
        overrides.apply_to(&mut style);
        let node = tree.new_leaf(style);
        let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
        Box::new(crate::widgets::TextArea {
            node, value, placeholder, focused: false, cursor: 0, selection_start: None,
            scroll_x: 0.0, scroll_y: 0.0, cursor_px: 0.0, cursor_py: 0.0,
            selection_start_px: None, selection_start_py: None,
            line_chars: Vec::new(), line_height: font_size * 1.4,
            font_size, theme,
            layout_dirty: true, last_val: String::new(),
            last_cursor: 0, last_selection: None, last_width: 0.0,
            undo_stack: Vec::new(), redo_stack: Vec::new(),
        })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
    fn set_font_size(&mut self, s: f32) { self.font_size = s; }
}

pub fn textarea(value: Signal<String>) -> AnyView {
    AnyView {
        inner: Box::new(TextAreaDescriptor {
            value,
            placeholder: String::new(),
            font_size: widgets::DEFAULT_TEXT_SIZE,
            overrides: StyleOverrides::default(),
        }),
    }
}

// ── ContainerDescriptor ──────────────────────────────────────────────────────

struct ContainerDescriptor {
    direction: Direction,
    children: Vec<AnyView>,
    bg: Option<Color>,
    radius: f32,
    overrides: StyleOverrides,
}

impl ViewDescriptor for ContainerDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let ContainerDescriptor { direction, children, bg, radius, overrides } = *self;
        let built: Vec<Box<dyn Widget>> = children.into_iter().map(|c| c.build(tree)).collect();
        let mut style = Style { direction, ..Default::default() };
        overrides.apply_to(&mut style);
        let ids: Vec<NodeId> = built.iter().map(|c| c.node_id()).collect();
        let node = tree.new_with_children(style, &ids);
        
        let c = Container { 
            node, 
            children: built, 
            background: bg,
            corner_radius: radius 
        };
        Box::new(c)
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
    fn set_background(&mut self, c: Color) { self.bg = Some(c); }
    fn set_corner_radius(&mut self, r: f32) { self.radius = r; }
}

pub fn col(children: impl IntoIterator<Item = AnyView>) -> AnyView {
    AnyView {
        inner: Box::new(ContainerDescriptor {
            direction: Direction::Column,
            children: children.into_iter().collect(),
            bg: None,
            radius: 0.0,
            overrides: StyleOverrides::default(),
        }),
    }
}

pub fn row(children: impl IntoIterator<Item = AnyView>) -> AnyView {
    AnyView {
        inner: Box::new(ContainerDescriptor {
            direction: Direction::Row,
            children: children.into_iter().collect(),
            bg: None,
            radius: 0.0,
            overrides: StyleOverrides::default(),
        }),
    }
}

// ── SpacerDescriptor ─────────────────────────────────────────────────────────

struct SpacerDescriptor {
    overrides: StyleOverrides,
}

impl ViewDescriptor for SpacerDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let mut style = Style { flex_grow: 1.0, ..Default::default() };
        self.overrides.apply_to(&mut style);
        let node = tree.new_leaf(style);
        Box::new(Spacer { node })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
}

pub fn spacer() -> AnyView {
    AnyView { inner: Box::new(SpacerDescriptor { overrides: StyleOverrides::default() }) }
}

// ── DividerDescriptor ────────────────────────────────────────────────────────

struct DividerDescriptor {
    color: Color,
    overrides: StyleOverrides,
}

impl ViewDescriptor for DividerDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let DividerDescriptor { color, overrides } = *self;
        let mut style = Style { height: Size::Px(1.0), ..Default::default() };
        overrides.apply_to(&mut style);
        let node = tree.new_leaf(style);
        Box::new(Divider { node, color })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
    fn set_background(&mut self, c: Color) { self.color = c; }
}

pub fn divider() -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    AnyView {
        inner: Box::new(DividerDescriptor {
            color: theme.muted,
            overrides: StyleOverrides::default(),
        }),
    }
}

// ── CheckboxDescriptor ───────────────────────────────────────────────────────

struct CheckboxDescriptor {
    label: String,
    checked: Signal<bool>,
    font_size: f32,
    overrides: StyleOverrides,
}

impl ViewDescriptor for CheckboxDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let scope = ferrite_reactive::Scope::new();
        let widget = scope.run(|| {
            let CheckboxDescriptor { label, checked, font_size, overrides } = *self;
            let mut style = widgets::checkbox_style(label.len(), font_size);
            overrides.apply_to(&mut style);
            let node = tree.new_leaf(style);
            let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
            
            let anim = ferrite_reactive::use_spring(move || if checked.get() { 1.0 } else { 0.0 }, ferrite_reactive::SpringConfig::bouncy());
            
            Box::new(Checkbox { node, label_text: label, checked, anim, font_size, theme }) as Box<dyn Widget>
        });
        Box::new(ScopedWidget { inner: widget, scope: Some(scope) })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
    fn set_font_size(&mut self, s: f32) { self.font_size = s; }
}

pub fn checkbox(label: &str, checked: Signal<bool>) -> AnyView {
    AnyView {
        inner: Box::new(CheckboxDescriptor {
            label: label.to_string(),
            checked,
            font_size: widgets::DEFAULT_TEXT_SIZE,
            overrides: StyleOverrides::default(),
        }),
    }
}

// ── SliderDescriptor ─────────────────────────────────────────────────────────

struct SliderDescriptor {
    value: Signal<f32>,
    min: f32,
    max: f32,
    slider_width: f32,
    overrides: StyleOverrides,
}

impl ViewDescriptor for SliderDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let SliderDescriptor { value, min, max, slider_width, overrides } = *self;
        let mut style = widgets::slider_style(slider_width);
        overrides.apply_to(&mut style);
        let node = tree.new_leaf(style);
        let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
        Box::new(Slider { node, value, min, max, theme })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
}

pub fn slider(value: Signal<f32>, min: f32, max: f32) -> AnyView {
    AnyView {
        inner: Box::new(SliderDescriptor {
            value, min, max,
            slider_width: 200.0,
            overrides: StyleOverrides::default(),
        }),
    }
}

// ── ScrollDescriptor ───────────────────────────────────────────────────────────

struct ScrollDescriptor {
    child: AnyView,
    overrides: StyleOverrides,
}

impl ViewDescriptor for ScrollDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let ScrollDescriptor { child, overrides } = *self;
        let built_child = child.flex_shrink(0.0).build(tree);
        let mut style = Style {
            overflow_x: ferrite_layout::Overflow::Scroll,
            overflow_y: ferrite_layout::Overflow::Scroll,
            flex_grow: 1.0,
            ..Default::default()
        };
        overrides.apply_to(&mut style);
        
        let node = tree.new_with_children(style, &[built_child.node_id()]);
        
        Box::new(Scroll {
            node,
            child: built_child,
            scroll_x: 0.0,
            scroll_y: 0.0,
        })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
}

pub fn scroll(child: impl View + 'static) -> AnyView {
    AnyView {
        inner: Box::new(ScrollDescriptor {
            child: child.view(),
            overrides: StyleOverrides::default(),
        }),
    }
}

// ── VisibleWhenWidget ────────────────────────────────────────────────────────

struct VisibleWhenWidget {
    node: NodeId,
    child: Box<dyn Widget>,
    visible: Rc<dyn Fn() -> bool>,
}

impl Widget for VisibleWhenWidget {
    fn node_id(&self) -> NodeId { self.node }
    
    fn children(&self) -> &[Box<dyn Widget>] {
        std::slice::from_ref(&self.child)
    }
    
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        std::slice::from_mut(&mut self.child)
    }

    fn update(&mut self, tree: &mut LayoutTree) { self.child.update(tree); }

    fn paint(&self, tree: &LayoutTree, ox: f32, oy: f32, out: &mut Vec<DrawCommand>) {
        if (self.visible)() {
            let r = tree.layout(self.node);
            let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
            self.child.paint(tree, abs.x, abs.y, out);
        }
    }

    fn click_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<NodeId> {
        if !(self.visible)() { return None; }
        let r = tree.layout(self.node);
        let ax = ox + r.x; let ay = oy + r.y;
        if px < ax || py < ay || px > ax + r.width || py > ay + r.height { return None; }
        self.child.click_at(tree, ax, ay, px, py)
    }
}

struct VisibleWhenDescriptor {
    inner: AnyView,
    signal: Rc<dyn Fn() -> bool>,
    overrides: StyleOverrides,
}

impl ViewDescriptor for VisibleWhenDescriptor {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let VisibleWhenDescriptor { inner, signal, overrides } = *self;
        let child = inner.build(tree);
        
        // Compute the wrapper node's intended style once, capture it in the hook
        let mut natural_style = Style::default();
        overrides.apply_to(&mut natural_style);
        let hidden_style = Style { width: Size::Px(0.0), height: Size::Px(0.0), ..Default::default() };
        
        let node = tree.new_with_children(natural_style, &[child.node_id()]);
        
        // Share signal execution across layout hook and repaint effect
        let sig_hook = signal.clone();
        let sig_eff = signal.clone();

        tree.add_pre_layout_hook(move || {
            vec![(node, if sig_hook() { natural_style } else { hidden_style })]
        });

        create_effect(move || { let _ = sig_eff(); request_repaint(); });

        Box::new(VisibleWhenWidget { node, child, visible: signal })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
}

// ── switch ───────────────────────────────────────────────────────────────────

pub fn switch<K: PartialEq + Clone + 'static>(key: Signal<K>, branches: impl IntoIterator<Item = (K, AnyView)>) -> AnyView {
    let children: Vec<AnyView> = branches.into_iter().map(|(k, view)| {
        view.visible_when(move || key.get() == k)
    }).collect();
    col(children)
}

// ── list ─────────────────────────────────────────────────────────────────────

pub fn list<T: Clone + 'static>(
    signal: Signal<Vec<T>>,
    view_fn: impl Fn(&T) -> AnyView + 'static,
) -> AnyView {
    AnyView { inner: Box::new(ListDescriptor {
        signal,
        view_fn: Rc::new(view_fn),
        overrides: StyleOverrides::default(),
    }) }
}

// ── Portal (Overlay) ─────────────────────────────────────────────────────────

/// A portal creates an overlay that renders outside the normal widget tree, floating on top.
pub fn portal(show: ferrite_reactive::Signal<bool>, content: impl Fn() -> AnyView + 'static) -> AnyView {
    struct PortalBuilder<F> {
        show: ferrite_reactive::Signal<bool>,
        content: F,
        overrides: StyleOverrides,
    }
    impl<F: Fn() -> AnyView + 'static> ViewDescriptor for PortalBuilder<F> {
        fn build(self: Box<Self>, tree: &mut ferrite_layout::LayoutTree) -> Box<dyn crate::Widget> {
            let mut style = ferrite_layout::Style {
                width: ferrite_layout::Size::Px(0.0),
                height: ferrite_layout::Size::Px(0.0),
                ..Default::default()
            };
            self.overrides.apply_to(&mut style);
            let node = tree.new_leaf(style);
            let widget = crate::widgets::PortalWidget {
                node,
                show: self.show,
                content: Box::new(self.content),
                active_overlay: None,
                last_show: false, // will update on first frame
            };
            Box::new(widget)
        }
        fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
    }

    AnyView { inner: Box::new(PortalBuilder { show, content, overrides: StyleOverrides::default() }) }
}

struct ListDescriptor<T> {
    signal: Signal<Vec<T>>,
    view_fn: Rc<dyn Fn(&T) -> AnyView>,
    overrides: StyleOverrides,
}

impl<T: Clone + 'static> ViewDescriptor for ListDescriptor<T> {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
        let scope = ferrite_reactive::Scope::new();
        let widget = scope.run(|| {
            let items = self.signal.get();
            let mut children = Vec::with_capacity(items.len());
            let mut child_nodes = Vec::with_capacity(items.len());
            
            for item in &items {
                let child = (self.view_fn)(item).build(tree);
                child_nodes.push(child.node_id());
                children.push(child);
            }

            let mut natural_style = Style {
                direction: Direction::Column, // Default to column for lists
                ..Default::default()
            };
            self.overrides.apply_to(&mut natural_style);
            
            let node = tree.new_with_children(natural_style, &child_nodes);
            let last_revision = ferrite_reactive::get_mutations(self.signal, 0).0;

            let sig_eff = self.signal.clone();
            ferrite_reactive::create_effect(move || {
                sig_eff.track(); // Subscribe to any mutation or change
                crate::dirty::request_repaint();
            });

            Box::new(ListWidget {
                node,
                signal: self.signal,
                view_fn: self.view_fn,
                children,
                last_revision,
            }) as Box<dyn Widget>
        });
        Box::new(ScopedWidget { inner: widget, scope: Some(scope) })
    }
    fn style_overrides_mut(&mut self) -> &mut StyleOverrides { &mut self.overrides }
}

struct ListWidget<T> {
    node: NodeId,
    signal: Signal<Vec<T>>,
    view_fn: Rc<dyn Fn(&T) -> AnyView>,
    children: Vec<Box<dyn Widget>>,
    last_revision: usize,
}

impl<T: Clone + 'static> Widget for ListWidget<T> {
    fn node_id(&self) -> NodeId { self.node }
    fn children(&self) -> &[Box<dyn Widget>] { &self.children }
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] { &mut self.children }
    
    fn update(&mut self, tree: &mut LayoutTree) {
        let (new_rev, mutations) = ferrite_reactive::get_mutations(self.signal, self.last_revision);
        if new_rev > self.last_revision {
            if mutations.is_empty() {
                // A full .set() or .update() occurred (not via SignalVecExt). Full rebuild required!
                for child in &mut self.children { child.destroy(tree); }
                self.children.clear();
                
                let items = self.signal.get();
                let mut child_nodes = Vec::with_capacity(items.len());
                for item in &items {
                    let child = (self.view_fn)(item).build(tree);
                    child_nodes.push(child.node_id());
                    self.children.push(child);
                }
                tree.set_children(self.node, &child_nodes);
                crate::dirty::request_repaint();
            } else {
                // Apply O(1) differential updates
                let mut changed = false;
                for mutation in mutations {
                    changed = true;
                    match mutation {
                        ferrite_reactive::ListMutation::Push(item) => {
                            let child = (self.view_fn)(&item).build(tree);
                            self.children.push(child);
                        }
                        ferrite_reactive::ListMutation::Insert(index, item) => {
                            let child = (self.view_fn)(&item).build(tree);
                            self.children.insert(index, child);
                        }
                        ferrite_reactive::ListMutation::Remove(index) => {
                            if index < self.children.len() {
                                let mut removed = self.children.remove(index);
                                removed.destroy(tree);
                            }
                        }
                        ferrite_reactive::ListMutation::Clear => {
                            for child in &mut self.children {
                                child.destroy(tree);
                            }
                            self.children.clear();
                        }
                    }
                }
                if changed {
                    let child_nodes: Vec<NodeId> = self.children.iter().map(|c| c.node_id()).collect();
                    tree.set_children(self.node, &child_nodes);
                    crate::dirty::request_repaint();
                }
            }
            self.last_revision = new_rev;
        }

        // Recursively update children
        for child in &mut self.children {
            child.update(tree);
        }
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

// ── Modal (High-level Portal) ──────────────────────────────────────────────────

pub fn modal(show: ferrite_reactive::Signal<bool>, mut on_close: impl FnMut() + Clone + 'static, content: impl Fn() -> AnyView + 'static) -> AnyView {
    let content = std::rc::Rc::new(content);
    portal(show.clone(), move || {
        let mut close = on_close.clone();
        let content = content.clone();
        
        let background = col([
            col([
                // The actual modal content
                content()
            ])
            .padding(24.0)
            .background(crate::context::try_inject::<crate::theme::Theme>().unwrap_or_default().surface)
            .corner_radius(12.0)
        ])
        .fill()
        .background(crate::Color::rgba(0.0, 0.0, 0.0, 0.5)) // dim backdrop
        .align(ferrite_layout::AlignItems::Center)
        .justify(ferrite_layout::JustifyContent::Center);
        
        // Wait, button currently has hover style so we don't want a button for background yet.
        // We will just use the layout. If we need click-away to close, we'd add an invisible click absorber.
        // For now, modal content needs to handle closing if it wants.
        background
    })
}

// ── Dropdown (Anchored Portal) ───────────────────────────────────────────────

pub fn dropdown(
    label: &str,
    width: f32,
    items: Vec<String>,
    mut on_select: impl FnMut(usize, String) + Clone + 'static,
) -> AnyView {
    let show = ferrite_reactive::create_signal(false);
    let anchor = Anchor::new();
    
    let trigger = button(label, {
        let show = show.clone();
        move || {
            let current = show.get();
            show.set(!current);
        }
    })
    .width(width)
    .anchor(anchor.clone());
    
    let items_rc = std::rc::Rc::new(items);
    let portal_view = portal(show.clone(), move || {
        let anchor_rect = anchor.get().unwrap_or_default();
        let items_clone = items_rc.clone();
        let mut on_select_clone = on_select.clone();
        let show_dropdown = show.clone();
        
        let mut children = Vec::new();
        for (i, item) in items_clone.iter().enumerate() {
            let mut select_cb = on_select_clone.clone();
            let show_cb = show_dropdown.clone();
            let item_clone = item.clone();
            
            // Dropdown items now use standard button with single_line truncation enforced by the renderer
            let text = item.clone();
            
            children.push(
                button(&text, move || {
                    select_cb(i, item_clone.clone());
                    show_cb.set(false);
                })
                .background(crate::Color::WHITE)
                .foreground(crate::Color::rgb(0.0, 0.0, 0.0))
                .width(width)
            );
        }
        
        col([
            col(children)
                .background(crate::Color::rgb(0.9, 0.9, 0.9))
                .padding(1.0)
                .corner_radius(4.0)
                .width(width + 2.0)
                .position_type(ferrite_layout::PositionType::Absolute)
                .inset(ferrite_layout::Inset {
                    top: ferrite_layout::Size::Px(anchor_rect.y + anchor_rect.height + 6.0),
                    left: ferrite_layout::Size::Px(anchor_rect.x),
                    right: ferrite_layout::Size::Auto,
                    bottom: ferrite_layout::Size::Auto,
                })
        ]).fill()
    });
    
    col([
        trigger,
        portal_view
    ])
}

// ── card ────────────────────────────────────────────────────────────────────

/// Groups content on a raised surface with a subtle 1-px border and rounded
/// corners. Uses `theme.surface` + `theme.border` automatically — you only
/// pass children.
///
/// ```no_run
/// card([
///     text("Title").size(18.0),
///     text("Body text here.").color(theme.text_secondary),
/// ])
/// ```
pub fn card(children: impl IntoIterator<Item = AnyView>) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    // Border effect: outer container in border colour, inner inset by 1 px.
    col([
        col(children)
            .padding(theme.space_4)
            .background(theme.surface)
            .corner_radius(theme.radius_md - 1.0),
    ])
    .padding(1.0)
    .background(theme.border)
    .corner_radius(theme.radius_md)
}

// ── badge ─────────────────────────────────────────────────────────────────────

/// Small primary-coloured pill — version numbers, counts, status labels.
pub fn badge(text_str: &str) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    col([text(text_str).size(11.0).color(theme.on_primary)])
        .padding_xy(8.0, 3.0)
        .background(theme.primary)
        .corner_radius(theme.radius_xs)
}

/// Success (green) badge.
pub fn badge_success(text_str: &str) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    col([text(text_str).size(11.0).color(Color::WHITE)])
        .padding_xy(8.0, 3.0)
        .background(theme.success)
        .corner_radius(theme.radius_xs)
}

/// Muted (surface-2) badge — for neutral labels, tags, secondary info.
pub fn badge_muted(text_str: &str) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    col([text(text_str).size(11.0).color(theme.text_secondary)])
        .padding_xy(8.0, 3.0)
        .background(theme.surface_2)
        .corner_radius(theme.radius_xs)
}

// ── button variants ───────────────────────────────────────────────────────────

/// Secondary button — uses `theme.surface_2` background and regular text.
/// Less prominent than the primary `button()`, more prominent than ghost.
pub fn button_secondary(label_str: &str, on_click: impl FnMut() + 'static) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    AnyView {
        inner: Box::new(ButtonDescriptor {
            label: label_str.to_string(),
            on_click: Box::new(on_click),
            background: theme.surface_2,
            foreground: theme.text,
            overrides: StyleOverrides::default(),
        }),
    }
}

/// Ghost button — transparent background, primary text colour. Use inside
/// cards or next to a primary button as a "cancel" or secondary action.
pub fn button_ghost(label_str: &str, on_click: impl FnMut() + 'static) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    AnyView {
        inner: Box::new(ButtonDescriptor {
            label: label_str.to_string(),
            on_click: Box::new(on_click),
            background: Color::TRANSPARENT,
            foreground: theme.primary,
            overrides: StyleOverrides::default(),
        }),
    }
}

/// Danger button — red background for irreversible or destructive actions.
pub fn button_danger(label_str: &str, on_click: impl FnMut() + 'static) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    AnyView {
        inner: Box::new(ButtonDescriptor {
            label: label_str.to_string(),
            on_click: Box::new(on_click),
            background: theme.danger,
            foreground: Color::WHITE,
            overrides: StyleOverrides::default(),
        }),
    }
}

// ── section_header ─────────────────────────────────────────────────────────────

/// A section title + optional secondary text, rendered as a small header row.
/// Useful above cards or content groups to label what's inside.
pub fn section_header(title: &str, subtitle: &str) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    let has_subtitle = !subtitle.is_empty();
    col([
        text(title).size(13.0).color(theme.text_secondary),
        if has_subtitle { text(subtitle).size(11.0).color(theme.muted) } else { spacer().height(0.0) },
    ])
    .gap(2.0)
}

// ── key_value ──────────────────────────────────────────────────────────────────

/// A single label + value row — for settings displays, metadata, detail panes.
///     Left: muted label text
///     Right: primary value text
pub fn key_value(key: &str, value: &str) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    row([
        text(key).size(14.0).color(theme.text_secondary),
        spacer(),
        text(value).size(14.0).color(theme.text),
    ])
    .align(AlignItems::Center)
}

/// Reactive version of `key_value` — `value_fn` re-runs when signals it
/// reads change.
pub fn key_value_dyn(key: &str, value_fn: impl Fn() -> String + 'static) -> AnyView {
    let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
    row([
        text(key).size(14.0).color(theme.text_secondary),
        spacer(),
        label(value_fn).size(14.0).color(theme.text),
    ])
    .align(AlignItems::Center)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferrite_reactive as reactive;

    #[test]
    fn text_builds_to_widget() {
        let mut tree = LayoutTree::new();
        let view = text("Hello").size(20.0).color(Color::rgb(1.0, 0.0, 0.0));
        let widget = view.build(&mut tree);
        let mut cmds = Vec::new();
        let rect = Rect { x: 0.0, y: 0.0, width: 100.0, height: 30.0 };
        widget.paint_self(rect, &mut cmds);
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            DrawCommand::Text { content, size, color, .. } => {
                assert_eq!(content, "Hello");
                assert_eq!(*size, 20.0);
                assert_eq!(*color, Color::rgb(1.0, 0.0, 0.0));
            }
            _ => panic!("expected text draw command"),
        }
    }

    #[test]
    fn col_with_children_builds() {
        let mut tree = LayoutTree::new();
        let view = col([
            text("A"),
            text("B"),
            text("C"),
        ]).gap(10.0).padding(20.0);
        let widget = view.build(&mut tree);
        assert_eq!(widget.children().len(), 3);
    }

    #[test]
    fn label_reacts_to_signal() {
        let count = reactive::create_signal(0i32);
        let mut tree = LayoutTree::new();
        let root = col([
            label(move || format!("Count: {}", count.get())),
        ]).width(200.0).height(50.0);
        let mut widget = root.build(&mut tree);
        tree.compute(widget.node_id(), 200.0, 50.0);
        let mut cmds = Vec::new();
        widget.paint(&tree, 0.0, 0.0, &mut cmds);
        let txt = cmds.iter().find_map(|c| match c {
            DrawCommand::Text { content, .. } => Some(content.clone()),
            _ => None,
        });
        assert_eq!(txt.as_deref(), Some("Count: 0"));

        count.set(5);
        cmds.clear();
        widget.update(&mut tree); // Manual update tick
        widget.paint(&tree, 0.0, 0.0, &mut cmds);
        let txt2 = cmds.iter().find_map(|c| match c {
            DrawCommand::Text { content, .. } => Some(content.clone()),
            _ => None,
        });
        assert_eq!(txt2.as_deref(), Some("Count: 5"));
    }

    use ferrite_reactive::SignalVecExt;
    
    #[test]
    fn list_mutations_update_children() {
        let count = reactive::create_signal(vec!["A".to_string(), "B".to_string()]);
        let mut tree = LayoutTree::new();
        let root = col([
            list(count, |item| text(item))
        ]).width(200.0).height(50.0);
        let mut widget = root.build(&mut tree);
        
        // Initial state
        assert_eq!(widget.children()[0].children().len(), 2);
        
        // Push
        count.push("C".to_string());
        widget.update(&mut tree);
        assert_eq!(widget.children()[0].children().len(), 3);
        
        // Remove
        count.remove(0);
        widget.update(&mut tree);
        assert_eq!(widget.children()[0].children().len(), 2);
        
        // Clear
        count.clear();
        widget.update(&mut tree);
        assert_eq!(widget.children()[0].children().len(), 0);
        
        // Set
        count.set(vec!["X".to_string()]);
        widget.update(&mut tree);
        assert_eq!(widget.children()[0].children().len(), 1);
    }

    #[test]
    fn button_click_updates_signal() {
        let count = reactive::create_signal(0i32);
        let mut tree = LayoutTree::new();
        let root_view = col([
            label(move || format!("{}", count.get())),
            button("+", move || count.update(|c| *c += 1)),
        ]).width(200.0).height(100.0);
        let root = root_view.build(&mut tree);
        let mut app = crate::App::new(tree, root);
        app.render(200.0, 100.0);
        assert_eq!(count.get(), 0);

        let btn_rect = app.absolute_rect(app.root().children()[1].node_id());
        if let Some(r) = btn_rect {
            app.click(r.x + r.width / 2.0, r.y + r.height / 2.0);
        }
        assert_eq!(count.get(), 1);
    }

    #[test]
    fn view_trait_for_component() {
        struct MyCounter { count: Signal<i32> }
        impl View for MyCounter {
            fn view(self) -> AnyView {
                col([
                    label(move || format!("{}", self.count.get())),
                    button("+", move || self.count.update(|c| *c += 1)),
                ])
            }
        }

        let count = reactive::create_signal(0);
        let component = MyCounter { count };
        let view = component.view();
        let mut tree = LayoutTree::new();
        let widget = view.build(&mut tree);
        assert_eq!(widget.children().len(), 2);
    }
}

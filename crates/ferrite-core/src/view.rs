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
}

// ── View trait ───────────────────────────────────────────────────────────────

pub trait View {
    fn view(self) -> AnyView;
}

impl View for AnyView {
    fn view(self) -> AnyView { self }
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
        Box::new(Text { node, content, color, size: font_size })
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
        Box::new(Button { node, label, on_click, background, foreground, theme, focused: false })
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
        let CheckboxDescriptor { label, checked, font_size, overrides } = *self;
        let mut style = widgets::checkbox_style(label.len(), font_size);
        overrides.apply_to(&mut style);
        let node = tree.new_leaf(style);
        let theme = crate::context::try_inject::<Theme>().unwrap_or_default();
        
        let anim = ferrite_reactive::use_spring(move || if checked.get() { 1.0 } else { 0.0 }, ferrite_reactive::SpringConfig::bouncy());
        
        Box::new(Checkbox { node, label_text: label, checked, anim, font_size, theme })
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

struct ListDescriptor<T> {
    signal: Signal<Vec<T>>,
    view_fn: Rc<dyn Fn(&T) -> AnyView>,
    overrides: StyleOverrides,
}

impl<T: Clone + 'static> ViewDescriptor for ListDescriptor<T> {
    fn build(self: Box<Self>, tree: &mut LayoutTree) -> Box<dyn Widget> {
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
        })
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

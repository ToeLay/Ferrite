use crate::{request_repaint, Color, DrawCommand, KeyCode, KeyEvent, Widget};
use ferrite_layout::{Direction, LayoutTree, NodeId, Rect, Size, Style};
use ferrite_reactive::Signal;
use std::cell::RefCell;
use std::rc::Rc;

// ── Container ────────────────────────────────────────────────────────────────

pub struct Container {
    pub(crate) node: NodeId,
    pub(crate) children: Vec<Box<dyn Widget>>,
    pub(crate) background: Option<Color>,
    pub(crate) corner_radius: f32,
}
impl Container {
    pub fn background(mut self, color: Color) -> Self { self.background = Some(color); self }
    pub fn corner_radius(mut self, radius: f32) -> Self { self.corner_radius = radius; self }
}
impl Widget for Container {
    fn node_id(&self) -> NodeId { self.node }
    fn children(&self) -> &[Box<dyn Widget>] { &self.children }
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] { &mut self.children }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        if let Some(color) = self.background {
            out.push(DrawCommand::Rect { rect, color, corner_radius: self.corner_radius });
        }
    }
}

pub(crate) fn container(tree: &mut LayoutTree, mut style: Style, dir: Direction, children: Vec<Box<dyn Widget>>) -> Container {
    style.direction = dir;
    let ids: Vec<NodeId> = children.iter().map(|c| c.node_id()).collect();
    let node = tree.new_with_children(style, &ids);
    Container { node, children, background: None, corner_radius: 0.0 }
}

pub(crate) fn column(tree: &mut LayoutTree, style: Style, children: Vec<Box<dyn Widget>>) -> Container {
    container(tree, style, Direction::Column, children)
}
pub(crate) fn row(tree: &mut LayoutTree, style: Style, children: Vec<Box<dyn Widget>>) -> Container {
    container(tree, style, Direction::Row, children)
}

// ── Text ─────────────────────────────────────────────────────────────────────

pub struct Text {
    pub(crate) node: NodeId,
    pub(crate) content: Rc<RefCell<String>>,
    pub(crate) color: Color,
    pub(crate) size: f32,
}
impl Text {
    pub fn color(mut self, color: Color) -> Self { self.color = color; self }
    pub fn font_size(mut self, tree: &mut LayoutTree, size: f32) -> Self {
        self.size = size;
        let count = self.content.borrow().chars().count() as f32;
        tree.set_style(self.node, text_node_style(count, size));
        self
    }
}
impl Widget for Text {
    fn node_id(&self) -> NodeId { self.node }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        out.push(DrawCommand::Text {
            x: rect.x, y: rect.y,
            content: self.content.borrow().clone(),
            size: self.size, color: self.color,
        });
    }
}

pub(crate) fn text_node_style(char_count: f32, size: f32) -> Style {
    Style { width: Size::Px(char_count * size * 0.62), height: Size::Px(size * 1.4), ..Default::default() }
}
pub(crate) const DEFAULT_TEXT_SIZE: f32 = 16.0;

pub(crate) fn text(tree: &mut LayoutTree, content: impl Into<String>) -> Text {
    let s = content.into();
    let node = tree.new_leaf(text_node_style(s.chars().count() as f32, DEFAULT_TEXT_SIZE));
    Text { node, content: Rc::new(RefCell::new(s)), color: Color::BLACK, size: DEFAULT_TEXT_SIZE }
}

pub(crate) fn text_dyn(tree: &mut LayoutTree, f: impl Fn() -> String + 'static) -> Text {
    let initial = f();
    let node = tree.new_leaf(text_node_style(initial.chars().count() as f32, DEFAULT_TEXT_SIZE));
    let content = Rc::new(RefCell::new(initial));
    let c2 = content.clone();
    ferrite_reactive::create_effect(move || { *c2.borrow_mut() = f(); request_repaint(); });
    Text { node, content, color: Color::BLACK, size: DEFAULT_TEXT_SIZE }
}

// ── Button ───────────────────────────────────────────────────────────────────

pub struct Button {
    pub(crate) node: NodeId,
    pub(crate) label: String,
    pub(crate) on_click: Box<dyn FnMut()>,
    pub(crate) background: Color,
    pub(crate) foreground: Color,
}
impl Button {
    pub fn background(mut self, c: Color) -> Self { self.background = c; self }
    pub fn foreground(mut self, c: Color) -> Self { self.foreground = c; self }
}
impl Widget for Button {
    fn node_id(&self) -> NodeId { self.node }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        out.push(DrawCommand::Rect { rect, color: self.background, corner_radius: 10.0 });
        out.push(DrawCommand::Text {
            x: rect.x + 18.0, y: rect.y + rect.height / 2.0 - 9.0,
            content: self.label.clone(), size: 18.0, color: self.foreground,
        });
    }
    fn on_click(&mut self) -> bool { (self.on_click)(); true }
}

pub(crate) fn button(tree: &mut LayoutTree, label: impl Into<String>, on_click: impl FnMut() + 'static) -> Button {
    let node = tree.new_leaf(Style { width: Size::Px(56.0), height: Size::Px(56.0), ..Default::default() });
    Button {
        node, label: label.into(), on_click: Box::new(on_click),
        background: Color::rgb(0.21, 0.43, 0.86), foreground: Color::WHITE,
    }
}

// ── TextInput ────────────────────────────────────────────────────────────────

pub struct TextInput {
    pub(crate) node: NodeId,
    pub(crate) value: Signal<String>,
    pub(crate) placeholder: String,
    pub(crate) focused: bool,
    pub(crate) cursor: usize,
    pub(crate) font_size: f32,
    pub(crate) width: f32,
}

impl TextInput {
    pub fn placeholder(mut self, s: impl Into<String>) -> Self { self.placeholder = s.into(); self }

    pub fn width(mut self, tree: &mut LayoutTree, w: f32) -> Self {
        self.width = w;
        tree.set_style(self.node, text_input_style(w, self.font_size));
        self
    }

    pub fn font_size(mut self, tree: &mut LayoutTree, size: f32) -> Self {
        self.font_size = size;
        tree.set_style(self.node, text_input_style(self.width, size));
        self
    }

    fn cursor_x(&self) -> f32 { self.cursor as f32 * self.font_size * 0.62 }
}

impl Widget for TextInput {
    fn node_id(&self) -> NodeId { self.node }
    fn is_focusable(&self) -> bool { true }

    fn on_focus_change(&mut self, focused: bool) {
        self.focused = focused;
        request_repaint();
    }

    fn on_key(&mut self, event: &KeyEvent) -> bool {
        let val = self.value.get();
        let char_count = val.chars().count();
        match &event.key {
            KeyCode::Char(ch) => {
                let byte_pos = val.char_indices().nth(self.cursor).map(|(i,_)| i).unwrap_or(val.len());
                let mut s = val.clone(); s.insert(byte_pos, *ch);
                self.cursor += 1; self.value.set(s); request_repaint(); true
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    let byte_pos = val.char_indices().nth(self.cursor - 1).map(|(i,_)| i).unwrap_or(0);
                    let mut s = val.clone(); s.remove(byte_pos);
                    self.cursor -= 1; self.value.set(s); request_repaint();
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor < char_count {
                    let byte_pos = val.char_indices().nth(self.cursor).map(|(i,_)| i).unwrap_or(val.len());
                    let mut s = val.clone(); s.remove(byte_pos);
                    self.value.set(s); request_repaint();
                }
                true
            }
            KeyCode::Left  => { if self.cursor > 0 { self.cursor -= 1; request_repaint(); } true }
            KeyCode::Right => { if self.cursor < char_count { self.cursor += 1; request_repaint(); } true }
            KeyCode::Home  => { self.cursor = 0; request_repaint(); true }
            KeyCode::End   => { self.cursor = char_count; request_repaint(); true }
            KeyCode::Tab | KeyCode::Escape => false,
            _ => false,
        }
    }

    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        let border_color = if self.focused { Color::rgb(0.21, 0.43, 0.86) } else { Color::rgb(0.76, 0.78, 0.82) };
        let pad = 10.0_f32;
        out.push(DrawCommand::Rect { rect, color: border_color, corner_radius: 7.0 });
        let inner = Rect { x: rect.x + 1.5, y: rect.y + 1.5, width: rect.width - 3.0, height: rect.height - 3.0 };
        out.push(DrawCommand::Rect { rect: inner, color: Color::rgb(1.0, 1.0, 1.0), corner_radius: 6.0 });
        let val = self.value.get();
        let text_y = rect.y + (rect.height - self.font_size) / 2.0;
        if val.is_empty() {
            out.push(DrawCommand::Text { x: rect.x + pad, y: text_y, content: self.placeholder.clone(),
                size: self.font_size, color: Color::rgb(0.6, 0.62, 0.66) });
        } else {
            out.push(DrawCommand::Text { x: rect.x + pad, y: text_y, content: val,
                size: self.font_size, color: Color::rgb(0.08, 0.08, 0.10) });
        }
        if self.focused {
            let cx = rect.x + pad + self.cursor_x();
            out.push(DrawCommand::Rect {
                rect: Rect { x: cx, y: text_y, width: 2.0, height: self.font_size },
                color: Color::rgb(0.21, 0.43, 0.86), corner_radius: 0.0,
            });
        }
    }
}

pub(crate) fn text_input_style(width: f32, font_size: f32) -> Style {
    Style { width: Size::Px(width), height: Size::Px(font_size * 2.4), ..Default::default() }
}

pub(crate) fn text_input(tree: &mut LayoutTree, value: Signal<String>, placeholder: impl Into<String>) -> TextInput {
    const DEFAULT_WIDTH: f32 = 280.0;
    const DEFAULT_FONT_SIZE: f32 = 16.0;
    let node = tree.new_leaf(text_input_style(DEFAULT_WIDTH, DEFAULT_FONT_SIZE));
    TextInput { node, value, placeholder: placeholder.into(), focused: false, cursor: 0,
        font_size: DEFAULT_FONT_SIZE, width: DEFAULT_WIDTH }
}

// ── Spacer ───────────────────────────────────────────────────────────────────

pub struct Spacer {
    pub(crate) node: NodeId,
}

impl Widget for Spacer {
    fn node_id(&self) -> NodeId { self.node }
}

pub(crate) fn spacer_widget(tree: &mut LayoutTree) -> Spacer {
    let node = tree.new_leaf(Style { flex_grow: 1.0, ..Default::default() });
    Spacer { node }
}

// ── Divider ──────────────────────────────────────────────────────────────────

pub struct Divider {
    pub(crate) node: NodeId,
    pub(crate) color: Color,
}

impl Widget for Divider {
    fn node_id(&self) -> NodeId { self.node }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        out.push(DrawCommand::Rect { rect, color: self.color, corner_radius: 0.0 });
    }
}

pub(crate) fn divider_widget(tree: &mut LayoutTree) -> Divider {
    let node = tree.new_leaf(Style { height: Size::Px(1.0), ..Default::default() });
    Divider { node, color: Color::rgb(0.85, 0.87, 0.90) }
}

// ── Checkbox ─────────────────────────────────────────────────────────────────

pub struct Checkbox {
    pub(crate) node: NodeId,
    pub(crate) label_text: String,
    pub(crate) checked: Signal<bool>,
    pub(crate) font_size: f32,
}

impl Widget for Checkbox {
    fn node_id(&self) -> NodeId { self.node }

    fn on_click(&mut self) -> bool {
        self.checked.update(|v| *v = !*v);
        request_repaint();
        true
    }

    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        let box_size = self.font_size * 1.2;
        let by = rect.y + (rect.height - box_size) / 2.0;

        let outer = Rect { x: rect.x, y: by, width: box_size, height: box_size };
        let border_color = if self.checked.get() {
            Color::rgb(0.21, 0.43, 0.86)
        } else {
            Color::rgb(0.72, 0.74, 0.78)
        };
        out.push(DrawCommand::Rect { rect: outer, color: border_color, corner_radius: 4.0 });

        let inner = Rect { x: outer.x + 2.0, y: outer.y + 2.0, width: outer.width - 4.0, height: outer.height - 4.0 };
        if self.checked.get() {
            out.push(DrawCommand::Rect { rect: inner, color: Color::rgb(0.21, 0.43, 0.86), corner_radius: 2.0 });
            let mark = Rect {
                x: inner.x + inner.width * 0.2, y: inner.y + inner.height * 0.2,
                width: inner.width * 0.6, height: inner.height * 0.6,
            };
            out.push(DrawCommand::Rect { rect: mark, color: Color::WHITE, corner_radius: 1.0 });
        } else {
            out.push(DrawCommand::Rect { rect: inner, color: Color::WHITE, corner_radius: 2.0 });
        }

        if !self.label_text.is_empty() {
            let tx = rect.x + box_size + 8.0;
            let ty = rect.y + (rect.height - self.font_size) / 2.0;
            out.push(DrawCommand::Text {
                x: tx, y: ty, content: self.label_text.clone(),
                size: self.font_size, color: Color::rgb(0.08, 0.08, 0.10),
            });
        }
    }
}

pub(crate) fn checkbox_style(label_len: usize, font_size: f32) -> Style {
    let box_size = font_size * 1.2;
    let label_w = if label_len > 0 { label_len as f32 * font_size * 0.62 + 8.0 } else { 0.0 };
    Style {
        width: Size::Px(box_size + label_w),
        height: Size::Px(box_size.max(font_size * 1.4)),
        ..Default::default()
    }
}

pub(crate) fn checkbox_widget(tree: &mut LayoutTree, label: impl Into<String>, checked: Signal<bool>) -> Checkbox {
    let label_text = label.into();
    let font_size = DEFAULT_TEXT_SIZE;
    let node = tree.new_leaf(checkbox_style(label_text.len(), font_size));
    Checkbox { node, label_text, checked, font_size }
}

// ── Slider ───────────────────────────────────────────────────────────────────

pub struct Slider {
    pub(crate) node: NodeId,
    pub(crate) value: Signal<f32>,
    pub(crate) min: f32,
    pub(crate) max: f32,
}

impl Widget for Slider {
    fn node_id(&self) -> NodeId { self.node }

    fn click_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> bool {
        let r = tree.layout(self.node_id());
        let ax = ox + r.x;
        let ay = oy + r.y;
        if px < ax || py < ay || px > ax + r.width || py > ay + r.height { return false; }
        let ratio = ((px - ax) / r.width).clamp(0.0, 1.0);
        self.value.set(self.min + (self.max - self.min) * ratio);
        request_repaint();
        true
    }

    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        let val = self.value.get();
        let ratio = ((val - self.min) / (self.max - self.min)).clamp(0.0, 1.0);
        let track_h = 4.0;
        let track_y = rect.y + rect.height / 2.0 - track_h / 2.0;

        out.push(DrawCommand::Rect {
            rect: Rect { x: rect.x, y: track_y, width: rect.width, height: track_h },
            color: Color::rgb(0.85, 0.87, 0.90), corner_radius: 2.0,
        });

        let fill_w = rect.width * ratio;
        if fill_w > 0.0 {
            out.push(DrawCommand::Rect {
                rect: Rect { x: rect.x, y: track_y, width: fill_w, height: track_h },
                color: Color::rgb(0.21, 0.43, 0.86), corner_radius: 2.0,
            });
        }

        let thumb_r = 8.0;
        let thumb_cx = rect.x + rect.width * ratio;
        let thumb_x = (thumb_cx - thumb_r).max(rect.x);
        let thumb_y = rect.y + rect.height / 2.0 - thumb_r;
        out.push(DrawCommand::Rect {
            rect: Rect { x: thumb_x, y: thumb_y, width: thumb_r * 2.0, height: thumb_r * 2.0 },
            color: Color::WHITE, corner_radius: thumb_r,
        });
        out.push(DrawCommand::Rect {
            rect: Rect { x: thumb_x + 2.0, y: thumb_y + 2.0, width: thumb_r * 2.0 - 4.0, height: thumb_r * 2.0 - 4.0 },
            color: Color::rgb(0.21, 0.43, 0.86), corner_radius: thumb_r - 2.0,
        });
    }
}

pub(crate) fn slider_style(width: f32) -> Style {
    Style { width: Size::Px(width), height: Size::Px(24.0), ..Default::default() }
}

pub(crate) fn slider_widget(tree: &mut LayoutTree, value: Signal<f32>, min: f32, max: f32) -> Slider {
    let node = tree.new_leaf(slider_style(200.0));
    Slider { node, value, min, max }
}

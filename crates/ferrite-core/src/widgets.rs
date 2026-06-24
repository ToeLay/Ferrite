use crate::{request_repaint, Color, DrawCommand, KeyCode, KeyEvent, Widget};
use ferrite_layout::{Direction, LayoutTree, NodeId, Rect, Size, Style};
use ferrite_reactive::Signal;
use std::cell::RefCell;
use std::rc::Rc;

// ── Container ────────────────────────────────────────────────────────────────

pub struct Container {
    node: NodeId,
    children: Vec<Box<dyn Widget>>,
    background: Option<Color>,
}
impl Container {
    pub fn background(mut self, color: Color) -> Self { self.background = Some(color); self }
}
impl Widget for Container {
    fn node_id(&self) -> NodeId { self.node }
    fn children(&self) -> &[Box<dyn Widget>] { &self.children }
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] { &mut self.children }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        if let Some(color) = self.background {
            out.push(DrawCommand::Rect { rect, color, corner_radius: 0.0 });
        }
    }
}

fn container(tree: &mut LayoutTree, mut style: Style, dir: Direction, children: Vec<Box<dyn Widget>>) -> Container {
    style.direction = dir;
    let ids: Vec<NodeId> = children.iter().map(|c| c.node_id()).collect();
    let node = tree.new_with_children(style, &ids);
    Container { node, children, background: None }
}

pub fn column(tree: &mut LayoutTree, style: Style, children: Vec<Box<dyn Widget>>) -> Container {
    container(tree, style, Direction::Column, children)
}
pub fn row(tree: &mut LayoutTree, style: Style, children: Vec<Box<dyn Widget>>) -> Container {
    container(tree, style, Direction::Row, children)
}

// ── Text ─────────────────────────────────────────────────────────────────────

pub struct Text {
    node: NodeId,
    content: Rc<RefCell<String>>,
    color: Color,
    size: f32,
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

fn text_node_style(char_count: f32, size: f32) -> Style {
    Style { width: Size::Px(char_count * size * 0.62), height: Size::Px(size * 1.4), ..Default::default() }
}
const DEFAULT_TEXT_SIZE: f32 = 16.0;

pub fn text(tree: &mut LayoutTree, content: impl Into<String>) -> Text {
    let s = content.into();
    let node = tree.new_leaf(text_node_style(s.chars().count() as f32, DEFAULT_TEXT_SIZE));
    Text { node, content: Rc::new(RefCell::new(s)), color: Color::BLACK, size: DEFAULT_TEXT_SIZE }
}

pub fn text_dyn(tree: &mut LayoutTree, f: impl Fn() -> String + 'static) -> Text {
    let initial = f();
    let node = tree.new_leaf(text_node_style(initial.chars().count() as f32, DEFAULT_TEXT_SIZE));
    let content = Rc::new(RefCell::new(initial));
    let c2 = content.clone();
    ferrite_reactive::create_effect(move || { *c2.borrow_mut() = f(); request_repaint(); });
    Text { node, content, color: Color::BLACK, size: DEFAULT_TEXT_SIZE }
}

// ── Button ───────────────────────────────────────────────────────────────────

pub struct Button {
    node: NodeId,
    label: String,
    on_click: Box<dyn FnMut()>,
    background: Color,
    foreground: Color,
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

pub fn button(tree: &mut LayoutTree, label: impl Into<String>, on_click: impl FnMut() + 'static) -> Button {
    let node = tree.new_leaf(Style { width: Size::Px(56.0), height: Size::Px(56.0), ..Default::default() });
    Button {
        node, label: label.into(), on_click: Box::new(on_click),
        background: Color::rgb(0.21, 0.43, 0.86), foreground: Color::WHITE,
    }
}

// ── TextInput ────────────────────────────────────────────────────────────────
//
// State lives in a `Signal<String>` owned by the caller — any reactive effect
// that reads `value.get()` updates automatically on keystroke, same as a
// signal driving a counter label. Cursor is a *char* index, not byte offset.

pub struct TextInput {
    node: NodeId,
    value: Signal<String>,
    placeholder: String,
    focused: bool,
    cursor: usize,
    font_size: f32,
    width: f32,
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

fn text_input_style(width: f32, font_size: f32) -> Style {
    Style { width: Size::Px(width), height: Size::Px(font_size * 2.4), ..Default::default() }
}

pub fn text_input(tree: &mut LayoutTree, value: Signal<String>, placeholder: impl Into<String>) -> TextInput {
    const DEFAULT_WIDTH: f32 = 280.0;
    const DEFAULT_FONT_SIZE: f32 = 16.0;
    let node = tree.new_leaf(text_input_style(DEFAULT_WIDTH, DEFAULT_FONT_SIZE));
    TextInput { node, value, placeholder: placeholder.into(), focused: false, cursor: 0,
        font_size: DEFAULT_FONT_SIZE, width: DEFAULT_WIDTH }
}

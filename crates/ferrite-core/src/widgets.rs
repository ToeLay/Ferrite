use crate::{request_repaint, Color, DrawCommand, KeyCode, KeyEvent, Widget};
use ferrite_layout::{LayoutTree, NodeId, Rect, Size, Style};
use ferrite_reactive::Signal;
use crate::theme::Theme;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub struct TextEditState {
    pub value: String,
    pub cursor: usize,
    pub selection_start: Option<usize>,
}

// ── Container ────────────────────────────────────────────────────────────────

pub struct Container {
    pub(crate) node: NodeId,
    pub(crate) children: Vec<Box<dyn Widget>>,
    pub(crate) background: Option<Color>,
    pub(crate) corner_radius: f32,
    pub(crate) clip: bool,
}
impl Container {
    pub fn background(mut self, color: Color) -> Self { self.background = Some(color); self }
    pub fn corner_radius(mut self, radius: f32) -> Self { self.corner_radius = radius; self }
}
impl Widget for Container {
    fn node_id(&self) -> NodeId { self.node }
    fn children(&self) -> &[Box<dyn Widget>] { &self.children }
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] { &mut self.children }
    fn update(&mut self, tree: &mut LayoutTree) {
        for child in &mut self.children { child.update(tree); }
    }
    fn paint(&self, tree: &ferrite_layout::LayoutTree, ox: f32, oy: f32, out: &mut Vec<crate::DrawCommand>) {
        let r = tree.layout(self.node);
        let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        
        self.paint_self(abs, out);
        
        if self.clip {
            out.push(crate::DrawCommand::PushClip { rect: abs });
        }
        
        for child in self.children() { 
            child.paint(tree, abs.x, abs.y, out); 
        }
        
        if self.clip {
            out.push(crate::DrawCommand::PopClip);
        }
    }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        if let Some(color) = self.background {
            out.push(DrawCommand::Rect { rect, color, corner_radius: self.corner_radius });
        }
    }
}

// ── Text ─────────────────────────────────────────────────────────────────────

pub struct Text {
    pub(crate) node: NodeId,
    pub(crate) content: Rc<RefCell<String>>,
    pub(crate) color: Color,
    pub(crate) size: f32,
    pub(crate) single_line: bool,
    pub(crate) version: std::rc::Rc<std::cell::Cell<u64>>,
}
impl Text {
    pub fn color(mut self, color: Color) -> Self { self.color = color; self }
}
impl Widget for Text {
    fn node_id(&self) -> NodeId { self.node }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        out.push(DrawCommand::Text {
            id: self.node.id(), version: self.version.get(),
            x: rect.x, y: rect.y,
            content: self.content.borrow().clone(),
            size: self.size, color: self.color,
            max_width: Some(rect.width),
            single_line: self.single_line,
            center: false,
        });
    }
}

pub(crate) const DEFAULT_TEXT_SIZE: f32 = 16.0;

// ── Button ───────────────────────────────────────────────────────────────────

pub struct Button {
    pub(crate) node: NodeId,
    pub(crate) label: String,
    pub(crate) on_click: Box<dyn FnMut()>,
    pub(crate) background: Color,
    pub(crate) foreground: Color,
    pub(crate) theme: Theme,
    pub(crate) focused: bool,
    pub(crate) hovered: ferrite_reactive::Signal<bool>,
    pub(crate) pressed: ferrite_reactive::Signal<bool>,
    pub(crate) anim: ferrite_reactive::Signal<f32>,
    pub(crate) font_size: f32,
}
impl Button {
    pub fn background(mut self, c: Color) -> Self { self.background = c; self }
    pub fn foreground(mut self, c: Color) -> Self { self.foreground = c; self }
}
impl Widget for Button {
    fn node_id(&self) -> NodeId { self.node }
    fn is_focusable(&self) -> bool { true }
    fn on_focus_change(&mut self, focused: bool) {
        self.focused = focused;
        request_repaint();
    }
    fn hover_signal(&self) -> Option<ferrite_reactive::Signal<bool>> { Some(self.hovered) }
    fn press_signal(&self) -> Option<ferrite_reactive::Signal<bool>> { Some(self.pressed) }
    fn on_key(&mut self, event: &crate::KeyEvent) -> bool {
        if self.focused {
            if event.key == crate::KeyCode::Return || event.key == crate::KeyCode::Char(' ') {
                self.on_click();
                return true;
            }
        }
        false
    }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        if self.focused {
            out.push(DrawCommand::StrokeRect {
                rect,
                color: self.theme.primary, corner_radius: 14.0, stroke_width: 2.0,
            });
        }
        
        // animate color based on state (0 = normal, 1 = hovered, 2 = pressed)
        let state = self.anim.get();
        let bg = if state > 1.0 {
            // Blend from hovered to pressed
            self.background.lighten(0.1).lerp(&self.background.darken(0.1), state - 1.0)
        } else {
            // Blend from normal to hovered
            self.background.lerp(&self.background.lighten(0.1), state)
        };
        
        // Inset button background to leave room for the focus ring (4px all sides)
        let bg_rect = Rect { x: rect.x + 4.0, y: rect.y + 4.0, width: rect.width - 8.0, height: rect.height - 8.0 };
        
        out.push(DrawCommand::Rect { rect: bg_rect, color: bg, corner_radius: 10.0 });
        out.push(DrawCommand::Text {
            id: self.node.id(), version: 0,
            x: bg_rect.x, y: bg_rect.y + bg_rect.height / 2.0 - self.font_size * 0.7,
            content: self.label.clone(), size: self.font_size, color: self.foreground,
            max_width: Some(bg_rect.width),
            single_line: true,
            center: true,
        });
    }
    fn on_click(&mut self) -> bool { (self.on_click)(); true }
}

// ── TextInput ────────────────────────────────────────────────────────────────

pub struct TextInput {
    pub(crate) node: NodeId,
    pub(crate) value: Signal<String>,
    pub(crate) placeholder: String,
    pub(crate) focused: bool,
    pub(crate) cursor: usize,
    pub(crate) selection_start: Option<usize>,
    pub(crate) scroll_x: f32,
    pub(crate) cursor_px: f32,
    pub(crate) selection_start_px: Option<f32>,
    pub(crate) font_size: f32,
    pub(crate) width: f32,
    pub(crate) theme: Theme,
    pub(crate) layout_dirty: bool,
    pub(crate) last_val: String,
    pub(crate) last_cursor: usize,
    pub(crate) last_selection: Option<usize>,
    pub(crate) last_width: f32,
    pub(crate) text_version: u64,
    pub(crate) undo_stack: Vec<TextEditState>,
    pub(crate) redo_stack: Vec<TextEditState>,
}

impl TextInput {
    pub fn placeholder(mut self, s: impl Into<String>) -> Self { self.placeholder = s.into(); self }

    pub fn width(mut self, tree: &mut LayoutTree, w: f32) -> Self {
        self.width = w;
        tree.set_style(self.node, text_input_style(w, self.font_size));
        self
    }

    fn save_state(&mut self) {
        let val = self.value.get();
        if let Some(last) = self.undo_stack.last() {
            if last.value == val { return; }
        }
        self.undo_stack.push(TextEditState {
            value: val,
            cursor: self.cursor,
            selection_start: self.selection_start,
        });
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            let current_val = self.value.get();
            self.redo_stack.push(TextEditState {
                value: current_val,
                cursor: self.cursor,
                selection_start: self.selection_start,
            });
            self.value.set(prev.value);
            self.cursor = prev.cursor;
            self.selection_start = prev.selection_start;
            self.layout_dirty = true;
            request_repaint();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            let current_val = self.value.get();
            self.undo_stack.push(TextEditState {
                value: current_val,
                cursor: self.cursor,
                selection_start: self.selection_start,
            });
            self.value.set(next.value);
            self.cursor = next.cursor;
            self.selection_start = next.selection_start;
            self.layout_dirty = true;
            request_repaint();
        }
    }

    pub fn font_size(mut self, tree: &mut LayoutTree, size: f32) -> Self {
        self.font_size = size;
        tree.set_style(self.node, text_input_style(self.width, size));
        self
    }
    
    // Returns (start, end) indices for the current selection in correct order
    fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|s| if s <= self.cursor { (s, self.cursor) } else { (self.cursor, s) })
    }
    
    // Deletes the currently selected text. Returns true if something was deleted.
    fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection_range() {
            if start != end {
                let val = self.value.get();
                let byte_start = val.char_indices().nth(start).map(|(i, _)| i).unwrap_or(val.len());
                let byte_end = val.char_indices().nth(end).map(|(i, _)| i).unwrap_or(val.len());
                let mut s = val.clone();
                s.replace_range(byte_start..byte_end, "");
                self.value.set(s);
                self.cursor = start;
                self.selection_start = None;
                return true;
            }
        }
        self.selection_start = None;
        false
    }
    
    // Update selection state based on shift key
    fn handle_shift(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            self.selection_start = None;
        }
    }
    fn ensure_cursor_visible(&mut self, width: f32, total_text_width: f32) {
        let pad = self.font_size * 0.5;
        let inner_w = width - 2.0 * pad;
        let cx = self.cursor_px;
        
        // If cursor is to the left of visible area
        if cx < self.scroll_x {
            self.scroll_x = cx.max(0.0);
        }
        // If cursor is to the right of visible area
        else if cx > self.scroll_x + inner_w {
            self.scroll_x = cx - inner_w + 2.0; // small margin
        }
        
        let max_scroll = (total_text_width - inner_w).max(0.0);
        if self.scroll_x > max_scroll {
            self.scroll_x = max_scroll;
        }
    }
}

impl Widget for TextInput {
    fn node_id(&self) -> NodeId { self.node }
    fn is_focusable(&self) -> bool { true }
    
    fn click_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<NodeId> {
        let r = tree.layout(self.node_id());
        let ax = ox + r.x;
        let ay = oy + r.y;
        if px < ax || py < ay || px > ax + r.width || py > ay + r.height { return None; }
        let pad = self.font_size * 0.5;
        let rel_x = px - (ax + pad) + self.scroll_x;
        let val = self.value.get();
        let found_idx;
        let mut _current_px = 0.0;
        
        if rel_x <= 0.0 {
            found_idx = 0;
        } else {
            found_idx = tree.char_at_x(self.node.id(), self.text_version, &val, self.font_size, rel_x, 0, true);
        }
        
        self.cursor = found_idx;
        self.selection_start = None;
        request_repaint();
        Some(self.node_id())
    }

    fn drag_at(&mut self, tree: &LayoutTree, ox: f32, _oy: f32, px: f32, _py: f32) -> bool {
        let r = tree.layout(self.node_id());
        let ax = ox + r.x;
        
        if self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        }
        let pad = self.font_size * 0.5;
        let rel_x = px - (ax + pad) + self.scroll_x;
        let val = self.value.get();
        let found_idx;
        
        if rel_x <= 0.0 {
            found_idx = 0;
        } else {
            found_idx = tree.char_at_x(self.node.id(), self.text_version, &val, self.font_size, rel_x, 0, true);
        }
        
        if self.cursor != found_idx {
            self.cursor = found_idx;
            request_repaint();
        }
        true
    }
    
    fn update(&mut self, tree: &mut LayoutTree) {
        let val = self.value.get();
        let r = tree.layout(self.node_id());
        if val != self.last_val || self.cursor != self.last_cursor || self.selection_start != self.last_selection || r.width != self.last_width {
            self.last_val = val.clone();
            self.last_cursor = self.cursor;
            self.last_selection = self.selection_start;
            self.last_width = r.width;
            self.layout_dirty = true;
            request_repaint();
        }
        if !self.layout_dirty { return; }
        self.layout_dirty = false;
        
        let char_count = val.chars().count();
        if self.cursor > char_count {
            self.cursor = char_count;
        }
        if let Some(s) = self.selection_start {
            if s > char_count {
                self.selection_start = Some(char_count);
            }
        }
        
        // Compute pixel offsets precisely using fontdue
        self.cursor_px = tree.char_x_at_index(self.node.id(), self.text_version, &val, self.font_size, self.cursor, 0, true);
        
        if let Some(s_start) = self.selection_start {
            self.selection_start_px = Some(tree.char_x_at_index(self.node.id(), self.text_version, &val, self.font_size, s_start, 0, true));
        } else {
            self.selection_start_px = None;
        }
        
        let (total_w, _) = tree.measure_text(self.node.id(), self.text_version, &val, self.font_size, None, true);
        self.ensure_cursor_visible(r.width, total_w);
    }

    fn on_focus_change(&mut self, focused: bool) {
        self.focused = focused;
        request_repaint();
    }

    fn on_key(&mut self, event: &KeyEvent) -> bool {
        let val = self.value.get();
        let char_count = val.chars().count();
        let is_cmd = event.modifiers.meta || event.modifiers.ctrl;
        
        match &event.key {
            KeyCode::Char(ch) => {
                if is_cmd && (*ch == 'a' || *ch == 'A') {
                    self.selection_start = Some(0);
                    self.cursor = char_count;
                    request_repaint();
                    return true;
                }
                if is_cmd && (*ch == 'c' || *ch == 'C') {
                    if let Some((start, end)) = self.selection_range() {
                        if start != end {
                            let text_val = self.value.get();
                            let chars: String = text_val.chars().skip(start).take(end - start).collect();
                            crate::clipboard::set_text(chars);
                        }
                    }
                    return true;
                }
                if is_cmd && (*ch == 'z' || *ch == 'Z') {
                    if event.modifiers.shift {
                        self.redo();
                    } else {
                        self.undo();
                    }
                    return true;
                }
                if is_cmd && (*ch == 'v' || *ch == 'V') {
                    if let Some(text) = crate::clipboard::get_text() {
                        self.save_state();
                        self.delete_selection();
                        let mut s = self.value.get();
                        let byte_pos = s.char_indices().nth(self.cursor).map(|(i,_)| i).unwrap_or(s.len());
                        s.insert_str(byte_pos, &text);
                        self.cursor += text.chars().count();
                        self.value.set(s);
                        request_repaint();
                    }
                    return true;
                }
                // Typing a normal char
                if !is_cmd {
                    self.save_state();
                    self.delete_selection();
                    let mut s = self.value.get();
                    let byte_pos = s.char_indices().nth(self.cursor).map(|(i,_)| i).unwrap_or(s.len());
                    s.insert(byte_pos, *ch);
                    self.cursor += 1;
                    self.value.set(s);
                    request_repaint();
                }
                true
            }
            KeyCode::Backspace => {
                self.save_state();
                if !self.delete_selection() && self.cursor > 0 {
                    let mut s = self.value.get();
                    let byte_pos = s.char_indices().nth(self.cursor - 1).map(|(i,_)| i).unwrap_or(0);
                    s.remove(byte_pos);
                    self.cursor -= 1;
                    self.value.set(s);
                    request_repaint();
                }
                true
            }
            KeyCode::Delete => {
                self.save_state();
                if !self.delete_selection() && self.cursor < char_count {
                    let mut s = self.value.get();
                    let byte_pos = s.char_indices().nth(self.cursor).map(|(i,_)| i).unwrap_or(s.len());
                    s.remove(byte_pos);
                    self.value.set(s);
                    request_repaint();
                }
                true
            }
            KeyCode::Left  => { 
                self.handle_shift(event.modifiers.shift);
                if is_cmd {
                    self.cursor = word_left(&val, self.cursor);
                } else if self.cursor > 0 { 
                    self.cursor -= 1; 
                } 
                request_repaint();
                true 
            }
            KeyCode::Right => { 
                self.handle_shift(event.modifiers.shift);
                if is_cmd {
                    self.cursor = word_right(&val, self.cursor);
                } else if self.cursor < char_count { 
                    self.cursor += 1; 
                } 
                request_repaint();
                true 
            }
            KeyCode::Home  => { 
                self.handle_shift(event.modifiers.shift);
                self.cursor = 0; request_repaint(); 
                true 
            }
            KeyCode::End   => { 
                self.handle_shift(event.modifiers.shift);
                self.cursor = char_count; request_repaint(); 
                true 
            }
            KeyCode::Tab | KeyCode::Escape => false,
            _ => false,
        }
    }

    fn on_double_click(&mut self) -> bool {
        let val = self.value.get();
        let (start, end) = word_bounds(&val, self.cursor);
        self.selection_start = Some(start);
        self.cursor = end;
        request_repaint();
        true
    }

    fn on_triple_click(&mut self) -> bool {
        let val = self.value.get();
        self.selection_start = Some(0);
        self.cursor = val.chars().count();
        request_repaint();
        true
    }


    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        let border_color = if self.focused { self.theme.primary } else { self.theme.muted };
        let pad = self.font_size * 0.5;
        out.push(DrawCommand::Rect { rect, color: border_color, corner_radius: self.theme.radius_md - 1.0 });
        let inner = Rect { x: rect.x + 1.5, y: rect.y + 1.5, width: rect.width - 3.0, height: rect.height - 3.0 };
        out.push(DrawCommand::Rect { rect: inner, color: self.theme.surface, corner_radius: self.theme.radius_md - 2.0 });
        out.push(DrawCommand::PushClip { rect: inner });
        
        let val = self.value.get();
        let text_y = rect.y + (rect.height / 2.0) - (self.font_size * 0.7);
        let base_x = rect.x + pad - self.scroll_x;
        
        if val.is_empty() {
            out.push(DrawCommand::Text { id: self.node.id(), version: self.text_version, x: base_x, y: text_y, content: self.placeholder.clone(),
                size: self.font_size, color: self.theme.muted, max_width: Some(rect.width - 2.0 * pad), single_line: true, center: false });
        } else {
            out.push(DrawCommand::Text { id: self.node.id(), version: self.text_version, x: base_x, y: text_y, content: val,
                size: self.font_size, color: self.theme.on_surface, max_width: Some(rect.width - 2.0 * pad), single_line: true, center: false });
        }
        if self.focused {
            let line_height = self.font_size * 1.4;
            let centering_offset = (line_height - self.font_size) / 2.0;
            
            // Draw selection box
            if let Some((start, end)) = self.selection_range() {
                if start != end {
                    let (s_px, e_px) = if start == self.cursor {
                        (self.cursor_px, self.selection_start_px.unwrap_or(0.0))
                    } else {
                        (self.selection_start_px.unwrap_or(0.0), self.cursor_px)
                    };
                    
                    let sx = base_x + s_px;
                    let ex = base_x + e_px;
                    out.push(DrawCommand::Rect {
                        rect: Rect { x: sx, y: text_y + centering_offset, width: ex - sx, height: self.font_size },
                        color: Color { a: 0.3, ..self.theme.primary },
                        corner_radius: 2.0,
                    });
                }
            }
            
            // Draw cursor
            let cx = base_x + self.cursor_px;
            out.push(DrawCommand::Rect {
                rect: Rect { x: cx, y: text_y + centering_offset, width: 2.0, height: self.font_size },
                color: self.theme.primary, corner_radius: 0.0,
            });
        }
        
        out.push(DrawCommand::PopClip);
    }
}

pub(crate) fn text_input_style(width: f32, font_size: f32) -> Style {
    Style { width: Size::Px(width), height: Size::Px(font_size * 2.4), ..Default::default() }
}

// ── ClickAbsorber ──────────────────────────────────────────────────────────

pub struct ClickAbsorber {
    pub(crate) node: NodeId,
    pub(crate) child: Option<Box<dyn Widget>>,
    pub(crate) on_click: Box<dyn FnMut()>,
}

impl Widget for ClickAbsorber {
    fn node_id(&self) -> NodeId { self.node }
    
    fn children(&self) -> &[Box<dyn Widget>] {
        match &self.child {
            Some(c) => std::slice::from_ref(c),
            None => &[],
        }
    }
    
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] {
        match &mut self.child {
            Some(c) => std::slice::from_mut(c),
            None => &mut [],
        }
    }
    
    fn update(&mut self, tree: &mut LayoutTree) {
        if let Some(c) = &mut self.child { c.update(tree); }
    }
    
    fn on_click(&mut self) -> bool {
        (self.on_click)();
        true
    }
}

// ── Spacer ───────────────────────────────────────────────────────────────────

pub struct Spacer {
    pub(crate) node: NodeId,
}

impl Widget for Spacer {
    fn node_id(&self) -> NodeId { self.node }
}

// ── Divider ────────────────────────────────────────────────────────────────

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

// ── Checkbox ─────────────────────────────────────────────────────────────────

pub struct Checkbox {
    pub(crate) node: NodeId,
    pub(crate) label_text: String,
    pub(crate) checked: Signal<bool>,
    pub(crate) anim: ferrite_reactive::Signal<f32>,
    pub(crate) font_size: f32,
    pub(crate) theme: Theme,
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
        let anim_val = self.anim.get();
        let border_color = if self.checked.get() { self.theme.primary } else { self.theme.muted };
        out.push(DrawCommand::Rect { rect: outer, color: border_color, corner_radius: self.theme.radius_sm });

        let inner = Rect { x: outer.x + 2.0, y: outer.y + 2.0, width: outer.width - 4.0, height: outer.height - 4.0 };
        
        // Background fill fades in
        if anim_val > 0.01 {
            out.push(DrawCommand::Rect { 
                rect: inner, 
                color: Color { a: self.theme.primary.a * anim_val, ..self.theme.primary }, 
                corner_radius: (self.theme.radius_sm - 2.0).max(0.0) 
            });
            
            // Checkmark scales and fades in
            let scale = 0.6 * anim_val;
            let mark_w = inner.width * scale;
            let mark_h = inner.height * scale;
            let mark = Rect {
                x: inner.x + (inner.width - mark_w) / 2.0, 
                y: inner.y + (inner.height - mark_h) / 2.0,
                width: mark_w, 
                height: mark_h,
            };
            out.push(DrawCommand::Rect { 
                rect: mark, 
                color: Color { a: self.theme.on_primary.a * anim_val, ..self.theme.on_primary }, 
                corner_radius: 1.0 
            });
        }
        
        if anim_val < 0.99 {
            // Draw surface color behind it if not fully animated
            out.push(DrawCommand::Rect { 
                rect: inner, 
                color: Color { a: self.theme.surface.a * (1.0 - anim_val), ..self.theme.surface }, 
                corner_radius: (self.theme.radius_sm - 2.0).max(0.0) 
            });
        }

        if !self.label_text.is_empty() {
            let tx = rect.x + box_size + self.theme.spacing;
            let ty = rect.y + (rect.height / 2.0) - (self.font_size * 0.7);
            out.push(DrawCommand::Text {
                id: self.node.id(), version: 0,
                x: tx, y: ty, content: self.label_text.clone(),
                size: self.font_size, color: self.theme.on_surface,
                max_width: Some(rect.width - box_size - self.theme.spacing),
                single_line: false,
                center: false,
            });
        }
    }
}

pub(crate) fn checkbox_style(tree: &ferrite_layout::LayoutTree, label: &str, font_size: f32) -> Style {
    let box_size = font_size * 1.2;
    let label_w = if !label.is_empty() {
        let (w, _) = tree.measure_text(0, 0, label, font_size, None, true);
        w + 8.0
    } else {
        0.0
    };
    let (_, h) = tree.measure_text(0, 0, "A", font_size, None, true);
    Style {
        width: Size::Px(box_size + label_w),
        height: Size::Px(box_size.max(h)),
        ..Default::default()
    }
}

// ── Slider ───────────────────────────────────────────────────────────────────

pub struct Slider {
    pub(crate) node: NodeId,
    pub(crate) value: Signal<f32>,
    pub(crate) min: f32,
    pub(crate) max: f32,
    pub(crate) theme: Theme,
}

impl Widget for Slider {
    fn node_id(&self) -> NodeId { self.node }

    fn click_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<NodeId> {
        let r = tree.layout(self.node_id());
        let ax = ox + r.x;
        let ay = oy + r.y;
        if px < ax || py < ay || px > ax + r.width || py > ay + r.height { return None; }
        let ratio = ((px - ax) / r.width).clamp(0.0, 1.0);
        self.value.set(self.min + (self.max - self.min) * ratio);
        request_repaint();
        Some(self.node_id())
    }

    fn drag_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, _py: f32) -> bool {
        let r = tree.layout(self.node_id());
        let ax = ox + r.x;
        let _ay = oy + r.y;
        // Unlike click, drag doesn't bounds check the Y axis so you don't lose
        // tracking if your mouse wanders up/down slightly while dragging horizontally
        if px < ax || px > ax + r.width {
            // we still clamp the value to the bounds
        }
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
            color: self.theme.muted, corner_radius: track_h / 2.0,
        });

        let fill_w = rect.width * ratio;
        if fill_w > 0.0 {
            out.push(DrawCommand::Rect {
                rect: Rect { x: rect.x, y: track_y, width: fill_w, height: track_h },
                color: self.theme.primary, corner_radius: track_h / 2.0,
            });
        }

        let thumb_r = self.theme.spacing;
        let thumb_cx = rect.x + rect.width * ratio;
        let thumb_x = (thumb_cx - thumb_r).max(rect.x);
        let thumb_y = rect.y + rect.height / 2.0 - thumb_r;
        out.push(DrawCommand::Rect {
            rect: Rect { x: thumb_x, y: thumb_y, width: thumb_r * 2.0, height: thumb_r * 2.0 },
            color: self.theme.surface, corner_radius: thumb_r,
        });
        out.push(DrawCommand::Rect {
            rect: Rect { x: thumb_x + 2.0, y: thumb_y + 2.0, width: thumb_r * 2.0 - 4.0, height: thumb_r * 2.0 - 4.0 },
            color: self.theme.primary, corner_radius: (thumb_r - 2.0).max(0.0),
        });
    }
}

pub(crate) fn slider_style(width: f32) -> Style {
    Style { width: Size::Px(width), height: Size::Px(24.0), ..Default::default() }
}

// ── Scroll ───────────────────────────────────────────────────────────────────

pub struct Scroll {
    pub(crate) node: NodeId,
    pub(crate) child: Box<dyn Widget>,
    pub(crate) scroll_x: f32,
    pub(crate) scroll_y: f32,
}

impl Widget for Scroll {
    fn node_id(&self) -> NodeId { self.node }
    fn children(&self) -> &[Box<dyn Widget>] { std::slice::from_ref(&self.child) }
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] { std::slice::from_mut(&mut self.child) }

    fn paint(&self, tree: &LayoutTree, ox: f32, oy: f32, out: &mut Vec<DrawCommand>) {
        let r = tree.layout(self.node_id());
        let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
        
        out.push(DrawCommand::PushClip { rect: abs });
        self.child.paint(tree, abs.x - self.scroll_x, abs.y - self.scroll_y, out);
        out.push(DrawCommand::PopClip);

        let child_layout = tree.layout(self.child.node_id());
        let max_y = (child_layout.height - r.height).max(0.0);
        if max_y > 0.0 {
            let thumb_height = (r.height / child_layout.height * r.height).max(20.0);
            let thumb_y = (self.scroll_y / max_y) * (r.height - thumb_height);
            out.push(DrawCommand::Rect {
                rect: Rect {
                    x: abs.x + abs.width - 8.0,
                    y: abs.y + thumb_y,
                    width: 6.0,
                    height: thumb_height,
                },
                color: crate::Color::rgba(0.5, 0.5, 0.5, 0.5),
                corner_radius: 3.0,
            });
        }
    }

    fn click_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<NodeId> {
        let r = tree.layout(self.node_id());
        let ax = ox + r.x; let ay = oy + r.y;
        if px < ax || py < ay || px > ax + r.width || py > ay + r.height { return None; }
        
        if let Some(id) = self.child.click_at(tree, ax - self.scroll_x, ay - self.scroll_y, px, py) {
            return Some(id);
        }
        if self.on_click() { Some(self.node_id()) } else { None }
    }

    fn drag_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> bool {
        let r = tree.layout(self.node_id());
        let ax = ox + r.x; let ay = oy + r.y;
        if px < ax || py < ay || px > ax + r.width || py > ay + r.height { return false; }
        
        if self.child.drag_at(tree, ax - self.scroll_x, ay - self.scroll_y, px, py) {
            return true;
        }
        false
    }

    fn dispatch_drag(&mut self, target: NodeId, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> bool {
        if self.node_id() == target {
            return self.drag_at(tree, ox, oy, px, py);
        }
        let r = tree.layout(self.node_id());
        let ax = ox + r.x; let ay = oy + r.y;
        self.child.dispatch_drag(target, tree, ax - self.scroll_x, ay - self.scroll_y, px, py)
    }

    fn scroll_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32, dx: f32, dy: f32) -> bool {
        let r = tree.layout(self.node_id());
        let ax = ox + r.x; let ay = oy + r.y;
        if px < ax || py < ay || px > ax + r.width || py > ay + r.height { return false; }
        
        if self.child.scroll_at(tree, ax - self.scroll_x, ay - self.scroll_y, px, py, dx, dy) {
            return true;
        }
        
        let child_layout = tree.layout(self.child.node_id());
        let max_x = (child_layout.width - r.width).max(0.0);
        let max_y = (child_layout.height - r.height).max(0.0);
        
        let new_x = (self.scroll_x - dx).clamp(0.0, max_x);
        let new_y = (self.scroll_y - dy).clamp(0.0, max_y);
        
        if (new_x - self.scroll_x).abs() > 0.001 || (new_y - self.scroll_y).abs() > 0.001 {
            self.scroll_x = new_x;
            self.scroll_y = new_y;
            request_repaint();
            true
        } else {
            false
        }
    }

    fn update(&mut self, tree: &mut LayoutTree) {
        self.child.update(tree);
    }

    fn find_focusable_at(&self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<NodeId> {
        let r = tree.layout(self.node_id());
        let ax = ox + r.x; let ay = oy + r.y;
        if px < ax || py < ay || px > ax + r.width || py > ay + r.height { return None; }
        
        if let Some(found) = self.child.find_focusable_at(tree, ax - self.scroll_x, ay - self.scroll_y, px, py) {
            return Some(found);
        }
        if self.is_focusable() { Some(self.node_id()) } else { None }
    }
}

// ── TextArea ─────────────────────────────────────────────────────────────────

pub struct TextArea {
    pub(crate) node: NodeId,
    pub(crate) value: Signal<String>,
    pub(crate) placeholder: String,
    pub(crate) focused: bool,
    pub(crate) cursor: usize,
    pub(crate) selection_start: Option<usize>,
    pub(crate) scroll_x: f32,
    pub(crate) scroll_y: f32,
    pub(crate) cursor_px: f32,
    pub(crate) cursor_py: f32,
    pub(crate) selection_start_px: Option<f32>,
    pub(crate) selection_start_py: Option<f32>,
    pub(crate) line_chars: Vec<usize>,
    pub(crate) line_height: f32,
    pub(crate) font_size: f32,
    pub(crate) theme: Theme,
    pub(crate) layout_dirty: bool,
    pub(crate) last_val: String,
    pub(crate) last_cursor: usize,
    pub(crate) last_selection: Option<usize>,
    pub(crate) last_width: f32,
    pub(crate) text_version: u64,
    pub(crate) undo_stack: Vec<TextEditState>,
    pub(crate) redo_stack: Vec<TextEditState>,
}

impl TextArea {
    fn save_state(&mut self) {
        let val = self.value.get();
        if let Some(last) = self.undo_stack.last() {
            if last.value == val { return; }
        }
        self.undo_stack.push(TextEditState {
            value: val,
            cursor: self.cursor,
            selection_start: self.selection_start,
        });
        if self.undo_stack.len() > 100 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo_stack.pop() {
            let current_val = self.value.get();
            self.redo_stack.push(TextEditState {
                value: current_val,
                cursor: self.cursor,
                selection_start: self.selection_start,
            });
            self.value.set(prev.value);
            self.cursor = prev.cursor;
            self.selection_start = prev.selection_start;
            self.layout_dirty = true;
            request_repaint();
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo_stack.pop() {
            let current_val = self.value.get();
            self.undo_stack.push(TextEditState {
                value: current_val,
                cursor: self.cursor,
                selection_start: self.selection_start,
            });
            self.value.set(next.value);
            self.cursor = next.cursor;
            self.selection_start = next.selection_start;
            self.layout_dirty = true;
            request_repaint();
        }
    }

    fn char_to_line_col(&self, index: usize) -> (usize, usize) {
        let val = self.value.get();
        let mut cur_line = 0;
        let mut cur_col = index;
        let mut chars_skipped = 0;
        for &lc in &self.line_chars {
            let ends_with_newline = lc > 0 && val.chars().nth(chars_skipped + lc - 1) == Some('\n');
            if cur_col < lc || (cur_col == lc && !ends_with_newline) {
                break;
            }
            cur_col -= lc;
            chars_skipped += lc;
            cur_line += 1;
        }
        (cur_line, cur_col)
    }

    pub fn placeholder(mut self, s: impl Into<String>) -> Self { self.placeholder = s.into(); self }
    pub fn font_size(mut self, size: f32) -> Self { self.font_size = size; self }
    
    fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection_start.map(|s| if s <= self.cursor { (s, self.cursor) } else { (self.cursor, s) })
    }
    
    fn delete_selection(&mut self) -> bool {
        if let Some((start, end)) = self.selection_range() {
            if start != end {
                let val = self.value.get();
                let byte_start = val.char_indices().nth(start).map(|(i, _)| i).unwrap_or(val.len());
                let byte_end = val.char_indices().nth(end).map(|(i, _)| i).unwrap_or(val.len());
                let mut s = val.clone();
                s.replace_range(byte_start..byte_end, "");
                self.value.set(s);
                self.cursor = start;
                self.selection_start = None;
                return true;
            }
        }
        self.selection_start = None;
        false
    }
    
    fn handle_shift(&mut self, shift: bool) {
        if shift {
            if self.selection_start.is_none() {
                self.selection_start = Some(self.cursor);
            }
        } else {
            self.selection_start = None;
        }
    }
    
    fn ensure_cursor_visible(&mut self, width: f32, height: f32, total_w: f32, total_h: f32) {
        let pad = self.font_size * 0.5;
        let inner_w = width - 2.0 * pad;
        let inner_h = height - 2.0 * pad;
        let cx = self.cursor_px;
        let cy = self.cursor_py;
        
        if cx < self.scroll_x {
            self.scroll_x = cx.max(0.0);
        } else if cx > self.scroll_x + inner_w {
            self.scroll_x = cx - inner_w + 2.0;
        }
        
        if cy < self.scroll_y {
            self.scroll_y = cy.max(0.0);
        } else if cy + self.line_height > self.scroll_y + inner_h {
            self.scroll_y = cy + self.line_height - inner_h + 2.0;
        }
        
        let max_scroll_x = (total_w - inner_w).max(0.0);
        if self.scroll_x > max_scroll_x {
            self.scroll_x = max_scroll_x;
        }
        let max_scroll_y = (total_h - inner_h).max(0.0);
        if self.scroll_y > max_scroll_y {
            self.scroll_y = max_scroll_y;
        }
    }
}

impl Widget for TextArea {
    fn node_id(&self) -> NodeId { self.node }
    fn is_focusable(&self) -> bool { true }
    
    fn click_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<NodeId> {
        let r = tree.layout(self.node_id());
        let ax = ox + r.x;
        let ay = oy + r.y;
        if px < ax || py < ay || px > ax + r.width || py > ay + r.height { return None; }
        
        let pad = self.font_size * 0.5;
        let rel_y = py - (ay + pad) + self.scroll_y;
        let rel_x = px - (ax + pad) + self.scroll_x;
        
        let line_height = self.line_height;
        let mut target_line = (rel_y / line_height).floor() as usize;
        if target_line >= self.line_chars.len() {
            target_line = self.line_chars.len().saturating_sub(1);
        }
        
        let val = self.value.get();
        let mut chars_skipped = 0;
        for i in 0..target_line {
            chars_skipped += self.line_chars[i];
        }
        
        let line_len = self.line_chars.get(target_line).copied().unwrap_or(0);
        let byte_start = val.char_indices().nth(chars_skipped).map(|(i,_)| i).unwrap_or(val.len());
        let byte_end = val.char_indices().nth(chars_skipped + line_len).map(|(i,_)| i).unwrap_or(val.len());
        
        let line_str = &val[byte_start..byte_end];
        let mut found_col;
        
        if rel_x <= 0.0 {
            found_col = 0;
        } else {
            found_col = tree.char_at_x(self.node.id(), self.text_version, &val, self.font_size, rel_x, target_line, false);
            if found_col > line_len { found_col = line_len; }
        }
        
        // Exclude the trailing \n from being clickable directly at the end
        if found_col > 0 && found_col == line_len && line_str.ends_with('\n') {
            found_col -= 1;
        }
        
        self.cursor = chars_skipped + found_col;
        self.selection_start = None;
        request_repaint();
        Some(self.node_id())
    }

    fn drag_at(&mut self, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> bool {
        let r = tree.layout(self.node_id());
        let ax = ox + r.x;
        let ay = oy + r.y;
        
        if self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        }
        
        let pad = self.font_size * 0.5;
        let rel_y = py - (ay + pad) + self.scroll_y;
        let rel_x = px - (ax + pad) + self.scroll_x;
        
        let line_height = self.line_height;
        let mut target_line = (rel_y / line_height).floor() as usize;
        if target_line >= self.line_chars.len() {
            target_line = self.line_chars.len().saturating_sub(1);
        }
        
        let val = self.value.get();
        let mut chars_skipped = 0;
        for i in 0..target_line {
            chars_skipped += self.line_chars[i];
        }
        
        let line_len = self.line_chars.get(target_line).copied().unwrap_or(0);
        let byte_start = val.char_indices().nth(chars_skipped).map(|(i,_)| i).unwrap_or(val.len());
        let byte_end = val.char_indices().nth(chars_skipped + line_len).map(|(i,_)| i).unwrap_or(val.len());
        
        let line_str = &val[byte_start..byte_end];
        let mut found_col;
        
        if rel_x <= 0.0 {
            found_col = 0;
        } else {
            found_col = tree.char_at_x(self.node.id(), self.text_version, &val, self.font_size, rel_x, target_line, false);
            if found_col > line_len { found_col = line_len; }
        }
        
        if found_col > 0 && found_col == line_len && line_str.ends_with('\n') {
            found_col -= 1;
        }
        
        let idx = chars_skipped + found_col;
        if self.cursor != idx {
            self.cursor = idx;
            request_repaint();
        }
        true
    }
    
    fn update(&mut self, tree: &mut LayoutTree) {
        let val = self.value.get();
        let r = tree.layout(self.node_id());
        if val != self.last_val || self.cursor != self.last_cursor || self.selection_start != self.last_selection || r.width != self.last_width {
            self.last_val = val.clone();
            self.last_cursor = self.cursor;
            self.last_selection = self.selection_start;
            self.last_width = r.width;
            self.layout_dirty = true;
            request_repaint();
        }
        if !self.layout_dirty { return; }
        self.layout_dirty = false;
        
        let char_count = val.chars().count();
        if self.cursor > char_count {
            self.cursor = char_count;
        }
        if let Some(s) = self.selection_start {
            if s > char_count {
                self.selection_start = Some(char_count);
            }
        }
        
        let pad = self.font_size * 0.5;
        let inner_w = r.width - 2.0 * pad;
        
        let (_, lh) = tree.measure_text(0, 0, "A", self.font_size, None, false);
        self.line_height = lh;
        
        let (total_w, total_h) = tree.measure_text(self.node.id(), self.text_version, &val, self.font_size, Some(inner_w), false);
        
        let line_chars = tree.wrap_lines(self.node.id(), self.text_version, &val, self.font_size, inner_w);
        self.line_chars = line_chars.clone();
        
        let (cur_line, cur_col) = self.char_to_line_col(self.cursor);
        
        self.cursor_px = tree.char_x_at_index(self.node.id(), self.text_version, &val, self.font_size, cur_col, cur_line, false);
        let line_height = self.line_height;
        self.cursor_py = cur_line as f32 * line_height;
        
        // Similar for selection
        if let Some(s) = self.selection_start {
            let (s_line, s_col) = self.char_to_line_col(s);
            
            self.selection_start_px = Some(tree.char_x_at_index(self.node.id(), self.text_version, &val, self.font_size, s_col, s_line, false));
            self.selection_start_py = Some(s_line as f32 * line_height);
        } else {
            self.selection_start_px = None;
            self.selection_start_py = None;
        }
        
        self.ensure_cursor_visible(r.width, r.height, total_w, total_h);
    }

    fn on_focus_change(&mut self, focused: bool) {
        self.focused = focused;
        request_repaint();
    }

    fn on_key(&mut self, event: &KeyEvent) -> bool {
        let val = self.value.get();
        let char_count = val.chars().count();
        let is_cmd = event.modifiers.meta || event.modifiers.ctrl;
        
        match &event.key {
            KeyCode::Char(ch) => {
                if is_cmd && (*ch == 'a' || *ch == 'A') {
                    self.selection_start = Some(0);
                    self.cursor = char_count;
                    self.layout_dirty = true;
                    request_repaint();
                    return true;
                }
                if is_cmd && (*ch == 'c' || *ch == 'C') {
                    if let Some((start, end)) = self.selection_range() {
                        if start != end {
                            let text_val = self.value.get();
                            let chars: String = text_val.chars().skip(start).take(end - start).collect();
                            crate::clipboard::set_text(chars);
                        }
                    }
                    return true;
                }
                if is_cmd && (*ch == 'z' || *ch == 'Z') {
                    if event.modifiers.shift {
                        self.redo();
                    } else {
                        self.undo();
                    }
                    return true;
                }
                if is_cmd && (*ch == 'v' || *ch == 'V') {
                    if let Some(text) = crate::clipboard::get_text() {
                        self.save_state();
                        self.delete_selection();
                        let mut s = self.value.get();
                        let byte_pos = s.char_indices().nth(self.cursor).map(|(i,_)| i).unwrap_or(s.len());
                        s.insert_str(byte_pos, &text);
                        self.cursor += text.chars().count();
                        self.value.set(s);
                        request_repaint();
                    }
                    return true;
                }
                if !is_cmd {
                    self.save_state();
                    self.delete_selection();
                    let mut s = self.value.get();
                    let byte_pos = s.char_indices().nth(self.cursor).map(|(i,_)| i).unwrap_or(s.len());
                    s.insert(byte_pos, *ch);
                    self.cursor += 1;
                    self.value.set(s);
                    request_repaint();
                }
                true
            }
            KeyCode::Return => {
                self.save_state();
                self.delete_selection();
                let mut s = self.value.get();
                let byte_pos = s.char_indices().nth(self.cursor).map(|(i,_)| i).unwrap_or(s.len());
                s.insert(byte_pos, '\n');
                self.cursor += 1;
                self.value.set(s);
                request_repaint();
                true
            }
            KeyCode::Backspace => {
                self.save_state();
                if !self.delete_selection() && self.cursor > 0 {
                    let mut s = self.value.get();
                    let byte_pos = s.char_indices().nth(self.cursor - 1).map(|(i,_)| i).unwrap_or(0);
                    s.remove(byte_pos);
                    self.cursor -= 1;
                    self.value.set(s);
                    request_repaint();
                }
                true
            }
            KeyCode::Delete => {
                self.save_state();
                if !self.delete_selection() && self.cursor < char_count {
                    let mut s = self.value.get();
                    let byte_pos = s.char_indices().nth(self.cursor).map(|(i,_)| i).unwrap_or(s.len());
                    s.remove(byte_pos);
                    self.value.set(s);
                    request_repaint();
                }
                true
            }
            KeyCode::Left  => { 
                self.handle_shift(event.modifiers.shift);
                if is_cmd {
                    self.cursor = word_left(&val, self.cursor);
                } else if self.cursor > 0 { 
                    self.cursor -= 1; 
                } 
                request_repaint();
                true 
            }
            KeyCode::Right => { 
                self.handle_shift(event.modifiers.shift);
                if is_cmd {
                    self.cursor = word_right(&val, self.cursor);
                } else if self.cursor < char_count { 
                    self.cursor += 1; 
                } 
                request_repaint();
                true 
            }
            KeyCode::Up | KeyCode::Down => {
                self.handle_shift(event.modifiers.shift);
                let (cur_line, cur_col) = self.char_to_line_col(self.cursor);
                let is_up = match event.key { KeyCode::Up => true, _ => false };
                
                let target_line = if is_up { cur_line.saturating_sub(1) } else { cur_line + 1 };
                
                if target_line < self.line_chars.len() {
                    let target_len = self.line_chars[target_line];
                    let new_col = cur_col.min(target_len.saturating_sub(1));
                    let mut skipped = 0;
                    for i in 0..target_line { skipped += self.line_chars[i]; }
                    self.cursor = skipped + new_col;
                    self.layout_dirty = true;
                    request_repaint();
                }
                true
            }
            KeyCode::Home  => { 
                self.handle_shift(event.modifiers.shift);
                self.cursor = 0; 
                self.layout_dirty = true;
                request_repaint(); 
                true 
            }
            KeyCode::End   => { 
                self.handle_shift(event.modifiers.shift);
                self.cursor = char_count; 
                self.layout_dirty = true;
                request_repaint(); 
                true 
            }
            KeyCode::Tab | KeyCode::Escape => false,

        }
    }

    fn on_double_click(&mut self) -> bool {
        let val = self.value.get();
        let (start, end) = word_bounds(&val, self.cursor);
        self.selection_start = Some(start);
        self.cursor = end;
        request_repaint();
        true
    }

    fn on_triple_click(&mut self) -> bool {
        let val = self.value.get();
        let (cur_line, _) = self.char_to_line_col(self.cursor);
        
        // Find line bounds
        let lines: Vec<&str> = val.split('\n').collect();
        if cur_line < lines.len() {
            let mut start_idx = 0;
            for i in 0..cur_line {
                start_idx += lines[i].chars().count() + 1; // +1 for '\n'
            }
            let end_idx = start_idx + lines[cur_line].chars().count();
            
            self.selection_start = Some(start_idx);
            self.cursor = end_idx;
            request_repaint();
            return true;
        }
        false
    }

    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        let border_color = if self.focused { self.theme.primary } else { self.theme.muted };
        let pad = self.font_size * 0.5;
        out.push(DrawCommand::Rect { rect, color: border_color, corner_radius: self.theme.radius_md - 1.0 });
        let inner = Rect { x: rect.x + 1.5, y: rect.y + 1.5, width: rect.width - 3.0, height: rect.height - 3.0 };
        out.push(DrawCommand::Rect { rect: inner, color: self.theme.surface, corner_radius: self.theme.radius_md - 2.0 });
        out.push(DrawCommand::PushClip { rect: inner });
        
        let val = self.value.get();
        let text_y = rect.y + pad - self.scroll_y;
        let base_x = rect.x + pad - self.scroll_x;
        
        if val.is_empty() {
            out.push(DrawCommand::Text { id: self.node.id(), version: self.text_version, x: base_x, y: text_y, content: self.placeholder.clone(),
                size: self.font_size, color: self.theme.muted, max_width: Some(rect.width - 2.0 * pad), single_line: false, center: false });
        } else {
            out.push(DrawCommand::Text { id: self.node.id(), version: self.text_version, x: base_x, y: text_y, content: val,
                size: self.font_size, color: self.theme.on_surface, max_width: Some(rect.width - 2.0 * pad), single_line: false, center: false });
        }
        
        
        if self.focused {
            let line_height = self.line_height;
            let centering_offset = (line_height - self.font_size) / 2.0;
            // Draw multi-line selection box
            if let Some((start, end)) = self.selection_range() {
                if start != end {
                    // Quick and dirty selection drawing: just drawing block spanning lines
                    let mut s_px = self.selection_start_px.unwrap_or(0.0);
                    let mut s_py = self.selection_start_py.unwrap_or(0.0);
                    let mut c_px = self.cursor_px;
                    let mut c_py = self.cursor_py;
                    
                    if start == self.cursor {
                        std::mem::swap(&mut s_px, &mut c_px);
                        std::mem::swap(&mut s_py, &mut c_py);
                    }
                    
                    let start_py = s_py;
                    let start_px = s_px;
                    let end_py = c_py;
                    let end_px = c_px;
                    
                    let inner_w = rect.width - 2.0 * pad;
                    
                    if (start_py - end_py).abs() < 1.0 { // Same line
                        let sx = base_x + start_px.min(end_px);
                        let ex = base_x + start_px.max(end_px);
                        out.push(DrawCommand::Rect {
                            rect: Rect { x: sx, y: text_y + start_py + centering_offset, width: ex - sx, height: self.font_size },
                            color: Color { a: 0.3, ..self.theme.primary }, corner_radius: 2.0,
                        });
                    } else { // Multiple lines
                        // 1. first line
                        out.push(DrawCommand::Rect {
                            rect: Rect { x: base_x + start_px, y: text_y + start_py + centering_offset, width: inner_w - start_px, height: self.font_size },
                            color: Color { a: 0.3, ..self.theme.primary }, corner_radius: 2.0,
                        });
                        // 2. middle lines
                        let mid_lines = ((end_py - start_py) / line_height).round() as i32 - 1;
                        if mid_lines > 0 {
                            for m in 0..mid_lines {
                                out.push(DrawCommand::Rect {
                                    rect: Rect { x: base_x, y: text_y + start_py + line_height * (m + 1) as f32 + centering_offset, width: inner_w, height: self.font_size },
                                    color: Color { a: 0.3, ..self.theme.primary }, corner_radius: 2.0,
                                });
                            }
                        }
                        // 3. last line
                        out.push(DrawCommand::Rect {
                            rect: Rect { x: base_x, y: text_y + end_py + centering_offset, width: end_px, height: self.font_size },
                            color: Color { a: 0.3, ..self.theme.primary }, corner_radius: 2.0,
                        });
                    }
                }
            }
            
            // Draw cursor
            let cx = base_x + self.cursor_px;
            let cy = text_y + self.cursor_py + centering_offset;
            out.push(DrawCommand::Rect {
                rect: Rect { x: cx, y: cy, width: 2.0, height: self.font_size },
                color: self.theme.primary, corner_radius: 0.0,
            });
        }
        
        out.push(DrawCommand::PopClip);
    }
}

fn word_left(s: &str, mut cursor: usize) -> usize {
    let chars: Vec<char> = s.chars().collect();
    if cursor == 0 { return 0; }
    cursor -= 1;
    while cursor > 0 && chars[cursor].is_whitespace() { cursor -= 1; }
    while cursor > 0 && !chars[cursor - 1].is_whitespace() { cursor -= 1; }
    cursor
}

fn word_right(s: &str, mut cursor: usize) -> usize {
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    if cursor >= len { return len; }
    while cursor < len && chars[cursor].is_whitespace() { cursor += 1; }
    while cursor < len && !chars[cursor].is_whitespace() { cursor += 1; }
    cursor
}

fn word_bounds(s: &str, cursor: usize) -> (usize, usize) {
    let chars: Vec<char> = s.chars().collect();
    if chars.is_empty() { return (0, 0); }
    let len = chars.len();
    
    let mut pos = if cursor >= len { len - 1 } else { cursor };
    
    if pos > 0 && chars[pos].is_whitespace() && !chars[pos - 1].is_whitespace() {
        pos -= 1;
    }
    
    let is_ws = chars[pos].is_whitespace();
    
    let mut start = pos;
    let mut end = pos;
    
    if is_ws {
        while start > 0 && chars[start - 1].is_whitespace() { start -= 1; }
        while end < len && chars[end].is_whitespace() { end += 1; }
    } else {
        while start > 0 && !chars[start - 1].is_whitespace() { start -= 1; }
        while end < len && !chars[end].is_whitespace() { end += 1; }
    }
    
    (start, end)
}

// ── Portal (Overlay) ─────────────────────────────────────────────────────────

pub struct PortalWidget {
    pub(crate) node: NodeId,
    pub(crate) show: Signal<bool>,
    pub(crate) content: Box<dyn Fn() -> crate::view::AnyView>,
    pub(crate) active_overlay: Option<crate::overlay::OverlayId>,
    pub(crate) last_show: bool,
}

impl Widget for PortalWidget {
    fn node_id(&self) -> NodeId { self.node }
    fn children(&self) -> &[Box<dyn Widget>] { &[] }
    fn children_mut(&mut self) -> &mut [Box<dyn Widget>] { &mut [] }

    fn update(&mut self, _tree: &mut LayoutTree) {
        self.show.track();
        let current_show = self.show.get();
        if current_show != self.last_show {
            self.last_show = current_show;
            if current_show {
                if self.active_overlay.is_none() {
                    let view = (self.content)();
                    self.active_overlay = Some(crate::overlay::show_overlay(view));
                }
            } else {
                if let Some(id) = self.active_overlay.take() {
                    crate::overlay::remove_overlay(id);
                }
            }
        }
    }
}

impl Drop for PortalWidget {
    fn drop(&mut self) {
        if let Some(id) = self.active_overlay.take() {
            crate::overlay::remove_overlay(id);
        }
    }
}

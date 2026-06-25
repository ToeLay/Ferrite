use crate::{request_repaint, Color, DrawCommand, KeyCode, KeyEvent, Widget};
use ferrite_layout::{LayoutTree, NodeId, Rect, Size, Style};
use ferrite_reactive::Signal;
use crate::theme::Theme;
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
    fn update(&mut self, tree: &mut LayoutTree) {
        for child in &mut self.children { child.update(tree); }
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
}
impl Text {
    pub fn color(mut self, color: Color) -> Self { self.color = color; self }
}
impl Widget for Text {
    fn node_id(&self) -> NodeId { self.node }
    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        out.push(DrawCommand::Text {
            x: rect.x, y: rect.y,
            content: self.content.borrow().clone(),
            size: self.size, color: self.color,
            max_width: Some(rect.width),
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
            max_width: Some(rect.width - 36.0),
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
        let pad = 10.0;
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
        
        let rel_x = px - (ax + 10.0) + self.scroll_x;
        let val = self.value.get();
        let mut found_idx = val.chars().count();
        let mut current_px = 0.0;
        
        if rel_x <= 0.0 {
            found_idx = 0;
        } else {
            for (i, (byte_idx, ch)) in val.char_indices().enumerate() {
                let s = &val[..byte_idx];
                let (w, _) = tree.measure_text(s, self.font_size);
                let (cw, _) = tree.measure_text(&ch.to_string(), self.font_size);
                if rel_x < w + cw / 2.0 {
                    found_idx = i;
                    break;
                }
            }
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
        
        let rel_x = px - (ax + 10.0) + self.scroll_x;
        let val = self.value.get();
        let mut found_idx = val.chars().count();
        
        if rel_x <= 0.0 {
            found_idx = 0;
        } else {
            for (i, (byte_idx, ch)) in val.char_indices().enumerate() {
                let s = &val[..byte_idx];
                let (w, _) = tree.measure_text(s, self.font_size);
                let (cw, _) = tree.measure_text(&ch.to_string(), self.font_size);
                if rel_x < w + cw / 2.0 {
                    found_idx = i;
                    break;
                }
            }
        }
        
        if self.cursor != found_idx {
            self.cursor = found_idx;
            request_repaint();
        }
        true
    }
    
    fn update(&mut self, tree: &mut LayoutTree) {
        let val = self.value.get();
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
        let byte_pos = val.char_indices().nth(self.cursor).map(|(i, _)| i).unwrap_or(val.len());
        let s = &val[..byte_pos];
        let (w, _) = tree.measure_text(s, self.font_size);
        self.cursor_px = w;
        
        if let Some(s_start) = self.selection_start {
            let s_byte = val.char_indices().nth(s_start).map(|(i, _)| i).unwrap_or(val.len());
            let s_str = &val[..s_byte];
            let (sw, _) = tree.measure_text(s_str, self.font_size);
            self.selection_start_px = Some(sw);
        } else {
            self.selection_start_px = None;
        }
        
        let r = tree.layout(self.node_id());
        let (total_w, _) = tree.measure_text(&val, self.font_size);
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
                if is_cmd && (*ch == 'v' || *ch == 'V') {
                    if let Some(text) = crate::clipboard::get_text() {
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
                if self.cursor > 0 { self.cursor -= 1; request_repaint(); } 
                true 
            }
            KeyCode::Right => { 
                self.handle_shift(event.modifiers.shift);
                if self.cursor < char_count { self.cursor += 1; request_repaint(); } 
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

    fn paint_self(&self, rect: Rect, out: &mut Vec<DrawCommand>) {
        let border_color = if self.focused { self.theme.primary } else { self.theme.muted };
        let pad = 10.0_f32;
        out.push(DrawCommand::Rect { rect, color: border_color, corner_radius: self.theme.radius_md - 1.0 });
        let inner = Rect { x: rect.x + 1.5, y: rect.y + 1.5, width: rect.width - 3.0, height: rect.height - 3.0 };
        out.push(DrawCommand::Rect { rect: inner, color: self.theme.surface, corner_radius: self.theme.radius_md - 2.0 });
        out.push(DrawCommand::PushClip { rect: inner });
        
        let val = self.value.get();
        let text_y = rect.y + (rect.height - self.font_size) / 2.0;
        let base_x = rect.x + pad - self.scroll_x;
        
        if val.is_empty() {
            out.push(DrawCommand::Text { x: base_x, y: text_y, content: self.placeholder.clone(),
                size: self.font_size, color: self.theme.muted, max_width: None });
        } else {
            out.push(DrawCommand::Text { x: base_x, y: text_y, content: val,
                size: self.font_size, color: self.theme.on_surface, max_width: None });
        }
        if self.focused {
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
                        rect: Rect { x: sx, y: text_y, width: ex - sx, height: self.font_size },
                        color: Color { a: 0.3, ..self.theme.primary },
                        corner_radius: 2.0,
                    });
                }
            }
            
            // Draw cursor
            let cx = base_x + self.cursor_px;
            out.push(DrawCommand::Rect {
                rect: Rect { x: cx, y: text_y, width: 2.0, height: self.font_size },
                color: self.theme.primary, corner_radius: 0.0,
            });
        }
        
        out.push(DrawCommand::PopClip);
    }
}

pub(crate) fn text_input_style(width: f32, font_size: f32) -> Style {
    Style { width: Size::Px(width), height: Size::Px(font_size * 2.4), ..Default::default() }
}

// ── Spacer ───────────────────────────────────────────────────────────────────

pub struct Spacer {
    pub(crate) node: NodeId,
}

impl Widget for Spacer {
    fn node_id(&self) -> NodeId { self.node }
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
            let ty = rect.y + (rect.height - self.font_size) / 2.0;
            out.push(DrawCommand::Text {
                x: tx, y: ty, content: self.label_text.clone(),
                size: self.font_size, color: self.theme.on_surface,
                max_width: Some(rect.width - box_size - self.theme.spacing),
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

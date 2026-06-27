use crate::{DrawCommand, KeyEvent, Widget};
use ferrite_layout::{LayoutTree, NodeId, Rect};

pub struct App {
    tree: LayoutTree,
    root: Box<dyn Widget>,
    overlays: Vec<(crate::overlay::OverlayId, Box<dyn Widget>)>,
    focused: Option<NodeId>,
    active_drag: Option<NodeId>,
    last_frame: Option<std::time::Instant>,
    hover_pos: Option<(f32, f32)>,
    hover_time: f32,
    hovered_tooltip: Option<String>,
    hovered_node: Option<NodeId>,
    pressed_node: Option<NodeId>,
}

impl App {
    pub fn new(tree: LayoutTree, root: Box<dyn Widget>) -> Self {
        ferrite_reactive::animation::set_wake_up(crate::dirty::request_repaint);
        App { 
            tree, 
            root, 
            overlays: Vec::new(), 
            focused: None, 
            active_drag: None, 
            last_frame: None,
            hover_pos: None,
            hover_time: 0.0,
            hovered_tooltip: None,
            hovered_node: None,
            pressed_node: None,
        }
    }

    pub fn layout_tree(&self) -> &LayoutTree { &self.tree }
    pub fn root_node_id(&self) -> NodeId { self.root.node_id() }
    pub fn root(&self) -> &dyn Widget { self.root.as_ref() }

    pub fn set_hover_pos(&mut self, pos: Option<(f32, f32)>) {
        self.hover_pos = pos;
        
        let new_hover = pos.and_then(|(x, y)| {
            let mut found = None;
            for (_, overlay) in self.overlays.iter().rev() {
                if let Some(f) = Self::find_hover_signal_at(overlay.as_ref(), &self.tree, 0.0, 0.0, x, y) {
                    found = Some(f);
                    break;
                }
            }
            if found.is_none() {
                found = Self::find_hover_signal_at(self.root.as_ref(), &self.tree, 0.0, 0.0, x, y);
            }
            found
        });

        if new_hover.map(|(n, _)| n) != self.hovered_node {
            if let Some(old) = self.hovered_node {
                if let Some((_, sig)) = Self::find_signal_by_node(self.root.as_ref(), &self.tree, old, true).or_else(|| {
                    self.overlays.iter().find_map(|(_, o)| Self::find_signal_by_node(o.as_ref(), &self.tree, old, true))
                }) {
                    sig.set(false);
                }
            }
            if let Some((new_id, new_sig)) = new_hover {
                new_sig.set(true);
                self.hovered_node = Some(new_id);
            } else {
                self.hovered_node = None;
            }
        }
    }

    fn find_hover_signal_at(w: &dyn Widget, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<(NodeId, ferrite_reactive::Signal<bool>)> {
        let r = tree.layout(w.node_id());
        let ax = ox + r.x; let ay = oy + r.y;
        if px < ax || py < ay || px > ax + r.width || py > ay + r.height { return None; }
        for child in w.children().iter().rev() {
            if let Some(found) = Self::find_hover_signal_at(child.as_ref(), tree, ax, ay, px, py) { return Some(found); }
        }
        if let Some(sig) = w.hover_signal() { Some((w.node_id(), sig)) } else { None }
    }

    fn find_press_signal_at(w: &dyn Widget, tree: &LayoutTree, ox: f32, oy: f32, px: f32, py: f32) -> Option<(NodeId, ferrite_reactive::Signal<bool>)> {
        let r = tree.layout(w.node_id());
        let ax = ox + r.x; let ay = oy + r.y;
        if px < ax || py < ay || px > ax + r.width || py > ay + r.height { return None; }
        for child in w.children().iter().rev() {
            if let Some(found) = Self::find_press_signal_at(child.as_ref(), tree, ax, ay, px, py) { return Some(found); }
        }
        if let Some(sig) = w.press_signal() { Some((w.node_id(), sig)) } else { None }
    }

    fn find_signal_by_node(w: &dyn Widget, tree: &LayoutTree, target: NodeId, hover: bool) -> Option<(NodeId, ferrite_reactive::Signal<bool>)> {
        if w.node_id() == target {
            if hover {
                if let Some(sig) = w.hover_signal() { return Some((w.node_id(), sig)); }
            } else {
                if let Some(sig) = w.press_signal() { return Some((w.node_id(), sig)); }
            }
        }
        for child in w.children() {
            if let Some(found) = Self::find_signal_by_node(child.as_ref(), tree, target, hover) { return Some(found); }
        }
        None
    }

    pub fn absolute_rect(&self, target: NodeId) -> Option<Rect> {
        fn walk(w: &dyn Widget, tree: &LayoutTree, ox: f32, oy: f32, target: NodeId) -> Option<Rect> {
            let r = tree.layout(w.node_id());
            let abs = Rect { x: ox + r.x, y: oy + r.y, width: r.width, height: r.height };
            if w.node_id() == target { return Some(abs); }
            for child in w.children() {
                if let Some(f) = walk(child.as_ref(), tree, abs.x, abs.y, target) { return Some(f); }
            }
            None
        }
        walk(self.root.as_ref(), &self.tree, 0.0, 0.0, target)
    }

    pub fn render(&mut self, width: f32, height: f32) -> Vec<DrawCommand> {
        let now = std::time::Instant::now();
        let dt = if let Some(last) = self.last_frame {
            now.duration_since(last).as_secs_f32()
        } else {
            1.0 / 60.0
        };
        self.last_frame = Some(now);

        // Tick animations (this might request repaint if animations are still running)
        ferrite_reactive::animation::tick_animations(dt);

        let mut drain_overlays = |tree: &mut ferrite_layout::LayoutTree, overlays: &mut Vec<(crate::overlay::OverlayId, Box<dyn crate::Widget>)>| {
            crate::overlay::PENDING_OVERLAYS.with(|o| {
                for (id, view) in o.borrow_mut().drain(..) {
                    let widget = view.build(tree);
                    overlays.push((id, widget));
                }
            });
            crate::overlay::REMOVED_OVERLAYS.with(|o| {
                for id in o.borrow_mut().drain(..) {
                    if let Some(pos) = overlays.iter().position(|(oid, _)| *oid == id) {
                        let (_, widget) = overlays.remove(pos);
                        tree.remove(widget.node_id());
                    }
                }
            });
        };

        drain_overlays(&mut self.tree, &mut self.overlays);

        self.root.update(&mut self.tree);
        for (_, overlay) in &mut self.overlays {
            overlay.update(&mut self.tree);
        }

        drain_overlays(&mut self.tree, &mut self.overlays);

        let dirty_nodes = crate::dirty::take_dirty_nodes();
        for node in dirty_nodes {
            self.tree.mark_dirty(node);
        }
        
        self.tree.compute(self.root.node_id(), width, height);
        for (_, overlay) in &mut self.overlays {
            self.tree.compute(overlay.node_id(), width, height);
        }
        
        let mut out = Vec::new();
        self.root.paint(&self.tree, 0.0, 0.0, &mut out);
        for (_, overlay) in &self.overlays {
            overlay.paint(&self.tree, 0.0, 0.0, &mut out);
        }
        
        // Render Tooltip
        let mut hit = None;
        if let Some((hx, hy)) = self.hover_pos {
            for cmd in out.iter().rev() {
                if let crate::DrawCommand::TooltipRegion { rect, text } = cmd {
                    if hx >= rect.x && hx <= rect.x + rect.width && hy >= rect.y && hy <= rect.y + rect.height {
                        hit = Some((text.clone(), *rect));
                        break;
                    }
                }
            }
        }
        
        if let Some((hit_text, hit_rect)) = hit {
            if Some(&hit_text) != self.hovered_tooltip.as_ref() {
                self.hovered_tooltip = Some(hit_text.clone());
                self.hover_time = 0.0;
            } else {
                self.hover_time += dt;
            }
            
            if self.hover_time > 0.4 {
                // Draw tooltip directly
                let padding = 8.0;
                let text_size = 14.0;
                let char_width = text_size * 0.6; // Approximation
                let tt_width = hit_text.len() as f32 * char_width + padding * 2.0;
                let tt_height = text_size + padding * 2.0;
                
                // Position above cursor, or below if it hits top of screen
                let (hx, hy) = self.hover_pos.unwrap();
                let mut tx = hx - tt_width / 2.0;
                let mut ty = hy + 24.0;
                if ty + tt_height > height {
                    ty = hy - tt_height - 10.0;
                }
                if tx < 10.0 { tx = 10.0; }
                if tx + tt_width > width - 10.0 { tx = width - tt_width - 10.0; }
                
                out.push(crate::DrawCommand::Rect {
                    rect: ferrite_layout::Rect { x: tx, y: ty, width: tt_width, height: tt_height },
                    color: crate::Color::rgb(0.2, 0.2, 0.22),
                    corner_radius: 6.0,
                });
                out.push(crate::DrawCommand::Text {
                    x: tx + padding,
                    y: ty + padding, // draw_text internally adds the baseline offset
                    content: hit_text,
                    size: text_size,
                    color: crate::Color::rgb(1.0, 1.0, 1.0),
                    max_width: None,
                    single_line: true,
                });
            } else {
                // Keep ticking until tooltip shows
                crate::request_repaint();
            }
        } else {
            self.hovered_tooltip = None;
            self.hover_time = 0.0;
        }
        
        out
    }

    pub fn click(&mut self, x: f32, y: f32) -> bool {
        let mut new_focus = None;
        let mut clicked = None;

        for (_, overlay) in self.overlays.iter_mut().rev() {
            if clicked.is_none() {
                clicked = overlay.click_at(&self.tree, 0.0, 0.0, x, y);
            }
            if new_focus.is_none() {
                new_focus = overlay.find_focusable_at(&self.tree, 0.0, 0.0, x, y);
            }
        }

        if clicked.is_none() {
            clicked = self.root.click_at(&self.tree, 0.0, 0.0, x, y);
        }
        if new_focus.is_none() {
            new_focus = self.root.find_focusable_at(&self.tree, 0.0, 0.0, x, y);
        }

        let mut press = None;
        for (_, overlay) in self.overlays.iter().rev() {
            if let Some(p) = Self::find_press_signal_at(overlay.as_ref(), &self.tree, 0.0, 0.0, x, y) {
                press = Some(p);
                break;
            }
        }
        if press.is_none() {
            press = Self::find_press_signal_at(self.root.as_ref(), &self.tree, 0.0, 0.0, x, y);
        }
        if let Some((id, sig)) = press {
            sig.set(true);
            self.pressed_node = Some(id);
        }

        if new_focus != self.focused {
            if let Some(old) = self.focused { self.root.dispatch_focus(old, false); }
            if let Some(new) = new_focus  {
                self.root.dispatch_focus(new, true);
                for (_, overlay) in self.overlays.iter_mut().rev() {
                    overlay.dispatch_focus(new, true);
                }
            }
            self.focused = new_focus;
        }
        self.active_drag = clicked;
        clicked.is_some()
    }

    pub fn double_click(&mut self, x: f32, y: f32) -> bool {
        let mut clicked = None;
        for (_, overlay) in self.overlays.iter_mut().rev() {
            if clicked.is_none() {
                clicked = overlay.double_click_at(&self.tree, 0.0, 0.0, x, y);
            }
        }
        if clicked.is_none() {
            clicked = self.root.double_click_at(&self.tree, 0.0, 0.0, x, y);
        }
        self.active_drag = None;
        clicked.is_some()
    }

    pub fn triple_click(&mut self, x: f32, y: f32) -> bool {
        let mut clicked = None;
        for (_, overlay) in self.overlays.iter_mut().rev() {
            if clicked.is_none() {
                clicked = overlay.triple_click_at(&self.tree, 0.0, 0.0, x, y);
            }
        }
        if clicked.is_none() {
            clicked = self.root.triple_click_at(&self.tree, 0.0, 0.0, x, y);
        }
        self.active_drag = None;
        clicked.is_some()
    }

    pub fn drag(&mut self, x: f32, y: f32) -> bool {
        if let Some(target) = self.active_drag {
            let mut dispatched = false;
            for (_, overlay) in self.overlays.iter_mut().rev() {
                if overlay.dispatch_drag(target, &self.tree, 0.0, 0.0, x, y) { dispatched = true; break; }
            }
            if !dispatched {
                self.root.dispatch_drag(target, &self.tree, 0.0, 0.0, x, y)
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn release_drag(&mut self) {
        self.active_drag = None;
        
        if let Some(old) = self.pressed_node.take() {
            if let Some((_, sig)) = Self::find_signal_by_node(self.root.as_ref(), &self.tree, old, false).or_else(|| {
                self.overlays.iter().find_map(|(_, o)| Self::find_signal_by_node(o.as_ref(), &self.tree, old, false))
            }) {
                sig.set(false);
            }
        }
    }

    pub fn scroll(&mut self, px: f32, py: f32, dx: f32, dy: f32) -> bool {
        for (_, overlay) in self.overlays.iter_mut().rev() {
            if overlay.scroll_at(&self.tree, 0.0, 0.0, px, py, dx, dy) { return true; }
        }
        self.root.scroll_at(&self.tree, 0.0, 0.0, px, py, dx, dy)
    }

    pub fn key_event(&mut self, event: KeyEvent) -> bool {
        if let Some(focused) = self.focused {
            let mut dispatched = false;
            for (_, overlay) in self.overlays.iter_mut().rev() {
                if overlay.dispatch_key(focused, &event) { dispatched = true; break; }
            }
            if !dispatched {
                self.root.dispatch_key(focused, &event)
            } else {
                true
            }
        } else {
            false
        }
    }

    pub fn blur(&mut self) {
        if let Some(old) = self.focused.take() {
            self.root.dispatch_focus(old, false);
            for (_, overlay) in self.overlays.iter_mut().rev() {
                overlay.dispatch_focus(old, false);
            }
        }
    }

    pub fn collect_focusable(&self) -> Vec<NodeId> {
        fn walk(w: &dyn crate::widget::Widget, out: &mut Vec<NodeId>) {
            if w.is_focusable() { out.push(w.node_id()); }
            for child in w.children() { walk(child.as_ref(), out); }
        }
        let mut out = Vec::new();
        walk(self.root.as_ref(), &mut out);
        for (_, overlay) in &self.overlays {
            walk(overlay.as_ref(), &mut out);
        }
        out
    }

    pub fn focus_next(&mut self) {
        let nodes = self.collect_focusable();
        if nodes.is_empty() { return; }
        let next = match self.focused {
            None => nodes[0],
            Some(cur) => {
                let idx = nodes.iter().position(|&n| n == cur).unwrap_or(0);
                nodes[(idx + 1) % nodes.len()]
            }
        };
        if let Some(old) = self.focused { self.root.dispatch_focus(old, false); }
        self.root.dispatch_focus(next, true);
        self.focused = Some(next);
        crate::dirty::request_repaint();
    }

    pub fn focus_prev(&mut self) {
        let nodes = self.collect_focusable();
        if nodes.is_empty() { return; }
        let next = match self.focused {
            None => nodes[nodes.len() - 1],
            Some(cur) => {
                let idx = nodes.iter().position(|&n| n == cur).unwrap_or(0);
                nodes[(idx + nodes.len() - 1) % nodes.len()]
            }
        };
        if let Some(old) = self.focused { self.root.dispatch_focus(old, false); }
        self.root.dispatch_focus(next, true);
        self.focused = Some(next);
        crate::dirty::request_repaint();
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{view::*, widget::Widget};
    use ferrite_reactive::*;

    #[test]
    fn test_modal_close() {
        let show_modal = create_signal(false);
        
        let app_view = col([
            modal(show_modal.clone(), move || show_modal.set(false), move || {
                let show_modal = show_modal.clone();
                col([
                    button("Save Changes", move || {
                        crate::toast::toast("Changes saved successfully!");
                        show_modal.set(false);
                    })
                ])
            }),
            crate::toast::toaster(),
        ]);

        let mut tree = ferrite_layout::LayoutTree::new();
        let root = app_view.build(&mut tree);
        let mut app = App::new(tree, root);
        
        app.render(800.0, 600.0);
        
        show_modal.set(true);
        app.render(800.0, 600.0);
        assert_eq!(app.overlays.len(), 1);
        
        // Find the Save Changes button and click it
        let mut clicked = false;
        let overlay_node = app.overlays[0].1.node_id();
        let overlay_layout = app.tree.layout(overlay_node);
        
        // We can just click everywhere in the overlay until we find it
        let mut click_count = 0;
        for x in 0..800 {
            for y in 0..600 {
                // To only click "Save Changes", we will simulate a click. If it returns true, we count it.
                // But we don't want to actually click Cancel. 
                // In our test, there's ONLY a "Save Changes" button! Let me check the test definition!
                if app.click(x as f32, y as f32) {
                    clicked = true;
                    break;
                }
            }
            if clicked { break; }
        }
        assert!(clicked, "Could not click Save Changes or Cancel");
        
        app.render(800.0, 600.0);
        assert_eq!(app.overlays.len(), 0); // This is what we want to verify!
    }
    #[test]
    fn test_toast_layout() {
        use crate::toast::*;
        use crate::view::*;
        let mut tree = ferrite_layout::LayoutTree::new();
        let view = col([
            button("Open", || {}),
            modal(ferrite_reactive::create_signal(true), || {}, || col([button("Save", || {})])),
            toaster()
        ]).fill().align(ferrite_layout::AlignItems::Center);
        let widget = view.build(&mut tree);
        let mut app = App::new(tree, widget);

        app.render(800.0, 600.0);
        
        toast("Test Message");
        
        app.render(800.0, 600.0);
        
        let root_layout = app.tree.layout(app.root.node_id());
        assert_eq!(root_layout.width, 800.0);
    }
}

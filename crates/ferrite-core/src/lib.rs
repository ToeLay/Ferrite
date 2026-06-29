mod app;
pub mod clipboard;
mod color;
pub mod context;
mod dirty;
mod draw;
pub mod event;
pub mod overlay;
pub mod toast;
pub mod theme;
pub mod view;
mod widget;
pub mod widgets; // This needs to be public because Widget implementors might need access to it, but its constructors are now pub(crate)
mod image;

pub use app::App;
pub use color::Color;
pub use dirty::{request_repaint, take_dirty};
pub use draw::DrawCommand;
pub use event::{KeyCode, KeyEvent, Modifiers};
pub use crate::image::{ImageData, ObjectFit};
pub use overlay::{show_overlay, remove_overlay};
pub use view::{AnyView, View, ViewIteratorExt};
pub use widget::Widget;
pub use theme::Theme;


pub use ferrite_layout::{AlignItems, Direction, Edges, JustifyContent, NodeId, Rect, Size, Style, PositionType, Inset};
pub use ferrite_layout::LayoutTree;
pub use ferrite_reactive as reactive;

pub use view::{text, label, button, input, textarea, col, row, spacer, divider, checkbox, slider, switch, scroll, list, portal, modal, dropdown, Anchor, image};
pub use context::{provide, inject, reset_context};
pub use toast::{toast, toaster, ToastData};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn click_updates_reactive_text_and_marks_dirty() {
        let count = reactive::create_signal(0i32);
        let root_view = col([
            label(move || format!("Count: {}", count.get())),
            button("+", move || count.update(|c| *c += 1)),
        ])
        .width(200.0)
        .height(120.0);

        let mut tree = LayoutTree::new();
        let root_widget = root_view.build(&mut tree);
        let mut app = App::new(tree, root_widget);

        take_dirty(); // clear any previous dirty flag from setup
        
        let cmds = app.render(200.0, 120.0);
        let init = cmds.iter().find_map(|c| match c {
            DrawCommand::Text { content, .. } if content.starts_with("Count") => Some(content.clone()),
            _ => None,
        });
        assert_eq!(init.as_deref(), Some("Count: 0"));

        // Simulate click
        // To find the button, we can inspect layout. It's the second child of the root.
        let root_children = app.root().children();
        assert_eq!(root_children.len(), 2);
        let btn_id = root_children[1].node_id();
        let btn_rect = app.absolute_rect(btn_id).unwrap();
        
        assert!(app.click(btn_rect.x + 10.0, btn_rect.y + 10.0));
        assert!(take_dirty());

        let cmds2 = app.render(200.0, 120.0);
        let updated = cmds2.iter().find_map(|c| match c {
            DrawCommand::Text { content, .. } if content.starts_with("Count") => Some(content.clone()),
            _ => None,
        });
        assert_eq!(updated.as_deref(), Some("Count: 1"));
    }

    #[test]
    fn text_input_updates_signal_on_key() {
        let value = reactive::create_signal(String::new());
        let root_view = col([
            input(value, "placeholder")
        ])
        .width(300.0)
        .height(60.0);

        let mut tree = LayoutTree::new();
        let root_widget = root_view.build(&mut tree);
        let mut app = App::new(tree, root_widget);

        app.render(300.0, 60.0);

        let input_id = app.root().children()[0].node_id();
        let input_rect = app.absolute_rect(input_id).unwrap();

        app.click(input_rect.x + 10.0, input_rect.y + 10.0); // focus
        app.key_event(KeyEvent { key: KeyCode::Char('h'), modifiers: Modifiers::default() });
        app.key_event(KeyEvent { key: KeyCode::Char('i'), modifiers: Modifiers::default() });
        assert_eq!(value.get(), "hi");
        app.key_event(KeyEvent { key: KeyCode::Backspace, modifiers: Modifiers::default() });
        assert_eq!(value.get(), "h");
    }
}

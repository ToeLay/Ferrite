mod app;
mod color;
mod dirty;
mod draw;
pub mod event;
mod widget;
pub mod widgets;

pub use app::App;
pub use color::Color;
pub use dirty::{request_repaint, take_dirty};
pub use draw::DrawCommand;
pub use event::{KeyCode, KeyEvent, Modifiers};
pub use widget::Widget;

pub use ferrite_layout::{AlignItems, Direction, Edges, JustifyContent, NodeId, Rect, Size, Style};
pub use ferrite_layout::LayoutTree;
pub use ferrite_reactive as reactive;

#[cfg(test)]
mod tests {
    use super::*;
    use widgets::{button, column, text_dyn, text_input};

    #[test]
    fn click_updates_reactive_text_and_marks_dirty() {
        let mut tree = LayoutTree::new();
        let count = reactive::create_signal(0i32);
        let label = text_dyn(&mut tree, move || format!("Count: {}", count.get()));
        let incr = button(&mut tree, "+", move || count.update(|c| *c += 1));
        let root = column(&mut tree,
            Style { width: Size::Px(200.0), height: Size::Px(120.0), ..Default::default() },
            vec![Box::new(label), Box::new(incr)]);
        let mut app = App::new(tree, Box::new(root));
        take_dirty();
        let cmds = app.render(200.0, 120.0);
        let init = cmds.iter().find_map(|c| match c {
            DrawCommand::Text { content, .. } if content.starts_with("Count") => Some(content.clone()),
            _ => None,
        });
        assert_eq!(init.as_deref(), Some("Count: 0"));
        assert!(app.click(20.0, 50.0));
        assert!(take_dirty());
        let cmds = app.render(200.0, 120.0);
        let updated = cmds.iter().find_map(|c| match c {
            DrawCommand::Text { content, .. } if content.starts_with("Count") => Some(content.clone()),
            _ => None,
        });
        assert_eq!(updated.as_deref(), Some("Count: 1"));
    }

    #[test]
    fn text_input_updates_signal_on_key() {
        let mut tree = LayoutTree::new();
        let value = reactive::create_signal(String::new());
        let input = text_input(&mut tree, value, "placeholder");
        let root = column(&mut tree,
            Style { width: Size::Px(300.0), height: Size::Px(60.0), ..Default::default() },
            vec![Box::new(input)]);
        let mut app = App::new(tree, Box::new(root));
        app.render(300.0, 60.0);
        app.click(150.0, 30.0); // focus
        app.key_event(KeyEvent { key: KeyCode::Char('h'), modifiers: Modifiers::default() });
        app.key_event(KeyEvent { key: KeyCode::Char('i'), modifiers: Modifiers::default() });
        assert_eq!(value.get(), "hi");
        app.key_event(KeyEvent { key: KeyCode::Backspace, modifiers: Modifiers::default() });
        assert_eq!(value.get(), "h");
    }
}

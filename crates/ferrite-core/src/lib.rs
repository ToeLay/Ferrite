//! The widget tree, layout integration, and draw command generation for
//! Ferrite. This crate knows nothing about windows, GPUs, or pixels — see
//! `ferrite-window` and `ferrite-render-skia` for that. What lives here is
//! the part that's true regardless of backend: what a widget *is*, how it
//! turns into boxes via `ferrite-layout`, and how it turns into an abstract
//! `DrawCommand` list that some backend then rasterizes.

mod app;
mod color;
mod dirty;
mod draw;
mod widget;
pub mod widgets;

pub use app::App;
pub use color::Color;
pub use dirty::{request_repaint, take_dirty};
pub use draw::DrawCommand;
pub use widget::Widget;

/// Layout types re-exported so a widget author doesn't need a direct
/// `ferrite-layout` dependency for the common case.
pub use ferrite_layout::{AlignItems, Direction, Edges, JustifyContent, NodeId, Rect, Size, Style};
pub use ferrite_layout::LayoutTree;

/// Reactivity re-exported for the same reason — building a `text_dyn` or a
/// button's `on_click` almost always means reaching for a signal.
pub use ferrite_reactive as reactive;

#[cfg(test)]
mod tests {
    use super::*;
    use widgets::{button, column, text_dyn};

    #[test]
    fn click_updates_reactive_text_and_marks_dirty() {
        let mut tree = LayoutTree::new();
        let count = reactive::create_signal(0i32);

        let label = text_dyn(&mut tree, move || format!("Count: {}", count.get()));
        let incr = button(&mut tree, "+", move || count.update(|c| *c += 1));

        let root = column(
            &mut tree,
            Style { width: Size::Px(200.0), height: Size::Px(120.0), ..Default::default() },
            vec![Box::new(label), Box::new(incr)],
        );

        let mut app = App::new(tree, Box::new(root));
        take_dirty(); // clear whatever the initial construction set

        let commands = app.render(200.0, 120.0);
        let initial_text = commands.iter().find_map(|c| match c {
            DrawCommand::Text { content, .. } if content.starts_with("Count") => Some(content.clone()),
            _ => None,
        });
        assert_eq!(initial_text.as_deref(), Some("Count: 0"));

        // The button is the second child, stacked below the text node with no
        // gap, so a click well inside its vertical range hits it.
        let handled = app.click(20.0, 50.0);
        assert!(handled, "click should have landed on the button");
        assert!(take_dirty(), "the effect re-running should have requested a repaint");

        let commands = app.render(200.0, 120.0);
        let updated_text = commands.iter().find_map(|c| match c {
            DrawCommand::Text { content, .. } if content.starts_with("Count") => Some(content.clone()),
            _ => None,
        });
        assert_eq!(updated_text.as_deref(), Some("Count: 1"));
    }
}

// Low-level re-exports (existing API, unchanged)
pub use ferrite_core::widgets;
pub use ferrite_core::{
    reactive, AlignItems, App, Color, Direction, DrawCommand, Edges, JustifyContent,
    KeyCode, KeyEvent, Modifiers, LayoutTree, NodeId, Rect, Size, Style, Widget,
};
pub use ferrite_reactive::{create_effect, create_memo, create_signal, Memo, Scope, Signal};

// New declarative API
pub use ferrite_core::{AnyView, View};
pub use ferrite_core::{text, label, button, input, col, row, spacer, divider, checkbox, slider, switch};
pub use ferrite_core::{provide, inject, reset_context};

// Alias create_signal to use_state for component-local state idiom
pub use ferrite_reactive::create_signal as use_state;

pub mod render { pub use ferrite_render_skia::render_to_pixmap; }
pub mod window  { pub use ferrite_window::{run as run_window, WindowConfig}; }

pub fn run(title: &str, size: (u32, u32), root: impl View) {
    let root_view = root.view();
    let mut tree = LayoutTree::new();
    let widget = root_view.build(&mut tree);
    let app = App::new(tree, widget);
    window::run_window(
        ferrite_window::WindowConfig {
            title: title.to_string(),
            width: size.0,
            height: size.1,
            background: Color::rgb(0.96, 0.97, 0.99),
        },
        app,
    );
}

pub fn run_with(config: ferrite_window::WindowConfig, root: impl View) {
    let root_view = root.view();
    let mut tree = LayoutTree::new();
    let widget = root_view.build(&mut tree);
    let app = App::new(tree, widget);
    window::run_window(config, app);
}

pub mod prelude {
    // Declarative API (primary)
    pub use crate::{
        text, label, button, input, col, row, spacer, divider, checkbox, slider, switch,
        AnyView, View,
        provide, inject, reset_context, use_state,
        run, run_with,
    };
    // Reactive primitives
    pub use crate::{
        create_effect, create_memo, create_signal,
        reactive, Memo, Scope, Signal,
    };
    // Layout & style
    pub use crate::{AlignItems, Color, Direction, Edges, JustifyContent, Size};
    // Low-level (still useful for power users)
    pub use crate::{App, LayoutTree, Style, Widget};
    pub use crate::window::WindowConfig;
}

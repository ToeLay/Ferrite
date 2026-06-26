// Low-level re-exports (existing API, unchanged)
pub use ferrite_core::widgets;
pub use ferrite_core::{
    reactive, AlignItems, App, Color, Direction, DrawCommand, Edges, JustifyContent,
    KeyCode, KeyEvent, Modifiers, LayoutTree, NodeId, Rect, Size, Style, Widget,
    PositionType, Inset
};
pub use ferrite_reactive::{create_effect, create_memo, create_signal, Memo, Scope, Signal, SignalVecExt};
pub use ferrite_reactive::{use_spring, use_tween, SpringConfig};

// New declarative API
pub use ferrite_core::{AnyView, View};
pub use ferrite_core::{text, label, button, input, textarea, col, row, spacer, divider, checkbox, slider, switch, scroll, list, portal, modal, dropdown, Anchor};
pub use ferrite_core::{provide, inject, reset_context, Theme};
pub use ferrite_core::{toast, toaster};

// Alias create_signal to use_state for component-local state idiom
pub use ferrite_reactive::create_signal as use_state;

pub mod render { pub use ferrite_render_skia::render_to_pixmap; }
pub mod window  { pub use ferrite_window::{run as run_window, WindowConfig}; }

pub fn run(title: &str, size: (u32, u32), root: impl View) {
    let root_view = portal(create_signal(true), move || {
        // We use a portal that renders the toaster on top of everything.
        // Wait, portal renders it as an overlay. So toaster() will be the top-most overlay.
        toaster()
    });
    
    // Actually, just put the toaster in a Z-stack with the root if we had one.
    // Or just run root and add toaster as a global overlay.
    // Yes! `portal` does exactly what we want!
    let root_view = col([
        root.view().flex_grow(1.0),
        root_view,
    ]).fill();
    let mut tree = LayoutTree::new();
    tree.set_text_measure(ferrite_render_skia::text_measure_fn());
    tree.set_wrap_lines(ferrite_render_skia::text_wrap_lines_fn());
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
    let root_view = portal(create_signal(true), move || toaster());
    let root_view = col([
        root.view().flex_grow(1.0),
        root_view,
    ]).fill();
    let mut tree = LayoutTree::new();
    tree.set_text_measure(ferrite_render_skia::text_measure_fn());
    tree.set_wrap_lines(ferrite_render_skia::text_wrap_lines_fn());
    let widget = root_view.build(&mut tree);
    let app = App::new(tree, widget);
    window::run_window(config, app);
}

pub mod prelude {
    // Declarative API (primary)
    pub use crate::{
        text, label, button, input, textarea, col, row, spacer, divider, checkbox, slider, switch, scroll, list, portal, modal, dropdown, Anchor,
        AnyView, View, Theme,
        provide, inject, reset_context, use_state,
        toast,
        run, run_with,
    };
    // Reactive primitives
    pub use crate::{
        create_effect, create_memo, create_signal,
        reactive, Memo, Scope, Signal, SignalVecExt,
    };
    // Layout & style
    pub use crate::{AlignItems, Color, Direction, Edges, JustifyContent, Size, PositionType, Inset};
    // Low-level (still useful for power users)
    pub use crate::{App, LayoutTree, Style, Widget};
    pub use crate::window::WindowConfig;
}

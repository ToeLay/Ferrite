pub use ferrite_core::widgets::{self, button, column, row, text, text_dyn, text_input};
pub use ferrite_core::{
    reactive, AlignItems, App, Color, Direction, DrawCommand, Edges, JustifyContent,
    KeyCode, KeyEvent, Modifiers, LayoutTree, NodeId, Rect, Size, Style, Widget,
};
pub use ferrite_reactive::{create_effect, create_memo, create_signal, Memo, Scope, Signal};

pub mod render { pub use ferrite_render_skia::render_to_pixmap; }
pub mod window  { pub use ferrite_window::{run, WindowConfig}; }

pub mod prelude {
    pub use crate::widgets::{button, column, row, text, text_dyn, text_input};
    pub use crate::{
        create_effect, create_memo, create_signal,
        reactive, AlignItems, App, Color, Direction, Edges, JustifyContent,
        LayoutTree, Memo, Scope, Signal, Size, Style, Widget,
    };
    pub use crate::window::{run as run_window, WindowConfig};
}

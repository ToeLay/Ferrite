//! Ferrite: a reactive GUI framework for Rust.
//!
//! ```no_run
//! use ferrite::prelude::*;
//!
//! fn main() {
//!     let mut tree = LayoutTree::new();
//!     let count = create_signal(0i32);
//!
//!     let label = text_dyn(&mut tree, move || format!("Count: {}", count.get()));
//!     let incr = button(&mut tree, "+", move || count.update(|c| *c += 1));
//!
//!     let root = column(
//!         &mut tree,
//!         Style { width: Size::Px(300.0), height: Size::Px(200.0), ..Default::default() },
//!         vec![Box::new(label), Box::new(incr)],
//!     );
//!
//!     let app = App::new(tree, Box::new(root));
//!     ferrite::window::run(WindowConfig::default(), app);
//! }
//! ```
//!
//! See `ARCHITECTURE.md` in the repo root for *why* it's shaped this way.

pub use ferrite_core::widgets::{self, button, column, row, text, text_dyn};
pub use ferrite_core::{
    reactive, AlignItems, App, Color, Direction, DrawCommand, Edges, JustifyContent, LayoutTree, NodeId, Rect, Size,
    Style, Widget,
};
pub use ferrite_reactive::{create_effect, create_memo, create_signal, Memo, Scope, Signal};

pub mod render {
    pub use ferrite_render_skia::render_to_pixmap;
}

pub mod window {
    pub use ferrite_window::{run, WindowConfig};
}

/// Everything you need for a typical app in one `use ferrite::prelude::*;`.
pub mod prelude {
    pub use crate::widgets::{button, column, row, text, text_dyn};
    pub use crate::{
        create_effect, create_memo, create_signal, reactive, AlignItems, App, Color, Direction, Edges,
        JustifyContent, LayoutTree, Memo, Scope, Signal, Size, Style, Widget,
    };
    pub use crate::window::{run as run_window, WindowConfig};
}

//! A reactive form: two text inputs whose values flow into a live preview
//! label -- no polling, no callbacks pushing strings around, just two signals
//! that any effect reading them re-runs on automatically.
//!
//! Run: cargo run -p text_input
//!
//! Click an input to focus it (border turns blue, cursor appears).
//! Press Tab to blur. Type freely -- the greeting updates live.

use ferrite::prelude::*;

fn main() {
    let mut tree = LayoutTree::new();

    // Two independent signals. Any reactive computation that reads them
    // re-runs automatically when the user types -- same mechanism as the
    // counter, just with strings instead of integers.
    let first = create_signal(String::new());
    let last  = create_signal(String::new());

    // ── Inputs ────────────────────────────────────────────────────────────
    let first_label = text(&mut tree, "First name").color(Color::rgb(0.35, 0.37, 0.42));
    let first_input = text_input(&mut tree, first, "Ada").width(&mut tree, 280.0);

    let last_label  = text(&mut tree, "Last name").color(Color::rgb(0.35, 0.37, 0.42));
    let last_input  = text_input(&mut tree, last, "Lovelace").width(&mut tree, 280.0);

    // ── Live greeting ─────────────────────────────────────────────────────
    let greeting = text_dyn(&mut tree, move || {
        let f = first.get();
        let l = last.get();
        match (f.is_empty(), l.is_empty()) {
            (true,  true)  => "Hello, stranger!".to_string(),
            (false, true)  => format!("Hello, {}!", f),
            (true,  false) => format!("Hello, {}!", l),
            _              => format!("Hello, {} {}!", f, l),
        }
    });
    let greeting = greeting.font_size(&mut tree, 22.0).color(Color::rgb(0.08, 0.09, 0.14));

    // ── Layout ────────────────────────────────────────────────────────────
    let first_group = column(&mut tree,
        Style { gap: 6.0, ..Default::default() },
        vec![Box::new(first_label), Box::new(first_input)]);

    let last_group = column(&mut tree,
        Style { gap: 6.0, ..Default::default() },
        vec![Box::new(last_label), Box::new(last_input)]);

    let root = column(&mut tree,
        Style {
            width: Size::Px(360.0),
            height: Size::Px(300.0),
            padding: Edges::all(32.0),
            gap: 20.0,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        vec![Box::new(first_group), Box::new(last_group), Box::new(greeting)]);

    let app = App::new(tree, Box::new(root));

    run_window(WindowConfig {
        title: "Ferrite — Text Input".to_string(),
        width: 360,
        height: 300,
        background: Color::rgb(0.96, 0.97, 0.99),
    }, app);
}

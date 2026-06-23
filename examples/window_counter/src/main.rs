//! The same counter as the `counter` example, but in a real OS window
//! instead of rendered to a PNG -- this is what an actual Ferrite app's
//! `main.rs` looks like. Run with: cargo run -p window_counter
//!
//! (This needs a display to run; it's included as the example proving the
//! native-window code path compiles and is exactly as small as it looks --
//! not as something a headless CI box can execute.)

use ferrite::prelude::*;

fn main() {
    let mut tree = LayoutTree::new();
    let count = create_signal(0i32);

    let title = text(&mut tree, "Ferrite Counter").color(Color::rgb(0.1, 0.1, 0.15));
    let title = title.font_size(&mut tree, 20.0);

    let label = text_dyn(&mut tree, move || format!("Count: {}", count.get()));
    let label = label.font_size(&mut tree, 28.0);

    let minus = button(&mut tree, "-", move || count.update(|c| *c -= 1));
    let plus = button(&mut tree, "+", move || count.update(|c| *c += 1));

    let controls = row(
        &mut tree,
        Style { gap: 16.0, justify_content: JustifyContent::Center, ..Default::default() },
        vec![Box::new(minus), Box::new(plus)],
    );

    let root = column(
        &mut tree,
        Style {
            width: Size::Px(360.0),
            height: Size::Px(240.0),
            padding: Edges::all(28.0),
            gap: 22.0,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        vec![Box::new(title), Box::new(label), Box::new(controls)],
    );

    let app = App::new(tree, Box::new(root));

    run_window(
        WindowConfig {
            title: "Ferrite Counter".to_string(),
            width: 360,
            height: 240,
            background: Color::rgb(0.96, 0.97, 0.99),
        },
        app,
    );
}

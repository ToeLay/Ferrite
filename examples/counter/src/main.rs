//! The canonical "hello world" of reactive UI, end to end through every
//! layer of Ferrite: a `Signal<i32>` drives a `text_dyn` label and is
//! mutated by two `Button`s, laid out with real flexbox via `ferrite-layout`,
//! and rasterized to a PNG via `ferrite-render-skia` -- with no window or
//! GPU involved, which is exactly the point of having a software backend.
//!
//! Run with: cargo run -p counter
//! Writes: counter_initial.png, counter_after_clicks.png

use ferrite_core::widgets::{button, column, row, text, text_dyn};
use ferrite_core::{reactive, AlignItems, App, Color, Edges, JustifyContent, LayoutTree, NodeId, Size, Style, Widget};

const WIDTH: f32 = 360.0;
const HEIGHT: f32 = 240.0;

struct ButtonIds {
    minus: NodeId,
    plus: NodeId,
}

fn build_app() -> (App, ButtonIds) {
    let mut tree = LayoutTree::new();
    let count = reactive::create_signal(0i32);

    let title = text(&mut tree, "Ferrite Counter Demo").color(Color::rgb(0.10, 0.11, 0.18));
    let title = title.font_size(&mut tree, 20.0);
    let label = text_dyn(&mut tree, move || format!("Count: {count}", count = count.get()))
        .color(Color::rgb(0.05, 0.05, 0.08));
    let label = label.font_size(&mut tree, 30.0);

    let minus = button(&mut tree, "-", move || count.update(|c| *c -= 1));
    let plus = button(&mut tree, "+", move || count.update(|c| *c += 1));
    let ids = ButtonIds { minus: minus.node_id(), plus: plus.node_id() };

    let controls = row(
        &mut tree,
        Style { gap: 18.0, justify_content: JustifyContent::Center, ..Default::default() },
        vec![Box::new(minus), Box::new(plus)],
    );

    let root = column(
        &mut tree,
        Style {
            width: Size::Px(WIDTH),
            height: Size::Px(HEIGHT),
            padding: Edges::all(28.0),
            gap: 22.0,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Center,
            ..Default::default()
        },
        vec![Box::new(title), Box::new(label), Box::new(controls)],
    );

    let app = App::new(tree, Box::new(root));
    (app, ids)
}

fn main() {
    let (mut app, buttons) = build_app();
    let bg = Color::rgb(0.96, 0.97, 0.99);

    // --- frame 1: initial state ---
    let commands = app.render(WIDTH, HEIGHT);
    let pixmap = ferrite_render_skia::render_to_pixmap(&commands, WIDTH as u32, HEIGHT as u32, bg);
    pixmap.save_png("counter_initial.png").expect("write png");
    println!("wrote counter_initial.png");

    // --- simulate real clicks on the actual rendered "+" button, three times ---
    let plus_rect = app.absolute_rect(buttons.plus).expect("plus button must be in the tree");
    let (cx, cy) = (plus_rect.x + plus_rect.width / 2.0, plus_rect.y + plus_rect.height / 2.0);
    for _ in 0..3 {
        let handled = app.click(cx, cy);
        assert!(handled, "click at the plus button's own center should always hit it");
    }
    // ...and one click on "-", to prove both directions work.
    let minus_rect = app.absolute_rect(buttons.minus).expect("minus button must be in the tree");
    let (mx, my) = (minus_rect.x + minus_rect.width / 2.0, minus_rect.y + minus_rect.height / 2.0);
    app.click(mx, my);

    assert!(ferrite_core::take_dirty(), "clicks should have triggered the reactive text effect");

    // --- frame 2: after 3x "+" and 1x "-" => count should read 2 ---
    let commands = app.render(WIDTH, HEIGHT);
    let pixmap = ferrite_render_skia::render_to_pixmap(&commands, WIDTH as u32, HEIGHT as u32, bg);
    pixmap.save_png("counter_after_clicks.png").expect("write png");
    println!("wrote counter_after_clicks.png (clicked + x3, - x1 => expect Count: 2)");
}

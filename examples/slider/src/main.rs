use ferrite::prelude::*;

fn main() {
    let r = use_state(0.5f32);
    let g = use_state(0.5f32);
    let b = use_state(0.5f32);

    let app = col([
        text("Color Mixer").size(24.0).padding_xy(0.0, 10.0),
        
        row([
            text("R").width(20.0),
            slider(r, 0.0, 1.0).width(200.0),
            label(move || format!("{:.2}", r.get())).width(50.0).padding_xy(10.0, 0.0),
        ]).align(AlignItems::Center),

        row([
            text("G").width(20.0),
            slider(g, 0.0, 1.0).width(200.0),
            label(move || format!("{:.2}", g.get())).width(50.0).padding_xy(10.0, 0.0),
        ]).align(AlignItems::Center),

        row([
            text("B").width(20.0),
            slider(b, 0.0, 1.0).width(200.0),
            label(move || format!("{:.2}", b.get())).width(50.0).padding_xy(10.0, 0.0),
        ]).align(AlignItems::Center),

        spacer(),
        
        // Color preview box
        col([
            label(move || format!("RGB({:.0}, {:.0}, {:.0})", r.get() * 255.0, g.get() * 255.0, b.get() * 255.0))
                .color(Color::BLACK)
        ])
        .background(Color::WHITE)
        .padding(10.0)
        .corner_radius(8.0)
    ])
    .padding(30.0)
    .gap(15.0)
    // The main window background gets dynamically updated to the mixed color via this wrapper logic.
    // However, window background isn't reactive in this framework yet (it is statically set in config).
    // So we'll just have the col itself be the background container filling the window.
    .background(Color::rgb(0.9, 0.9, 0.95))
    .flex_grow(1.0); // fill window

    run("Slider Test", (400, 300), app);
}

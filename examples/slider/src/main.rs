use ferrite::prelude::*;

fn main() {
    provide(Theme::dark());
    let theme = Theme::dark(); // Just for the main app background

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
    .background(theme.surface)
    .flex_grow(1.0); // fill window

    run("Slider Test", (400, 300), app);
}

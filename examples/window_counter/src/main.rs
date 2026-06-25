use ferrite::prelude::*;

fn main() {
    let count = use_state(0i32);

    ferrite::run(
        "Ferrite Counter",
        (360, 240),
        col([
            text("Ferrite Counter")
                .size(24.0)
                .color(Color::rgb(0.1, 0.1, 0.15)),
            label(move || format!("Count: {}", count.get()))
                .size(32.0),
            row([
                button("-", move || count.update(|c| *c -= 1)),
                button("+", move || count.update(|c| *c += 1)),
            ])
            .gap(16.0)
            .justify(JustifyContent::Center),
        ])
        .padding(32.0)
        .gap(24.0)
        .align(AlignItems::Center),
    );
}

use ferrite::prelude::*;

fn main() {
    let first = use_state(String::new());
    let last = use_state(String::new());

    ferrite::run(
        "Reactive Text Input",
        (400, 340),
        col([
            text("Profile Details")
                .size(24.0)
                .color(Color::rgb(0.1, 0.1, 0.15)),
            
            col([
                text("First name").color(Color::rgb(0.35, 0.37, 0.42)),
                input(first, "Ada").width(280.0),
            ]).gap(8.0),

            col([
                text("Last name").color(Color::rgb(0.35, 0.37, 0.42)),
                input(last, "Lovelace").width(280.0),
            ]).gap(8.0),

            spacer(),
            divider(),
            
            label(move || {
                let f = first.get();
                let l = last.get();
                if f.is_empty() && l.is_empty() {
                    "Hello, stranger.".to_string()
                } else {
                    format!("Hello, {} {}", f, l)
                }
            })
            .size(20.0)
            .color(Color::rgb(0.21, 0.43, 0.86)),
        ])
        .padding(32.0)
        .gap(20.0),
    );
}

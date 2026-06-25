use ferrite::prelude::*;

fn main() {
    provide(Theme::dark());

    let mut items = Vec::new();
    for i in 1..=50 {
        let row_bg = if i % 2 == 0 { Color::rgb(0.18, 0.18, 0.18) } else { Color::rgb(0.14, 0.14, 0.14) };
        items.push(
            row([
                text(&format!("Item #{}", i)).size(18.0),
                spacer(),
                button("Click Me", move || println!("Clicked item {}", i)),
            ])
            .padding(15.0)
            .align(AlignItems::Center)
            .background(row_bg)
        );
        if i < 50 {
            items.push(divider());
        }
    }

    let list = col(items).background(Color::rgb(0.15, 0.15, 0.15));
    
    let app = col([
        text("Scrollable List").size(24.0).padding(20.0),
        divider(),
        scroll(list),
    ])
    .background(Color::rgb(0.1, 0.1, 0.1))
    .flex_grow(1.0); // fill window

    run("Scroll Test", (400, 500), app);
}

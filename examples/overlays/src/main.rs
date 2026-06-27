use ferrite::prelude::*;

fn main() {
    ferrite::run("Overlay Showcase", (600, 500), app());
}

fn app() -> impl View {
    // State for Modal
    let show_modal = use_state(false);

    col([
        // Title
        text("Overlay & Toast Showcase")
            .size(24.0)
            .margin(20.0),

        // 1. Tooltips
        row([
            button("Hover me for Tooltip", || {})
                .tooltip("This is a simple tooltip!")
                .margin(10.0),
            
            button("Hover me too", || {})
                .tooltip("Another tooltip over here.")
        ]).margin(20.0),

        // 2. Modals
        row([
            button("Open Modal", move || {
                println!("Open Modal clicked!");
                show_modal.set(true);
            }),
        ]).margin(20.0),

        // 3. Dropdowns
        row([
            dropdown(
                "Options Menu",
                200.0, // width
                vec![
                    "Profile Settings".to_string(),
                    "Appearance".to_string(),
                    "Logout".to_string(),
                    "A very long menu item that should be truncated automatically".to_string(),
                ],
                move |index, item| {
                    ferrite::toast(&format!("Selected [{}]: {}", index, item));
                }
            )
        ]).margin(20.0),

        // 4. Toasts
        row([
            button("Show Toast", || {
                ferrite::toast("This is a global toast message!");
            }),
            button("Show Another Toast", || {
                ferrite::toast("You clicked another toast button!");
            }).margin(10.0),
        ]).margin(20.0),

        // 5. Iterator View Composition
        text("Static List Built with Iterators:").margin(10.0),
        ["Option A", "Option B", "Option C"].iter()
            .map(|name| text(name).size(16.0).padding(10.0))
            .intersperse_with(|| divider())
            .collect_col()
            .width(200.0)
            .margin(10.0)
            .background(Color::rgb(0.9, 0.9, 0.9))
            .corner_radius(8.0),

        // Overlay Components defined declaratively:
        
        // Modal Declaration
        modal(
            show_modal, 
            move || show_modal.set(false), // on_close
            move || {
                col([
                    row([
                        text("Settings Modal").size(22.0).margin(10.0),
                        spacer(),
                    ]).padding(10.0),
                    
                    text("This is an example of a cleanly styled modal window. It floats above the rest of the application.")
                        .size(16.0)
                        .margin(10.0),
                        
                    spacer(),
                    
                    row([
                        spacer(),
                        button("Cancel", move || show_modal.set(false)).padding(10.0).margin(10.0),
                        button("Save Changes", move || {
                            ferrite::toast("Changes saved successfully!");
                            show_modal.set(false);
                        }).padding(10.0).margin(10.0),
                    ]),
                ])
                .padding(20.0)
                .background(Color::WHITE)
                .corner_radius(12.0)
                .width(400.0)
                .height(300.0)
            }
        )
    ])
    .fill()
    .align(AlignItems::Center)
}

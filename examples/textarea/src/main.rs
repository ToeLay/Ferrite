use ferrite::prelude::*;

fn app() -> impl View {
    let text_val = use_state("Hello world!\nThis is a multiline text area.\nYou can type here, use enter for new lines, and it scrolls vertically.".to_string());
    let show_modal = use_state(false);

    let modal_content = {
        let show_modal = show_modal.clone();
        move || {
            col([
                text("This is an overlay!").size(24.0),
                spacer().height(20.0),
                button("Close Modal", move || show_modal.set(false))
            ])
            .padding(40.0)
            .background(Color::rgba(1.0, 1.0, 1.0, 1.0))
            .corner_radius(10.0)
            .align(ferrite::prelude::AlignItems::Center)
            .justify(ferrite::prelude::JustifyContent::Center)
            .position_type(ferrite::prelude::PositionType::Absolute)
            // Center the absolute element using top and left sizes... wait, auto doesn't center in flexbox automatically for absolute.
            // But if it's the root of its own Taffy tree (which it is, because overlays are separate trees computed with width/height), 
            // then its size will be just the content size, positioned at 0,0 unless we center it!
            // Actually, if we make the modal container full screen:
            // .width_percent(100.0).height_percent(100.0)
            // then it will fill the screen, and align/justify center will center its children!
            // Oh wait, the overlay *is* the view we return here.
            // So we can wrap it in a full-screen container!
        }
    };

    let modal_overlay = {
        move || {
            col([
                modal_content()
            ])
            .width_percent(100.0)
            .height_percent(100.0)
            .background(Color::rgba(0.0, 0.0, 0.0, 0.6)) // Dimmed background
            .align(ferrite::prelude::AlignItems::Center)
            .justify(ferrite::prelude::JustifyContent::Center)
        }
    };

    col([
        text("TextArea Demo").size(24.0),
        spacer().height(20.0),
        button("Show Modal", move || show_modal.set(true)),
        spacer().height(20.0),
        textarea(text_val.clone()),
        spacer().height(20.0),
        label(move || format!("Length: {} characters", text_val.get().len())),
        
        // Include the portal in the tree!
        portal(show_modal, modal_overlay),
    ])
    .padding(40.0)
    .fill()
}

fn main() {
    ferrite::run("TextArea Demo", (600, 600), app());
}

use ferrite::prelude::*;

fn app() -> impl View {
    let text_val = use_state("Hello world!\nThis is a multiline text area.\nYou can type here, use enter for new lines, and it scrolls vertically.".to_string());

    col([
        text("TextArea Demo").size(24.0),
        spacer().height(20.0),
        textarea(text_val.clone()),
        spacer().height(20.0),
        label(move || format!("Length: {} characters", text_val.get().len())),
    ])
    .padding(40.0)
    .fill()
}

fn main() {
    ferrite::run("TextArea Demo", (600, 600), app());
}

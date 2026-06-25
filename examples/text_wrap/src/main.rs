use ferrite::{run, col, text};
use ferrite_core::Color;

fn main() {
    let long_text = "This is a very long string of text that should automatically wrap when it hits the edge of its container. \
    It demonstrates how the Taffy flexbox layout system accurately computes height based on the text width, ensuring that \
    subsequent elements are pushed down correctly without overlapping. Here is a very long word: \
    Supercalifragilisticexpialidocious! Let's see how it breaks.";

    let app = col([
        text("Text Wrapping Demo").size(24.0).padding(20.0),
        
        // A narrow column to force wrapping
        col([
            text(long_text).size(25.0).color(Color::rgb(0.8, 0.8, 0.8))
        ])
        .padding(20.0)
        .background(Color::rgb(0.2, 0.2, 0.2)),

        text("This text should be correctly positioned below the wrapped text!").padding(20.0),
    ])
    .background(Color::rgb(0.1, 0.1, 0.1));

    run("Text Wrapping Demo", (500, 500), app);
}

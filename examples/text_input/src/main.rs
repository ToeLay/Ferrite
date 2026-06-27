//! A clean single-page form: two inputs, a live greeting.
//! Shows card layout, field labels, placeholder text, reactive preview.

use ferrite::prelude::*;
use ferrite_core::view::{badge, button_secondary, card, divider, spacer};
use ferrite_core::Theme;

fn main() {
    provide(Theme::light());

    ferrite::run("Profile", (420, 380), col([
        // ── Header ───────────────────────────────────────────────────────
        row([
            text("Profile").size(22.0),
            spacer(),
            badge("New"),
        ])
        .align(AlignItems::Center),

        divider(),

        // ── Form card ────────────────────────────────────────────────────
        form_view(),
    ])
    .padding(32.0)
    .gap(20.0));
}

fn form_view() -> AnyView {
    let theme = inject::<Theme>();
    let first = use_state(String::new());
    let last  = use_state(String::new());

    card([
        col([
            // First name
            col([
                text("First name").size(12.0).color(theme.text_secondary),
                input(first, "Ada").width(330.0),
            ]).gap(6.0),

            // Last name
            col([
                text("Last name").size(12.0).color(theme.text_secondary),
                input(last, "Lovelace").width(330.0),
            ]).gap(6.0),

            divider(),

            // Reactive greeting
            label(move || {
                let f = first.get();
                let l = last.get();
                match (f.is_empty(), l.is_empty()) {
                    (true,  true)  => "Hello, stranger!".into(),
                    (false, true)  => format!("Hello, {}!", f),
                    (true,  false) => format!("Hello, {}!", l),
                    _              => format!("Hello, {} {}!", f, l),
                }
            })
            .size(20.0)
            .color(theme.primary),

            // Actions
            row([
                spacer(),
                button_secondary("Clear", move || {
                    first.set(String::new());
                    last.set(String::new());
                }),
                button("Save", || {}),
            ])
            .gap(8.0)
            .align(AlignItems::Center),
        ])
        .gap(14.0),
    ])
}

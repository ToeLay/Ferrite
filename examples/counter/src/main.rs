//! Ferrite showcase — Basic and Advanced tabs.
//! Demonstrates card layout, text hierarchy, badges, button variants,
//! reactive labels, sliders, checkboxes, and the theme system.

use ferrite::prelude::*;
use ferrite_core::view::{
    badge, badge_muted, badge_success,
    button_ghost, button_secondary,
    card, divider, key_value_dyn, section_header,
    spacer, switch,
};
use ferrite_core::Theme;

#[derive(Clone, PartialEq, Eq)]
enum Tab { Basic, Advanced }

fn main() {
    provide(Theme::light());
    let active_tab = use_state(Tab::Basic);

    ferrite::run("Ferrite", (500, 520), col([
        // ── Header ───────────────────────────────────────────────────────
        app_header(),

        divider(),

        // ── Tab bar ──────────────────────────────────────────────────────
        tab_bar(active_tab),

        divider(),

        // ── Tab content ──────────────────────────────────────────────────
        switch(active_tab, [
            (Tab::Basic,    basic_tab()),
            (Tab::Advanced, advanced_tab()),
        ]),
    ])
    .padding(28.0)
    .gap(16.0));
}

// ── App chrome ────────────────────────────────────────────────────────────────

fn app_header() -> AnyView {
    let theme = inject::<Theme>();
    row([
        col([
            text("Ferrite").size(22.0).color(theme.text),
            text("UI Framework").size(12.0).color(theme.muted),
        ]).gap(1.0),
        spacer(),
        badge("v0.1"),
    ])
    .align(AlignItems::Center)
}

fn tab_bar(active: Signal<Tab>) -> AnyView {
    let theme = inject::<Theme>();
    row([
        tab_button("Basic",    active, Tab::Basic),
        tab_button("Advanced", active, Tab::Advanced),
    ])
    .gap(6.0)
}

/// A tab button that shows its active/inactive state by swapping between
/// a filled (primary) and a ghost variant.
fn tab_button(title: &'static str, active: Signal<Tab>, this: Tab) -> AnyView {
    let this_active  = this.clone();
    let this_inactive = this.clone();

    col([
        // Primary (active) version
        button(title, {
            let a = active;
            let t = this.clone();
            move || a.set(t.clone())
        })
        .visible_when(move || active.get() == this_active),

        // Ghost (inactive) version
        button_ghost(title, {
            let a = active;
            let t = this.clone();
            move || a.set(t.clone())
        })
        .visible_when(move || active.get() != this_inactive),
    ])
}

// ── Basic tab ─────────────────────────────────────────────────────────────────

fn basic_tab() -> AnyView {
    let theme = inject::<Theme>();
    let count = use_state(0i32);
    let name  = use_state(String::new());

    col([
        // ── Counter card ──────────────────────────────────────────────────
        card([
            // Card header row: label + status badge
            row([
                text("Counter").size(12.0).color(theme.text_secondary),
                spacer(),
                // Badge changes green above 0, muted when at 0
                label(move || if count.get() > 0 { String::new() } else { String::new() })
                    .visible_when(move || count.get() > 0)
                    // (intentionally empty — the badge_success below shows instead)
                    .size(1.0),
                badge_success("Active")
                    .visible_when(move || count.get() > 0),
                badge_muted("Idle")
                    .visible_when(move || count.get() == 0),
            ])
            .align(AlignItems::Center),

            // Big number — the hero element of the card
            label(move || count.get().to_string())
                .size(64.0)
                .color(theme.text),

            // Controls row: reset on left, decrement/increment on right
            row([
                button_ghost("Reset", move || count.set(0)),
                spacer(),
                row([
                    button_secondary("−", move || count.update(|c| *c -= 1)),
                    button("+", move || count.update(|c| *c += 1)),
                ]).gap(8.0),
            ])
            .align(AlignItems::Center),
        ]),

        // ── Name card ─────────────────────────────────────────────────────
        card([
            col([
                text("Your name").size(12.0).color(theme.text_secondary),
                input(name, "Ada Lovelace").width(360.0),
                label(move || {
                    let n = name.get();
                    if n.is_empty() { String::new() }
                    else { format!("Hello, {}!", n) }
                })
                .size(20.0)
                .color(theme.primary)
                .visible_when(move || !name.get().is_empty()),
            ])
            .gap(10.0),
        ]),
    ])
    .gap(12.0)
}

// ── Advanced tab ──────────────────────────────────────────────────────────────

fn advanced_tab() -> AnyView {
    let theme = inject::<Theme>();
    let volume = use_state(72.0f32);
    let enabled = use_state(true);
    let dark_mode = use_state(false);

    col([
        // ── Audio card ────────────────────────────────────────────────────
        card([
            section_header("Audio", ""),

            col([
                row([
                    text("Enable audio").size(14.0).color(theme.text),
                    spacer(),
                    checkbox("", enabled),
                ]).align(AlignItems::Center),

                col([
                    row([
                        text("Master volume").size(14.0).color(theme.text),
                        spacer(),
                        label(move || format!("{:.0}%", volume.get()))
                            .size(14.0)
                            .color(theme.primary),
                    ]).align(AlignItems::Center),

                    slider(volume, 0.0, 100.0).width(380.0),
                ])
                .gap(8.0)
                .visible_when(move || enabled.get()),

                text("Audio is disabled.")
                    .size(13.0)
                    .color(theme.muted)
                    .visible_when(move || !enabled.get()),
            ])
            .gap(14.0),
        ]),

        // ── Appearance card ───────────────────────────────────────────────
        card([
            section_header("Appearance", ""),

            row([
                text("Dark mode").size(14.0).color(theme.text),
                spacer(),
                checkbox("", dark_mode),
            ]).align(AlignItems::Center),
        ]),

        // ── Info card: reactive stats ─────────────────────────────────────
        card([
            section_header("Current state", ""),
            key_value_dyn("Volume",    move || format!("{:.0}", volume.get())),
            key_value_dyn("Audio",     move || if enabled.get() { "On".into() } else { "Off".into() }),
            key_value_dyn("Dark mode", move || if dark_mode.get() { "On".into() } else { "Off".into() }),
        ]),
    ])
    .gap(12.0)
}

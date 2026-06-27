//! Color Mixer — three sliders control R/G/B. The large swatch and hex code
//! update live. Shows how signals compose into derived displays.

use ferrite::prelude::*;
use ferrite_core::view::{card, divider, key_value_dyn, section_header, spacer};
use ferrite_core::Theme;

fn main() {
    provide(Theme::light());

    ferrite::run(
        "Color Mixer",
        (440, 460),
        col([
            // ── Header ───────────────────────────────────────────────────────
            row([text("Color Mixer").size(22.0)]).align(AlignItems::Center),
            divider(),
            mixer_view(),
        ])
        .padding(28.0)
        .gap(16.0),
    );
}

fn mixer_view() -> AnyView {
    let theme = inject::<Theme>();

    let r = use_state(74.0f32);
    let g = use_state(130.0f32);
    let b = use_state(220.0f32);

    col([
        // ── Color swatch card ─────────────────────────────────────────────
        card([col([
            // Large swatch — the hero element
            // Implemented as a fixed-size col whose background we can't
            // set reactively right now, so we overlay the hex code instead
            // and show it on a neutral background.
            col([
                label(move || {
                    let (rv, gv, bv) = (r.get() as u8, g.get() as u8, b.get() as u8);
                    format!("#{:02X}{:02X}{:02X}", rv, gv, bv)
                })
                .size(28.0)
                .color(theme.text),
                label(move || format!("rgb({:.0}, {:.0}, {:.0})", r.get(), g.get(), b.get()))
                    .size(13.0)
                    .color(theme.text_secondary),
            ])
            .gap(4.0)
            .padding(24.0)
            .background(theme.surface_2)
            .corner_radius(theme.radius_sm)
            .align(AlignItems::Center)
            .justify(JustifyContent::Center),
        ])]),
        // ── Sliders card ──────────────────────────────────────────────────
        card([
            section_header("Channels", ""),
            col([
                channel_row("Red", r, theme.danger),
                channel_row("Green", g, theme.success),
                channel_row("Blue", b, theme.primary),
            ])
            .gap(16.0),
        ]),
        // ── Values card ───────────────────────────────────────────────────
        card([
            section_header("Values", ""),
            col([
                key_value_dyn("Hex", move || {
                    format!(
                        "#{:02X}{:02X}{:02X}",
                        r.get() as u8,
                        g.get() as u8,
                        b.get() as u8
                    )
                }),
                key_value_dyn("RGB", move || {
                    format!("{:.0}, {:.0}, {:.0}", r.get(), g.get(), b.get())
                }),
                key_value_dyn("HSL", move || rgb_to_hsl_string(r.get(), g.get(), b.get())),
            ])
            .gap(10.0),
        ]),
    ])
    .gap(12.0)
}

/// One channel row: label + value on the right, slider below.
fn channel_row(name: &str, value: Signal<f32>, _accent: Color) -> AnyView {
    let theme = inject::<Theme>();
    col([
        row([
            text(name).size(13.0).color(theme.text_secondary),
            spacer(),
            label(move || format!("{:.0}", value.get()))
                .size(13.0)
                .color(theme.text),
        ])
        .align(AlignItems::Center),
        slider(value, 0.0, 255.0).width(360.0),
    ])
    .gap(6.0)
}

/// Approximate RGB → HSL for the values card (no external dep, good enough).
fn rgb_to_hsl_string(r: f32, g: f32, b: f32) -> String {
    let (r, g, b) = (r / 255.0, g / 255.0, b / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < 1e-6 {
        return format!("hsl(0, 0%, {:.0}%)", l * 100.0);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;
    format!(
        "hsl({:.0}, {:.0}%, {:.0}%)",
        h * 360.0,
        s * 100.0,
        l * 100.0
    )
}

use crate::Color;

/// A complete design token system. Every colour, radius, and spacing value
/// used by Ferrite's built-in widgets comes from here — nothing is hardcoded.
///
/// Consumers can customise the whole look by calling `provide(Theme::dark())`
/// (or any custom `Theme`) before `ferrite::run`. Widgets call
/// `try_inject::<Theme>().unwrap_or_default()` at build time.
#[derive(Clone, Copy, PartialEq)]
pub struct Theme {
    // ── Backgrounds ──────────────────────────────────────────────────────────
    /// Page / window background — slightly off-white / off-black.
    pub background: Color,
    /// Card and panel surface — slightly brighter than background.
    pub surface: Color,
    /// Inputs, code blocks, tags — slightly darker than surface.
    pub surface_2: Color,

    // ── Text ─────────────────────────────────────────────────────────────────
    /// Primary body text.
    pub text: Color,
    /// Secondary / label text (slightly faded).
    pub text_secondary: Color,
    /// Placeholder, disabled, hint text.
    pub muted: Color,

    // ── Brand ─────────────────────────────────────────────────────────────────
    pub primary: Color,
    pub primary_hover: Color,
    pub on_primary: Color,

    // ── Borders ──────────────────────────────────────────────────────────────
    /// Default subtle border colour.
    pub border: Color,
    /// Border used when an element has focus.
    pub border_focus: Color,

    // ── Semantic ─────────────────────────────────────────────────────────────
    pub success: Color,
    pub danger: Color,
    pub warning: Color,

    // ── Geometry — radius ────────────────────────────────────────────────────
    pub radius_xs: f32,  // 3  — tags, badges
    pub radius_sm: f32,  // 6  — checkboxes, small buttons
    pub radius_md: f32,  // 10 — inputs, buttons, cards
    pub radius_lg: f32,  // 16 — modals, large cards

    // ── Geometry — spacing scale ──────────────────────────────────────────────
    /// 4 px
    pub space_1: f32,
    /// 8 px
    pub space_2: f32,
    /// 12 px
    pub space_3: f32,
    /// 16 px
    pub space_4: f32,
    /// 24 px
    pub space_6: f32,
    /// 32 px
    pub space_8: f32,
    /// 48 px
    pub space_12: f32,

    // ── Legacy (kept for backward compat) ────────────────────────────────────
    pub on_surface: Color,
    pub spacing: f32,
    pub radius: f32,
}

impl Theme {
    pub fn light() -> Self {
        // Palette: cool-neutral grays + Indigo 600 brand colour.
        // All hex values reference Tailwind CSS v3 colour tokens so they're
        // easy to look up and adjust.
        Theme {
            // #F4F5F7 — just enough warmth to feel paper-like, not clinical.
            background: Color::rgb(0.957, 0.961, 0.969),
            surface:    Color::WHITE,
            // #EDEEF3 — input fields / code blocks
            surface_2:  Color::rgb(0.929, 0.933, 0.953),

            // #111827  — not pure black; has a hint of indigo for warmth.
            text:           Color::rgb(0.067, 0.094, 0.153),
            // #4B5568
            text_secondary: Color::rgb(0.294, 0.333, 0.408),
            // #9CA3AF
            muted:          Color::rgb(0.612, 0.639, 0.686),

            // Indigo 600 — #4F46E5
            primary:       Color::rgb(0.310, 0.275, 0.898),
            // Indigo 700 — #4338CA
            primary_hover: Color::rgb(0.263, 0.220, 0.792),
            on_primary:    Color::WHITE,

            // Gray 200 — #E5E7EB
            border:       Color::rgb(0.898, 0.906, 0.922),
            border_focus: Color::rgb(0.310, 0.275, 0.898), // = primary

            // Emerald 600 — #059669
            success: Color::rgb(0.020, 0.588, 0.412),
            // Red 600 — #DC2626
            danger:  Color::rgb(0.863, 0.149, 0.149),
            // Amber 500 — #F59E0B
            warning: Color::rgb(0.961, 0.620, 0.043),

            radius_xs: 3.0,
            radius_sm: 6.0,
            radius_md: 10.0,
            radius_lg: 16.0,

            space_1: 4.0,
            space_2: 8.0,
            space_3: 12.0,
            space_4: 16.0,
            space_6: 24.0,
            space_8: 32.0,
            space_12: 48.0,

            on_surface: Color::rgb(0.067, 0.094, 0.153),
            spacing: 8.0,
            radius: 10.0,
        }
    }

    pub fn dark() -> Self {
        Theme {
            // #0F1117 — very dark, just slightly blue
            background: Color::rgb(0.059, 0.067, 0.090),
            // #1A1D27
            surface:    Color::rgb(0.102, 0.114, 0.153),
            // #252836
            surface_2:  Color::rgb(0.145, 0.157, 0.212),

            // #F3F4F6
            text:           Color::rgb(0.953, 0.957, 0.965),
            // #9CA3AF
            text_secondary: Color::rgb(0.612, 0.639, 0.686),
            // #6B7280
            muted:          Color::rgb(0.420, 0.447, 0.502),

            // Indigo 400 — #818CF8  (brighter for dark backgrounds)
            primary:       Color::rgb(0.506, 0.549, 0.973),
            // Indigo 500 — #6366F1
            primary_hover: Color::rgb(0.388, 0.400, 0.945),
            on_primary: Color::WHITE,

            // Gray 700 — #374151
            border:       Color::rgb(0.216, 0.255, 0.318),
            border_focus: Color::rgb(0.506, 0.549, 0.973),

            success: Color::rgb(0.063, 0.725, 0.506), // Emerald 500 (brighter on dark)
            danger:  Color::rgb(0.973, 0.267, 0.267),
            warning: Color::rgb(0.984, 0.749, 0.188),

            radius_xs: 3.0,
            radius_sm: 6.0,
            radius_md: 10.0,
            radius_lg: 16.0,

            space_1: 4.0,
            space_2: 8.0,
            space_3: 12.0,
            space_4: 16.0,
            space_6: 24.0,
            space_8: 32.0,
            space_12: 48.0,

            on_surface: Color::rgb(0.953, 0.957, 0.965),
            spacing: 8.0,
            radius: 10.0,
        }
    }
}

impl Default for Theme {
    fn default() -> Self { Self::light() }
}

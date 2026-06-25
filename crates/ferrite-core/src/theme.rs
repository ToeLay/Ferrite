use crate::Color;

#[derive(Clone)]
pub struct Theme {
    pub primary:    Color,
    pub on_primary: Color,
    pub surface:    Color,
    pub on_surface: Color,
    pub muted:      Color,
    pub radius_sm:  f32,
    pub radius_md:  f32,
    pub spacing:    f32,
}

impl Theme {
    pub fn light() -> Self {
        Theme {
            primary:    Color::rgb(0.21, 0.43, 0.86), // Blue
            on_primary: Color::WHITE,
            surface:    Color::WHITE,
            on_surface: Color::rgb(0.08, 0.08, 0.10), // Almost black
            muted:      Color::rgb(0.85, 0.87, 0.90), // Light gray
            radius_sm:  4.0,
            radius_md:  8.0,
            spacing:    10.0,
        }
    }

    pub fn dark() -> Self {
        Theme {
            primary:    Color::rgb(0.35, 0.55, 0.95), // Lighter blue for dark mode
            on_primary: Color::WHITE,
            surface:    Color::rgb(0.12, 0.12, 0.14), // Dark gray
            on_surface: Color::rgb(0.9, 0.9, 0.92),   // Off-white
            muted:      Color::rgb(0.25, 0.25, 0.28), // Darker gray
            radius_sm:  4.0,
            radius_md:  8.0,
            spacing:    10.0,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

//! Reference render backend for Ferrite: takes the `DrawCommand` list
//! `ferrite-core` produces and rasterizes it on the CPU with `tiny-skia`,
//! using `fontdue` for glyph rasterization against an embedded font
//! (IBM Plex Mono, SIL OFL 1.1 -- see `assets/IBMPlexMono-OFL.txt`).
//!
//! This exists for three reasons, not just as a placeholder:
//! 1. It's a real, dependency-light way to render Ferrite UIs headlessly --
//!    screenshot tests, server-side rendering of a settings panel, etc.
//! 2. It's a much smaller surface than a GPU backend to get *correct* first,
//!    so the windowing/event story (`ferrite-window`) can be built and tested
//!    against something trustworthy before a wgpu backend's correctness has
//!    to be untangled from its performance.
//! 3. `ferrite-core` only knows about `DrawCommand`s -- this crate is proof
//!    that constraint is real, not aspirational: nothing in here reaches
//!    back into widgets, layout, or signals.

use ferrite_core::{Color as FColor, DrawCommand, Rect as FRect};
use fontdue::{Font, FontSettings};
use std::sync::OnceLock;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Rect as SkRect, Transform};

static FONT_BYTES: &[u8] = include_bytes!("../assets/IBMPlexMono-Regular.ttf");

fn font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| {
        Font::from_bytes(FONT_BYTES, FontSettings::default()).expect("embedded font failed to parse")
    })
}

/// Rasterize a full draw command list into a new pixmap of the given size
/// (physical pixels), filled with `background` first.
pub fn render_to_pixmap(commands: &[DrawCommand], width: u32, height: u32, background: FColor) -> Pixmap {
    let mut pixmap = Pixmap::new(width.max(1), height.max(1)).expect("non-zero pixmap size");
    pixmap.fill(to_skia_color(background));
    for cmd in commands {
        match cmd {
            DrawCommand::Rect { rect, color, corner_radius } => draw_rect(&mut pixmap, *rect, *color, *corner_radius),
            DrawCommand::Text { x, y, content, size, color } => draw_text(&mut pixmap, *x, *y, content, *size, *color),
        }
    }
    pixmap
}

fn to_skia_color(c: FColor) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(c.r.clamp(0.0, 1.0), c.g.clamp(0.0, 1.0), c.b.clamp(0.0, 1.0), c.a.clamp(0.0, 1.0))
        .unwrap_or(tiny_skia::Color::BLACK)
}

fn draw_rect(pixmap: &mut Pixmap, rect: FRect, color: FColor, radius: f32) {
    let mut paint = Paint::default();
    paint.set_color(to_skia_color(color));
    paint.anti_alias = true;

    let path = if radius <= 0.0 {
        let Some(r) = SkRect::from_xywh(rect.x, rect.y, rect.width, rect.height) else { return };
        Some(PathBuilder::from_rect(r))
    } else {
        rounded_rect_path(rect, radius.min(rect.width / 2.0).min(rect.height / 2.0))
    };
    let Some(path) = path else { return };
    pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), None);
}

/// Builds a rounded-rect path by hand: tiny-skia's high-level API doesn't
/// ship a round-rect constructor, so each corner is a quadratic Bezier from
/// one straight edge to the next. (Splitting a circular arc into a single
/// quad isn't mathematically exact, but at the corner radii a UI actually
/// uses -- a handful of pixels -- the error is sub-pixel and invisible.)
fn rounded_rect_path(rect: FRect, r: f32) -> Option<tiny_skia::Path> {
    let (x, y, w, h) = (rect.x, rect.y, rect.width, rect.height);
    let mut pb = PathBuilder::new();
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
    pb.finish()
}

fn draw_text(pixmap: &mut Pixmap, x: f32, y: f32, content: &str, size: f32, color: FColor) {
    let f = font();
    let (r, g, b) = ((color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8);
    let mut pen_x = x;
    // `y` is the box's top from layout; nudge down by ~80% of size to
    // approximate baseline placement without doing real line-metrics math.
    let baseline_y = y + size * 0.8;

    for ch in content.chars() {
        let (metrics, bitmap) = f.rasterize(ch, size);
        let glyph_x = pen_x + metrics.xmin as f32;
        let glyph_y = baseline_y - metrics.ymin as f32 - metrics.height as f32;

        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let coverage = bitmap[row * metrics.width + col];
                if coverage == 0 {
                    continue;
                }
                let px = (glyph_x + col as f32) as i32;
                let py = (glyph_y + row as f32) as i32;
                blend_pixel(pixmap, px, py, r, g, b, coverage, color.a);
            }
        }
        pen_x += metrics.advance_width;
    }
}

/// Manual "source over" blend of one glyph coverage sample into the pixmap's
/// premultiplied buffer. This is the one place this crate touches raw
/// pixels directly instead of going through tiny-skia's path-filling API --
/// glyph coverage bitmaps from fontdue are exactly that, per-pixel coverage,
/// not geometry, so there's no path to hand tiny-skia in the first place.
fn blend_pixel(pixmap: &mut Pixmap, x: i32, y: i32, r: u8, g: u8, b: u8, coverage: u8, alpha_mul: f32) {
    if x < 0 || y < 0 || x as u32 >= pixmap.width() || y as u32 >= pixmap.height() {
        return;
    }
    let idx = (y as u32 * pixmap.width() + x as u32) as usize;
    let src_a = (coverage as f32 / 255.0) * alpha_mul.clamp(0.0, 1.0);
    if src_a <= 0.0 {
        return;
    }
    let pixels = pixmap.pixels_mut();
    let dst = pixels[idx];
    let inv = 1.0 - src_a;
    let blend = |src_c: u8, dst_c: u8| -> u8 {
        ((src_c as f32) * src_a + (dst_c as f32) * inv).round().clamp(0.0, 255.0) as u8
    };
    let new_r = blend(r, dst.red());
    let new_g = blend(g, dst.green());
    let new_b = blend(b, dst.blue());
    let new_a = ((src_a * 255.0) + (dst.alpha() as f32) * inv).round().clamp(0.0, 255.0) as u8;
    // Clamp components to the (possibly lower) new alpha: PremultipliedColorU8
    // requires r,g,b <= a, and float rounding can occasionally push a
    // premultiplied channel one unit above its own alpha.
    let cap = |c: u8| c.min(new_a);
    if let Some(p) = tiny_skia::PremultipliedColorU8::from_rgba(cap(new_r), cap(new_g), cap(new_b), new_a) {
        pixels[idx] = p;
    }
}

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
use tiny_skia::{FillRule, Mask, Paint, PathBuilder, Pixmap, Rect as SkRect, Transform};

static FONT_BYTES: &[u8] = include_bytes!("../assets/IBMPlexMono-Regular.ttf");

fn font() -> &'static Font {
    static FONT: OnceLock<Font> = OnceLock::new();
    FONT.get_or_init(|| {
        Font::from_bytes(FONT_BYTES, FontSettings::default()).expect("embedded font failed to parse")
    })
}

pub fn text_measure_fn() -> impl Fn(&str, f32) -> (f32, f32) {
    |text: &str, size: f32| {
        let f = font();
        let w: f32 = text.chars().map(|c| f.metrics(c, size).advance_width).sum();
        let h = f.horizontal_line_metrics(size).map(|m| m.new_line_size).unwrap_or(size * 1.4);
        (w, h)
    }
}

/// Rasterize a full draw command list into a new pixmap of the given size
/// (physical pixels), filled with `background` first.
pub fn render_to_pixmap(commands: &[DrawCommand], width: u32, height: u32, background: FColor) -> Pixmap {
    let mut pixmap = Pixmap::new(width.max(1), height.max(1)).expect("non-zero pixmap size");
    pixmap.fill(to_skia_color(background));
    
    let mut clip_stack: Vec<FRect> = Vec::new();
    let mut active_clip: Option<FRect> = None;
    let mut clip_mask = Mask::new(width.max(1), height.max(1)).expect("non-zero mask size");
    let mut mask_active = false;

    for cmd in commands {
        match cmd {
            DrawCommand::PushClip { rect } => {
                let new_clip = if let Some(prev) = active_clip {
                    intersect_rect(prev, *rect).unwrap_or(FRect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 })
                } else {
                    *rect
                };
                clip_stack.push(new_clip);
                active_clip = Some(new_clip);
                update_clip_mask(&mut clip_mask, new_clip, width, height);
                mask_active = true;
            }
            DrawCommand::PopClip => {
                clip_stack.pop();
                active_clip = clip_stack.last().copied();
                if let Some(clip) = active_clip {
                    update_clip_mask(&mut clip_mask, clip, width, height);
                    mask_active = true;
                } else {
                    mask_active = false;
                }
            }
            DrawCommand::Rect { rect, color, corner_radius } => {
                let mut draw_r = *rect;
                if let Some(c) = active_clip {
                    if rect.x > c.x + c.width || rect.y > c.y + c.height || rect.x + rect.width < c.x || rect.y + rect.height < c.y {
                        continue;
                    }
                    if *corner_radius <= 0.0 {
                        if let Some(intersected) = intersect_rect(*rect, c) {
                            draw_r = intersected;
                        }
                    }
                }
                let mask = if mask_active && *corner_radius > 0.0 { Some(&clip_mask) } else { None };
                draw_rect(&mut pixmap, draw_r, *color, *corner_radius, mask)
            }
            DrawCommand::Text { x, y, content, size, color } => {
                if let Some(c) = active_clip {
                    // Approximate text bounds
                    let width = content.chars().count() as f32 * *size * 0.62;
                    if *x > c.x + c.width || *y > c.y + c.height || *x + width < c.x || *y - *size < c.y {
                        continue;
                    }
                }
                draw_text(&mut pixmap, *x, *y, content, *size, *color, active_clip)
            }
        }
    }
    pixmap
}

fn intersect_rect(a: FRect, b: FRect) -> Option<FRect> {
    let x = a.x.max(b.x);
    let y = a.y.max(b.y);
    let right = (a.x + a.width).min(b.x + b.width);
    let bottom = (a.y + a.height).min(b.y + b.height);
    if x < right && y < bottom {
        Some(FRect { x, y, width: right - x, height: bottom - y })
    } else {
        None
    }
}

fn update_clip_mask(clip_mask: &mut Mask, rect: FRect, width: u32, height: u32) {
    clip_mask.clear();
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    if let Some(sk_rect) = SkRect::from_xywh(rect.x, rect.y, rect.width, rect.height) {
        let path = PathBuilder::from_rect(sk_rect);
        clip_mask.fill_path(&path, FillRule::Winding, false, Transform::identity());
    }
}

fn to_skia_color(c: FColor) -> tiny_skia::Color {
    tiny_skia::Color::from_rgba(c.r.clamp(0.0, 1.0), c.g.clamp(0.0, 1.0), c.b.clamp(0.0, 1.0), c.a.clamp(0.0, 1.0))
        .unwrap_or(tiny_skia::Color::BLACK)
}

fn draw_rect(pixmap: &mut Pixmap, rect: FRect, color: FColor, radius: f32, clip: Option<&Mask>) {
    let mut paint = Paint::default();
    paint.set_color(to_skia_color(color));
    paint.anti_alias = true;

    if radius <= 0.0 {
        let Some(r) = SkRect::from_xywh(rect.x, rect.y, rect.width, rect.height) else { return };
        pixmap.fill_rect(r, &paint, Transform::identity(), clip);
    } else {
        let path = rounded_rect_path(rect, radius.min(rect.width / 2.0).min(rect.height / 2.0));
        if let Some(path) = path {
            pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::identity(), clip);
        }
    }
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

fn draw_text(pixmap: &mut Pixmap, x: f32, y: f32, content: &str, size: f32, color: FColor, clip: Option<FRect>) {
    let f = font();
    let (r, g, b) = ((color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8);
    let mut pen_x = x;
    // `y` is the box's top from layout; nudge down by ~80% of size to
    // approximate baseline placement without doing real line-metrics math.
    let baseline_y = y + size * 0.8;

    for ch in content.chars() {
        let metrics = f.metrics(ch, size);
        let glyph_x = pen_x + metrics.xmin as f32;
        let glyph_y = baseline_y - metrics.ymin as f32 - metrics.height as f32;
        
        // Fast clip cull per character
        if let Some(c) = clip {
            if glyph_x > c.x + c.width || glyph_x + (metrics.width as f32) < c.x || glyph_y > c.y + c.height || glyph_y + (metrics.height as f32) < c.y {
                pen_x += metrics.advance_width;
                continue;
            }
        }

        let (_, bitmap) = f.rasterize(ch, size);

        for row in 0..metrics.height {
            for col in 0..metrics.width {
                let coverage = bitmap[row * metrics.width + col];
                if coverage == 0 {
                    continue;
                }
                let px = (glyph_x + col as f32) as i32;
                let py = (glyph_y + row as f32) as i32;
                if let Some(c) = clip {
                    if (px as f32) < c.x || (px as f32) >= c.x + c.width || (py as f32) < c.y || (py as f32) >= c.y + c.height {
                        continue;
                    }
                }
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

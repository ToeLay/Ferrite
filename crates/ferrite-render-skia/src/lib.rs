//! Reference render backend for Ferrite using tiny-skia and cosmic-text

use ferrite_core::{Color as FColor, DrawCommand, Rect as FRect};
use std::cell::RefCell;
use tiny_skia::{FillRule, Mask, Paint, PathBuilder, Pixmap, Rect as SkRect, Transform};

use std::collections::HashMap;

type CacheKey = (usize, u64, u32, Option<u32>, bool);

thread_local! {
    static FONT_SYSTEM: RefCell<cosmic_text::FontSystem> = RefCell::new(cosmic_text::FontSystem::new());
    static SWASH_CACHE: RefCell<cosmic_text::SwashCache> = RefCell::new(cosmic_text::SwashCache::new());
    static BUFFER_CACHE: RefCell<HashMap<CacheKey, (Option<String>, cosmic_text::Buffer)>> = RefCell::new(HashMap::new());
    static GLYPH_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

fn with_buffer<R>(id: usize, version: u64, text: &str, size: f32, max_width: Option<f32>, single_line: bool, f: impl FnOnce(&mut cosmic_text::Buffer) -> R) -> R {
    let size_bits = size.to_bits();
    let max_w_bits = if single_line { None } else { max_width.map(|w| w.to_bits()) };

    let mut buffer = BUFFER_CACHE.with(|bc| {
        let mut cache = bc.borrow_mut();
        let key = (id, version, size_bits, max_w_bits, single_line);
        
        let (mut cached_text, mut b) = if let Some(entry) = cache.remove(&key) {
            entry
        } else {
            let buf = if cache.len() >= 128 {
                let k = cache.keys().next().unwrap().clone();
                cache.remove(&k).unwrap().1
            } else {
                FONT_SYSTEM.with(|fs| {
                    let mut font_system = fs.borrow_mut();
                    let metrics = cosmic_text::Metrics::new(size, size * 1.4);
                    cosmic_text::Buffer::new(&mut font_system, metrics)
                })
            };
            (None, buf)
        };
        
        if cached_text.as_deref() != Some(text) {
            FONT_SYSTEM.with(|fs| {
                let mut font_system = fs.borrow_mut();
                let mut b_fs = b.borrow_with(&mut font_system);
                let metrics = cosmic_text::Metrics::new(size, size * 1.4);
                b_fs.set_metrics(metrics);
                if single_line {
                    b_fs.set_size(None, None);
                } else {
                    b_fs.set_size(max_width, None);
                }
                let attrs = cosmic_text::Attrs::new().family(cosmic_text::Family::SansSerif);
                b_fs.set_text(text, &attrs, cosmic_text::Shaping::Advanced, None);
                if single_line {
                    b_fs.shape_until_scroll(false); // only shape visible content (1 line)
                } else {
                    b_fs.shape_until_scroll(true);  // TextArea needs full height measurement
                }
            });
            cached_text = Some(text.to_string());
        }
        
        (cached_text, b)
    });
    
    let res = f(&mut buffer.1);
    
    BUFFER_CACHE.with(|bc| {
        bc.borrow_mut().insert((id, version, size_bits, max_w_bits, single_line), buffer);
    });
    
    res
}

pub fn text_measure_fn() -> impl Fn(usize, u64, &str, f32, Option<f32>, bool) -> (f32, f32) {
    |id, version, text, size, max_width, single_line| {
        with_buffer(id, version, text, size, max_width, single_line, |buffer| {
            let mut max_w = 0.0f32;
            let mut total_h = 0.0f32;
            
            for run in buffer.layout_runs() {
                max_w = max_w.max(run.line_w);
                total_h += size * 1.4;
            }
            if total_h == 0.0 && !text.is_empty() {
                 total_h = size * 1.4;
            } else if text.is_empty() {
                 total_h = size * 1.4;
            }
            
            (max_w, total_h)
        })
    }
}

pub fn wrap_lines_fn() -> impl Fn(usize, u64, &str, f32, f32) -> Vec<usize> {
    |id, version, text, size, max_width| {
        with_buffer(id, version, text, size, Some(max_width), false, |buffer| {
            let mut res: Vec<usize> = Vec::new();
            
            for line in buffer.lines.iter() {
                let layout = line.layout_opt().unwrap();
                let num_runs = layout.len();
                if num_runs == 0 {
                    res.push(1); // just the \n (or empty line)
                } else {
                    let mut start_byte = 0;
                    for (i, _run) in layout.iter().enumerate() {
                        let end_byte = if i + 1 < num_runs {
                            if let Some(first_glyph) = layout[i + 1].glyphs.first() {
                                first_glyph.start
                            } else {
                                start_byte
                            }
                        } else {
                            line.text().len()
                        };
                        
                        let text_slice = &line.text()[start_byte..end_byte];
                        let mut count = text_slice.chars().count();
                        
                        if i == num_runs - 1 {
                            // The last run of the BufferLine gets the \n if it's not the last line
                            count += 1;
                        }
                        res.push(count);
                        start_byte = end_byte;
                    }
                }
            }
            
            // The last BufferLine doesn't have a trailing \n in the original text usually,
            if let Some(last) = res.last_mut() {
                if *last > 0 {
                    *last -= 1;
                }
            }
            
            if res.is_empty() {
                res.push(text.chars().count());
            }
            
            // Safety check
            let total: usize = res.iter().sum();
            let actual = text.chars().count();
            if total != actual {
                if let Some(last) = res.last_mut() {
                    if actual > total {
                        *last += actual - total;
                    } else if total > actual && *last >= (total - actual) {
                        *last -= total - actual;
                    }
                }
            }
            
            res
        })
    }
}

pub fn char_at_x_fn() -> impl Fn(usize, u64, &str, f32, f32, usize, bool) -> usize {
    |id, version, text, size, target_x, line_idx, single_line| {
        with_buffer(id, version, text, size, None, single_line, |buffer| {
            if let Some(run) = buffer.layout_runs().nth(line_idx) {
                for (glyph_idx, glyph) in run.glyphs.iter().enumerate() {
                    if target_x < glyph.x + glyph.w / 2.0 {
                        return glyph_idx;
                    }
                }
                return run.glyphs.len(); // after last glyph on this run
            }
            0
        })
    }
}

pub fn char_x_at_index_fn() -> impl Fn(usize, u64, &str, f32, usize, usize, bool) -> f32 {
    |id, version, text, size, char_idx, line_idx, single_line| {
        with_buffer(id, version, text, size, None, single_line, |buffer| {
            if let Some(run) = buffer.layout_runs().nth(line_idx) {
                if char_idx >= run.glyphs.len() {
                    if let Some(last) = run.glyphs.last() {
                        return last.x + last.w;
                    }
                    return 0.0;
                }
                return run.glyphs[char_idx].x;
            }
            0.0
        })
    }
}

pub fn render_to_pixmap(commands: &[DrawCommand], width: u32, height: u32, scale: f32, background: FColor) -> Pixmap {
    let mut pixmap = Pixmap::new(width.max(1), height.max(1)).expect("non-zero pixmap size");
    pixmap.fill(to_skia_color(background));
    
    let mut clip_stack: Vec<FRect> = Vec::new();
    let mut active_clip: Option<FRect> = None;
    let mut clip_mask = Mask::new(width.max(1), height.max(1)).expect("non-zero mask size");
    let mut mask_active = false;

    for cmd in commands {
        match cmd {
            DrawCommand::PushClip { rect } => {
                let rect = FRect { x: rect.x * scale, y: rect.y * scale, width: rect.width * scale, height: rect.height * scale };
                let new_clip = if let Some(prev) = active_clip {
                    intersect_rect(prev, rect).unwrap_or(FRect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 })
                } else {
                    rect
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
            DrawCommand::Rect { rect, color, corner_radius } | DrawCommand::StrokeRect { rect, color, corner_radius, .. } => {
                let rect = FRect { x: rect.x * scale, y: rect.y * scale, width: rect.width * scale, height: rect.height * scale };
                let corner_radius = *corner_radius * scale;
                let mut draw_r = rect;
                if let Some(c) = active_clip {
                    if rect.x > c.x + c.width || rect.y > c.y + c.height || rect.x + rect.width < c.x || rect.y + rect.height < c.y {
                        continue;
                    }
                    if corner_radius <= 0.0 {
                        if let Some(intersected) = intersect_rect(rect, c) {
                            draw_r = intersected;
                        }
                    }
                }
                let mask = if mask_active && corner_radius > 0.0 { Some(&clip_mask) } else { None };
                draw_rect(&mut pixmap, draw_r, *color, corner_radius, mask)
            }
            DrawCommand::Text { id, version, x, y, content, size, color, max_width, single_line, center } => {
                let logical_x = *x;
                let logical_y = *y;
                let logical_size = *size;
                let logical_max_width = *max_width;
                
                let _needs_ellipsis = false;
                let mut ellipsis_width = 0.0;
                
                if *single_line && logical_max_width.is_some() {
                    with_buffer(0, 0, "...", logical_size, None, true, |buffer| {
                        if let Some(run) = buffer.layout_runs().next() {
                            ellipsis_width = run.line_w;
                        }
                    });
                }
                
                let mut draw_ellipsis = false;
                let mut ellipsis_x = 0.0;
                let mut ellipsis_y = 0.0;
                
                with_buffer(*id, *version, content, logical_size, logical_max_width, *single_line, |buffer| {
                    let mut x_offset = 0.0;
                    if *center && *single_line {
                        if let Some(run) = buffer.layout_runs().next() {
                            if let Some(mw) = logical_max_width {
                                x_offset = (mw - run.line_w) / 2.0;
                                if x_offset < 0.0 { x_offset = 0.0; }
                            }
                        }
                    }
                    
                    let mut is_truncated = false;
                    if *single_line && logical_max_width.is_some() {
                        let mw = logical_max_width.unwrap();
                        let mut count = 0;
                        for run in buffer.layout_runs() {
                            count += 1;
                            if count == 1 && run.line_w > mw {
                                is_truncated = true;
                            }
                        }
                        if count > 1 { is_truncated = true; }
                    }
                    if is_truncated {
                        draw_ellipsis = true;
                    }
                    
                    let (r, g, b) = ((color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8);
                    
                    let mut run_idx = 0;
                    for run in buffer.layout_runs() {
                        if *single_line && run_idx > 0 { break; }
                        run_idx += 1;
                        
                        let run_y = logical_y + run.line_y;
                        if is_truncated {
                            ellipsis_y = run_y;
                        }
                        
                        // Fast clip cull per line
                        if let Some(c) = active_clip {
                            let p_run_y = run_y * scale;
                            let p_size = logical_size * scale;
                            if p_run_y - p_size > c.y + c.height || p_run_y + p_size < c.y {
                                continue;
                            }
                        }

                        let mut last_glyph_end = 0.0;
                        for glyph in run.glyphs.iter() {
                            if is_truncated {
                                let mw = logical_max_width.unwrap();
                                let cutoff = if mw > ellipsis_width { mw - ellipsis_width } else { 0.0 };
                                let glyph_logical_end = glyph.x + glyph.w;
                                if glyph_logical_end > cutoff {
                                    continue;
                                }
                                last_glyph_end = glyph_logical_end;
                            }
                            
                            let physical_glyph = glyph.physical(((logical_x + x_offset) * scale, (logical_y + run.line_y) * scale), scale);
                            let glyph_x = physical_glyph.x as f32;
                            let _glyph_y = physical_glyph.y as f32;
                            
                            if let Some(c) = active_clip {
                                if glyph_x > c.x + c.width || glyph_x + (logical_size * scale) < c.x {
                                    continue;
                                }
                            }

                            SWASH_CACHE.with(|sc| {
                                FONT_SYSTEM.with(|fs| {
                                    let mut font_system = fs.borrow_mut();
                                    let mut swash_cache = sc.borrow_mut();
                                    
                                    if let Some(image) = swash_cache.get_image(&mut font_system, physical_glyph.cache_key) {
                                        let px_base = physical_glyph.x + image.placement.left;
                                        let py_base = physical_glyph.y - image.placement.top;
                                        
                                        let w = image.placement.width as u32;
                                        let h = image.placement.height as u32;
                                        if w == 0 || h == 0 { return; }
                                        
                                        let alpha_mul_u16 = (color.a.clamp(0.0, 1.0) * 256.0) as u16;
                                        
                                        GLYPH_BUFFER.with(|gb| {
                                            let mut gb = gb.borrow_mut();
                                            let needed = (w * h * 4) as usize;
                                            if gb.len() < needed {
                                                gb.resize(needed, 0);
                                            }
                                            
                                            let mut glyph_pm = tiny_skia::PixmapMut::from_bytes(&mut gb[..needed], w, h).unwrap();
                                            let pixels = glyph_pm.pixels_mut();
                                            
                                            match image.content {
                                                cosmic_text::SwashContent::Mask => {
                                                    for i in 0..pixels.len() {
                                                        let coverage = image.data[i];
                                                        let a = ((coverage as u16 * alpha_mul_u16) >> 8) as u32;
                                                        if a > 0 {
                                                            let pr = (r as u32 * a) / 255;
                                                            let pg = (g as u32 * a) / 255;
                                                            let pb = (b as u32 * a) / 255;
                                                            pixels[i] = tiny_skia::PremultipliedColorU8::from_rgba(pr as u8, pg as u8, pb as u8, a as u8).unwrap_or(tiny_skia::PremultipliedColorU8::TRANSPARENT);
                                                        } else {
                                                            pixels[i] = tiny_skia::PremultipliedColorU8::TRANSPARENT;
                                                        }
                                                    }
                                                }
                                                cosmic_text::SwashContent::Color => {
                                                    for i in 0..pixels.len() {
                                                        let idx = i * 4;
                                                        let cr = image.data[idx];
                                                        let cg = image.data[idx+1];
                                                        let cb = image.data[idx+2];
                                                        let ca = image.data[idx+3];
                                                        let a = ((ca as u16 * alpha_mul_u16) >> 8) as u32;
                                                        if a > 0 {
                                                            let pr = (cr as u32 * a) / 255;
                                                            let pg = (cg as u32 * a) / 255;
                                                            let pb = (cb as u32 * a) / 255;
                                                            pixels[i] = tiny_skia::PremultipliedColorU8::from_rgba(pr as u8, pg as u8, pb as u8, a as u8).unwrap_or(tiny_skia::PremultipliedColorU8::TRANSPARENT);
                                                        } else {
                                                            pixels[i] = tiny_skia::PremultipliedColorU8::TRANSPARENT;
                                                        }
                                                    }
                                                }
                                                cosmic_text::SwashContent::SubpixelMask => {
                                                    for i in 0..pixels.len() {
                                                        let idx = i * 3;
                                                        let _sr = image.data[idx];
                                                        let sg = image.data[idx+1];
                                                        let _sb = image.data[idx+2];
                                                        // Use green channel for alpha coverage as a simple approximation
                                                        let a = ((sg as u16 * alpha_mul_u16) >> 8) as u32;
                                                        if a > 0 {
                                                            let pr = (r as u32 * a) / 255;
                                                            let pg = (g as u32 * a) / 255;
                                                            let pb = (b as u32 * a) / 255;
                                                            pixels[i] = tiny_skia::PremultipliedColorU8::from_rgba(pr as u8, pg as u8, pb as u8, a as u8).unwrap_or(tiny_skia::PremultipliedColorU8::TRANSPARENT);
                                                        } else {
                                                            pixels[i] = tiny_skia::PremultipliedColorU8::TRANSPARENT;
                                                        }
                                                    }
                                                }
                            
                                            }
                                            
                                            pixmap.draw_pixmap(
                                                px_base, py_base,
                                                glyph_pm.as_ref(),
                                                &tiny_skia::PixmapPaint::default(),
                                                tiny_skia::Transform::identity(),
                                                None // we check fast bounds earlier instead of per-pixel clip mask
                                            );
                                        });
                                    }
                                });
                            });
                        }
                        
                        if is_truncated {
                            ellipsis_x = logical_x + x_offset + last_glyph_end;
                        }
                    }
                });
                
                if draw_ellipsis {
                    with_buffer(0, 0, "...", logical_size, None, true, |buffer| {
                        let (r, g, b) = ((color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8);
                        for run in buffer.layout_runs() {
                            for glyph in run.glyphs.iter() {
                            let physical_glyph = glyph.physical((ellipsis_x * scale, ellipsis_y * scale), scale);
                            
                            let glyph_x = physical_glyph.x as f32;
                                
                                if let Some(c) = active_clip {
                                    if glyph_x > c.x + c.width || glyph_x + (logical_size * scale) < c.x {
                                        continue;
                                    }
                                }
                                
                                SWASH_CACHE.with(|sc| {
                                    FONT_SYSTEM.with(|fs| {
                                        let mut font_system = fs.borrow_mut();
                                        let mut swash_cache = sc.borrow_mut();
                                        if let Some(image) = swash_cache.get_image(&mut font_system, physical_glyph.cache_key) {
                                            let px_base = physical_glyph.x + image.placement.left;
                                            let py_base = physical_glyph.y - image.placement.top;
                                            let w = image.placement.width as u32;
                                            let h = image.placement.height as u32;
                                            if w == 0 || h == 0 { return; }
                                            let alpha_mul_u16 = (color.a.clamp(0.0, 1.0) * 256.0) as u16;
                                            
                                            GLYPH_BUFFER.with(|gb| {
                                                let mut gb = gb.borrow_mut();
                                                let needed = (w * h * 4) as usize;
                                                if gb.len() < needed {
                                                    gb.resize(needed, 0);
                                                }
                                                
                                                let mut glyph_pm = tiny_skia::PixmapMut::from_bytes(&mut gb[..needed], w, h).unwrap();
                                                let pixels = glyph_pm.pixels_mut();
                                                
                                                match image.content {
                                                    cosmic_text::SwashContent::Mask => {
                                                        for i in 0..pixels.len() {
                                                            let a = ((image.data[i] as u16 * alpha_mul_u16) >> 8) as u32;
                                                            if a > 0 {
                                                                let pr = (r as u32 * a) / 255;
                                                                let pg = (g as u32 * a) / 255;
                                                                let pb = (b as u32 * a) / 255;
                                                                pixels[i] = tiny_skia::PremultipliedColorU8::from_rgba(pr as u8, pg as u8, pb as u8, a as u8).unwrap_or(tiny_skia::PremultipliedColorU8::TRANSPARENT);
                                                            } else {
                                                                pixels[i] = tiny_skia::PremultipliedColorU8::TRANSPARENT;
                                                            }
                                                        }
                                                    }
                                                    cosmic_text::SwashContent::SubpixelMask => {
                                                        for i in 0..pixels.len() {
                                                            let sg = image.data[i * 3 + 1];
                                                            let a = ((sg as u16 * alpha_mul_u16) >> 8) as u32;
                                                            if a > 0 {
                                                                let pr = (r as u32 * a) / 255;
                                                                let pg = (g as u32 * a) / 255;
                                                                let pb = (b as u32 * a) / 255;
                                                                pixels[i] = tiny_skia::PremultipliedColorU8::from_rgba(pr as u8, pg as u8, pb as u8, a as u8).unwrap_or(tiny_skia::PremultipliedColorU8::TRANSPARENT);
                                                            } else {
                                                                pixels[i] = tiny_skia::PremultipliedColorU8::TRANSPARENT;
                                                            }
                                                        }
                                                    }
                                                    _ => {}
                                                }
                                                
                                                pixmap.draw_pixmap(
                                                    px_base, py_base,
                                                    glyph_pm.as_ref(),
                                                    &tiny_skia::PixmapPaint::default(),
                                                    tiny_skia::Transform::identity(),
                                                    None // we check fast bounds earlier instead of per-pixel clip mask
                                                );
                                            });
                                        }
                                    });
                                });
                            }
                        }
                    });
                }
            }
            DrawCommand::TooltipRegion { .. } => {
                // Ignore. Tooltip metadata for App, not drawn directly.
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

fn update_clip_mask(clip_mask: &mut Mask, rect: FRect, _width: u32, _height: u32) {
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

// blend_pixel removed as we now use tiny_skia::draw_pixmap

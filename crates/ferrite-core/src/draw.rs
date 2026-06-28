use crate::Color;
use ferrite_layout::Rect;

/// What to draw, in absolute (window-space) logical pixels. A render backend
/// (skia, wgpu, whatever) consumes a `&[DrawCommand]` and doesn't need to
/// know anything about widgets, signals, or layout — that separation is the
/// point: swapping the renderer never touches `ferrite-core`.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    Rect { rect: Rect, color: Color, corner_radius: f32 },
    StrokeRect { rect: Rect, color: Color, corner_radius: f32, stroke_width: f32 },
    Text {
        id: usize,
        version: u64,
        x: f32,
        y: f32,
        content: String,
        size: f32,
        color: crate::Color,
        max_width: Option<f32>,
        single_line: bool,
        center: bool,
    },
    PushClip { rect: Rect },
    PopClip,
    TooltipRegion {
        rect: Rect,
        text: String,
    },
}

//! Native window + event loop for Ferrite.
//!
//! This deliberately does *not* reach for wgpu. The `DrawCommand -> pixels`
//! step is already solved by `ferrite-render-skia` on the CPU; what this
//! crate adds is the thinnest possible bridge from "here is a pixel buffer"
//! to "here is that buffer on someone's screen, and here are their clicks" --
//! `winit` for the window and input, `softbuffer` to blit the buffer.
//!
//! A `ferrite-render-wgpu` backend is the natural next crate (same
//! `DrawCommand` input, GPU-accelerated path instead of `tiny-skia`) and
//! wouldn't change anything in `ferrite-core` to add -- that boundary is the
//! whole reason the renderer is a separate crate from the widget tree.

use ferrite_core::App;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle};
use winit::window::{Window, WindowAttributes, WindowId};

pub use ferrite_core::Color;

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub background: Color,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            title: "Ferrite App".to_string(),
            width: 800,
            height: 600,
            background: Color::rgb(1.0, 1.0, 1.0),
        }
    }
}

struct Runner {
    config: WindowConfig,
    app: App,
    context: Context<OwnedDisplayHandle>,
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    cursor_pos: (f64, f64),
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title(self.config.title.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(self.config.width as f64, self.config.height as f64));
        let window = Rc::new(event_loop.create_window(attrs).expect("ferrite-window: failed to create window"));
        let mut surface =
            Surface::new(&self.context, window.clone()).expect("ferrite-window: failed to create softbuffer surface");

        // softbuffer requires `resize` to be called at least once before the
        // first `present` — there's no implicit "use the window's current
        // size" on surface creation. A `Resized` event isn't guaranteed to
        // arrive before the first paint, so this can't be deferred to the
        // `WindowEvent::Resized` handler below; it has to happen right here.
        let size = window.inner_size();
        let w = NonZeroU32::new(size.width.max(1)).unwrap();
        let h = NonZeroU32::new(size.height.max(1)).unwrap();
        surface.resize(w, h).expect("ferrite-window: initial resize failed");

        window.request_redraw();
        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                if let Some(surface) = &mut self.surface {
                    let w = NonZeroU32::new(size.width.max(1)).unwrap();
                    let h = NonZeroU32::new(size.height.max(1)).unwrap();
                    surface.resize(w, h).expect("ferrite-window: resize failed");
                }
                ferrite_core::request_repaint();
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = (position.x, position.y);
            }

            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                self.app.click(self.cursor_pos.0 as f32, self.cursor_pos.1 as f32);
            }

            WindowEvent::RedrawRequested => self.redraw(),

            _ => {}
        }

        // Any of the above (a layout-affecting resize, a click that mutated
        // a signal and re-ran an effect) may have called `request_repaint`.
        // Check once per event rather than threading a "did this change
        // anything" bool through every branch above.
        if ferrite_core::take_dirty() {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

impl Runner {
    fn redraw(&mut self) {
        let Some(window) = self.window.clone() else { return };
        let size = window.inner_size();
        let (w, h) = (size.width.max(1), size.height.max(1));

        let commands = self.app.render(w as f32, h as f32);
        let pixmap = ferrite_render_skia::render_to_pixmap(&commands, w, h, self.config.background);

        let Some(surface) = &mut self.surface else { return };
        let Ok(mut buffer) = surface.buffer_mut() else { return };
        let src = pixmap.data();
        for (i, px) in buffer.iter_mut().enumerate() {
            let o = i * 4;
            let (r, g, b) = (src[o] as u32, src[o + 1] as u32, src[o + 2] as u32);
            *px = (r << 16) | (g << 8) | b; // softbuffer: 0RGB packed u32
        }
        let _ = buffer.present();
    }
}

/// Open a window and run the event loop until it's closed. Blocks the
/// calling thread — this is meant to be the last call in `main`.
pub fn run(config: WindowConfig, app: App) {
    let event_loop = EventLoop::new().expect("ferrite-window: failed to create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let context = Context::new(event_loop.owned_display_handle())
        .expect("ferrite-window: failed to create softbuffer context");

    let mut runner = Runner { config, app, context, window: None, surface: None, cursor_pos: (0.0, 0.0) };
    event_loop.run_app(&mut runner).expect("ferrite-window: event loop error");
}

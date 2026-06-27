use ferrite_core::{App, KeyCode, KeyEvent, Modifiers};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, OwnedDisplayHandle};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};

pub use ferrite_core::Color;

mod clipboard;

pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub background: Color,
}
impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig { title: "Ferrite App".to_string(), width: 800, height: 600, background: Color::rgb(1.0, 1.0, 1.0) }
    }
}

struct Runner {
    config: WindowConfig,
    app: App,
    context: Context<OwnedDisplayHandle>,
    window: Option<Rc<Window>>,
    surface: Option<Surface<OwnedDisplayHandle, Rc<Window>>>,
    cursor_pos: (f64, f64),
    modifiers: winit::keyboard::ModifiersState,
    drag_active: bool,
    drag_start: (f32, f32),
    last_click_time: Option<std::time::Instant>,
    last_click_pos: Option<(f64, f64)>,
    click_count: u32,
}

impl ApplicationHandler for Runner {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let attrs = WindowAttributes::default()
            .with_title(self.config.title.clone())
            .with_inner_size(winit::dpi::LogicalSize::new(self.config.width as f64, self.config.height as f64));
        let window = Rc::new(event_loop.create_window(attrs).expect("create window"));
        let mut surface = Surface::new(&self.context, window.clone()).expect("create surface");
        let size = window.inner_size();
        let w = NonZeroU32::new(size.width.max(1)).unwrap();
        let h = NonZeroU32::new(size.height.max(1)).unwrap();
        surface.resize(w, h).expect("initial resize");
        window.set_ime_allowed(true);
        window.request_redraw();
        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }

            WindowEvent::Resized(size) => {
                if let Some(surface) = &mut self.surface {
                    let w = NonZeroU32::new(size.width.max(1)).unwrap();
                    let h = NonZeroU32::new(size.height.max(1)).unwrap();
                    surface.resize(w, h).expect("resize");
                }
                ferrite_core::request_repaint();
            }

            WindowEvent::CursorMoved { position, .. } => {
                let scale = self.window.as_ref().map(|w| w.scale_factor()).unwrap_or(1.0);
                let logical_x = position.x / scale;
                let logical_y = position.y / scale;
                self.cursor_pos = (logical_x, logical_y);
                self.app.set_hover_pos(Some((logical_x as f32, logical_y as f32)));
                ferrite_core::request_repaint(); // Start hover detection
                if self.drag_active {
                    self.app.drag(self.cursor_pos.0 as f32, self.cursor_pos.1 as f32);
                }
            }

            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let now = std::time::Instant::now();
                let mut current_click_count = 1;
                
                if let (Some(last_time), Some(last_pos)) = (self.last_click_time, self.last_click_pos) {
                    if now.duration_since(last_time).as_millis() < 300 {
                        let dx = self.cursor_pos.0 - last_pos.0;
                        let dy = self.cursor_pos.1 - last_pos.1;
                        if dx * dx + dy * dy < 25.0 {
                            current_click_count = self.click_count + 1;
                        }
                    }
                }
                
                self.click_count = current_click_count;
                self.drag_active = true;
                self.drag_start = (self.cursor_pos.0 as f32, self.cursor_pos.1 as f32);
                
                if current_click_count == 2 {
                    self.app.double_click(self.cursor_pos.0 as f32, self.cursor_pos.1 as f32);
                    self.last_click_time = Some(now);
                    self.last_click_pos = Some(self.cursor_pos);
                } else if current_click_count == 3 {
                    self.app.triple_click(self.cursor_pos.0 as f32, self.cursor_pos.1 as f32);
                    self.last_click_time = None; // Reset so next click is 1
                    self.click_count = 0;
                } else {
                    self.app.click(self.cursor_pos.0 as f32, self.cursor_pos.1 as f32);
                    self.last_click_time = Some(now);
                    self.last_click_pos = Some(self.cursor_pos);
                }
                ferrite_core::request_repaint();
            }
            
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                self.drag_active = false;
                self.app.release_drag();
                ferrite_core::request_repaint();
            }

            WindowEvent::MouseWheel { delta, .. } => {
                let (dx, dy) = match delta {
                    MouseScrollDelta::LineDelta(x, y) => (x * 20.0, y * 20.0),
                    MouseScrollDelta::PixelDelta(pos) => {
                        let scale = self.window.as_ref().map(|w| w.scale_factor()).unwrap_or(1.0);
                        (pos.x as f32 / scale as f32, pos.y as f32 / scale as f32)
                    },
                };
                self.app.scroll(self.cursor_pos.0 as f32, self.cursor_pos.1 as f32, dx, dy);
                ferrite_core::request_repaint();
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let Some(fe) = map_key(&event.logical_key, self.modifiers) {
                        if fe.key == KeyCode::Tab {
                            if fe.modifiers.shift {
                                self.app.focus_prev();
                            } else {
                                self.app.focus_next();
                            }
                        } else {
                            self.app.key_event(fe);
                        }
                        ferrite_core::request_repaint();
                    }
                }
            }

            WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                for ch in text.chars() {
                    self.app.key_event(KeyEvent {
                        key: KeyCode::Char(ch),
                        modifiers: Modifiers::default(),
                    });
                }
                ferrite_core::request_repaint();
            }

            WindowEvent::RedrawRequested => self.redraw(),
            WindowEvent::CursorLeft { .. } => {
                self.app.set_hover_pos(None);
                ferrite_core::request_repaint();
            }

            _ => {}
        }

        if ferrite_core::take_dirty() {
            if let Some(w) = &self.window { w.request_redraw(); }
        }
    }
}

impl Runner {
    fn redraw(&mut self) {
        let Some(window) = self.window.clone() else { return };
        let size = window.inner_size();
        let (w, h) = (size.width.max(1), size.height.max(1));
        
        let scale = window.scale_factor() as f32;
        let logical_w = w as f32 / scale;
        let logical_h = h as f32 / scale;
        
        let commands = self.app.render(logical_w, logical_h);
        
        let pixmap = ferrite_render_skia::render_to_pixmap(&commands, w, h, scale, self.config.background);
        
        let Some(surface) = &mut self.surface else { return };
        let Ok(mut buffer) = surface.buffer_mut() else { return };
        
        let src = pixmap.data();
        for (px, chunk) in buffer.iter_mut().zip(src.chunks_exact(4)) {
            *px = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | chunk[2] as u32;
        }
        
        let _ = buffer.present();
    }
}

fn map_key(key: &Key, mods: winit::keyboard::ModifiersState) -> Option<KeyEvent> {
    let modifiers = Modifiers {
        shift: mods.shift_key(),
        ctrl:  mods.control_key(),
        alt:   mods.alt_key(),
        meta:  mods.super_key(),
    };
    let code = match key {
        Key::Character(s) => KeyCode::Char(s.chars().next()?),
        Key::Named(named) => match named {
            NamedKey::Backspace  => KeyCode::Backspace,
            NamedKey::Delete     => KeyCode::Delete,
            NamedKey::Enter      => KeyCode::Return,
            NamedKey::Tab        => KeyCode::Tab,
            NamedKey::Escape     => KeyCode::Escape,
            NamedKey::ArrowLeft  => KeyCode::Left,
            NamedKey::ArrowRight => KeyCode::Right,
            NamedKey::ArrowUp    => KeyCode::Up,
            NamedKey::ArrowDown  => KeyCode::Down,
            NamedKey::Home       => KeyCode::Home,
            NamedKey::End        => KeyCode::End,
            NamedKey::Space      => KeyCode::Char(' '),
            _ => return None,
        },
        _ => return None,
    };
    Some(KeyEvent { key: code, modifiers })
}

pub fn run(config: WindowConfig, mut app: App) {
    clipboard::init_clipboard();
    app.set_text_measure(ferrite_render_skia::text_measure_fn());
    let event_loop = EventLoop::new().expect("create event loop");
    event_loop.set_control_flow(ControlFlow::Wait);
    let context = Context::new(event_loop.owned_display_handle()).expect("create softbuffer context");
    let mut runner = Runner {
        config, app, context,
        window: None,
        surface: None,
        cursor_pos: (0.0, 0.0),
        modifiers: winit::keyboard::ModifiersState::empty(),
        drag_active: false,
        drag_start: (0.0, 0.0),
        last_click_time: None,
        last_click_pos: None,
        click_count: 0,
    };
    event_loop.run_app(&mut runner).expect("event loop");
}

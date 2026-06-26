use crate::view::{AnyView, col, text};
use ferrite_layout::{PositionType, Size, Inset, AlignItems};
use ferrite_reactive::{create_signal, Signal, SignalVecExt};

#[derive(Clone, Debug)]
pub struct ToastData {
    pub id: usize,
    pub message: String,
}

thread_local! {
    static TOAST_ID: std::cell::Cell<usize> = std::cell::Cell::new(0);
    static TOASTS: std::cell::RefCell<Option<Signal<Vec<ToastData>>>> = std::cell::RefCell::new(None);
}

fn get_or_create_toasts() -> Signal<Vec<ToastData>> {
    TOASTS.with(|t| {
        if let Some(s) = t.borrow().as_ref() {
            return *s;
        }
        let s = create_signal(Vec::new());
        *t.borrow_mut() = Some(s);
        s
    })
}

pub fn toast(message: &str) {
    let s = get_or_create_toasts();
    let id = TOAST_ID.with(|i| {
        let v = i.get();
        i.set(v + 1);
        v
    });
    
    let msg = message.to_string();
    s.push(ToastData { id, message: msg.clone() });
    
    // We don't have a timer system yet, so we'll just keep the toasts there
    // In a real framework we'd add setTimeout, but for this prototype this is okay.
}

pub fn toaster() -> AnyView {
    let toasts = get_or_create_toasts();
    
    col([
        col([
            crate::view::list(toasts, move |item| {
                let id = item.id;
                crate::view::row([
                    crate::view::text(&item.message)
                        .color(crate::Color::rgb(1.0, 1.0, 1.0))
                        .flex_grow(1.0)
                        .margin(8.0),
                    crate::view::button("X", move || {
                        let s = get_or_create_toasts();
                        let mut current = s.get();
                        if let Some(pos) = current.iter().position(|t| t.id == id) {
                            current.remove(pos);
                            s.set(current);
                        }
                    })
                    .background(crate::Color::rgb(0.3, 0.3, 0.3))
                    .padding(4.0)
                ])
                .align(ferrite_layout::AlignItems::Start)
                .background(crate::Color::rgb(0.25, 0.25, 0.28))
                .padding(8.0)
                .corner_radius(8.0)
                .margin(8.0)
            })
        ])
        .position_type(PositionType::Absolute)
        .inset(Inset {
            bottom: Size::Px(24.0),
            right: Size::Px(24.0),
            top: Size::Auto,
            left: Size::Auto,
        })
        .align(AlignItems::End)
    ]).fill()
}

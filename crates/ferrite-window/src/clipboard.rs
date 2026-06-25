use arboard::Clipboard;
use std::sync::Mutex;
use std::sync::OnceLock;

static CLIPBOARD: OnceLock<Mutex<Option<Clipboard>>> = OnceLock::new();

pub(crate) fn init_clipboard() {
    let _ = CLIPBOARD.set(Mutex::new(Clipboard::new().ok()));
    
    // Register the hooks with ferrite-core
    ferrite_core::clipboard::set_clipboard_hooks(
        || {
            if let Some(mutex) = CLIPBOARD.get() {
                if let Ok(mut lock) = mutex.lock() {
                    if let Some(cb) = lock.as_mut() {
                        return cb.get_text().ok();
                    }
                }
            }
            None
        },
        |text| {
            if let Some(mutex) = CLIPBOARD.get() {
                if let Ok(mut lock) = mutex.lock() {
                    if let Some(cb) = lock.as_mut() {
                        let _ = cb.set_text(text);
                    }
                }
            }
        },
    );
}

//! Clipboard hook definitions for Ferrite.

static mut GET_TEXT_HOOK: fn() -> Option<String> = || None;
static mut SET_TEXT_HOOK: fn(String) = |_| {};

/// Internal API used by the windowing shell (e.g. ferrite-window) to register clipboard access.
pub fn set_clipboard_hooks(get_fn: fn() -> Option<String>, set_fn: fn(String)) {
    unsafe {
        GET_TEXT_HOOK = get_fn;
        SET_TEXT_HOOK = set_fn;
    }
}

/// Retrieves the text from the OS clipboard, if available.
pub fn get_text() -> Option<String> {
    unsafe { GET_TEXT_HOOK() }
}

/// Sets the text on the OS clipboard.
pub fn set_text(text: impl Into<String>) {
    unsafe { SET_TEXT_HOOK(text.into()) }
}

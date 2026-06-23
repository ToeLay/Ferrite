use std::cell::Cell;

thread_local! {
    static DIRTY: Cell<bool> = const { Cell::new(true) };
}

/// Called by reactive effects wired into widgets (e.g. `text_dyn`) whenever
/// the value they're displaying changes. The event loop checks this once per
/// iteration via [`take_dirty`] and only repaints when it's actually needed —
/// a `create_effect` firing doesn't imply a frame was wasted.
pub fn request_repaint() {
    DIRTY.with(|d| d.set(true));
}

/// Returns whether a repaint was requested since the last call, clearing the flag.
pub fn take_dirty() -> bool {
    DIRTY.with(|d| d.replace(false))
}

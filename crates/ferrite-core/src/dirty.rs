use std::cell::Cell;
use std::cell::RefCell;
use ferrite_layout::NodeId;

thread_local! {
    static DIRTY: Cell<bool> = const { Cell::new(true) };
    static DIRTY_NODES: RefCell<Vec<NodeId>> = RefCell::new(Vec::new());
}

/// Called by reactive effects wired into widgets (e.g. `text_dyn`) whenever
/// the value they're displaying changes. The event loop checks this once per
/// iteration via [`take_dirty`] and only repaints when it's actually needed —
/// a `create_effect` firing doesn't imply a frame was wasted.
pub fn request_repaint() {
    DIRTY.with(|d| d.set(true));
}

pub fn request_layout(node: NodeId) {
    DIRTY.with(|d| d.set(true));
    DIRTY_NODES.with(|n| n.borrow_mut().push(node));
}

/// Returns whether a repaint was requested since the last call, clearing the flag.
pub fn take_dirty() -> bool {
    DIRTY.with(|d| d.replace(false))
}

pub fn take_dirty_nodes() -> Vec<NodeId> {
    DIRTY_NODES.with(|n| {
        let mut list = n.borrow_mut();
        std::mem::replace(&mut *list, Vec::new())
    })
}

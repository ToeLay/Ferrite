use std::cell::RefCell;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct OverlayId(pub(crate) usize);

thread_local! {
    pub(crate) static PENDING_OVERLAYS: RefCell<Vec<(OverlayId, crate::view::AnyView)>> = RefCell::new(Vec::new());
    pub(crate) static REMOVED_OVERLAYS: RefCell<Vec<OverlayId>> = RefCell::new(Vec::new());
    static NEXT_ID: std::cell::Cell<usize> = std::cell::Cell::new(0);
}

/// Imperatively display an overlay, returning an ID you can use to remove it.
pub fn show_overlay(view: crate::view::AnyView) -> OverlayId {
    let id = OverlayId(NEXT_ID.get());
    NEXT_ID.set(id.0 + 1);
    PENDING_OVERLAYS.with(|o| o.borrow_mut().push((id, view)));
    crate::dirty::request_repaint();
    id
}

/// Remove a previously shown overlay.
pub fn remove_overlay(id: OverlayId) {
    REMOVED_OVERLAYS.with(|o| o.borrow_mut().push(id));
    crate::dirty::request_repaint();
}

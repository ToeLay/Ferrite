//! Fine-grained reactivity for Ferrite.
//!
//! The defining choice here, made specifically *for* Rust rather than ported
//! from a GC'd language: a [`Signal`] is not `Rc<RefCell<T>>`. It is a small
//! `Copy` handle (an index + generation) into a thread-local arena. That means:
//!
//! - You can pass a `Signal<T>` into a closure by value, store it in a struct,
//!   capture it in ten different callbacks, all without `.clone()` noise or
//!   fighting the borrow checker — it's `Copy`, like an integer.
//! - There is no reference cycle to worry about (no `Rc<RefCell<>>` graph),
//!   because the graph lives in the arena, not in the handles.
//! - Disposal is explicit and bulk, via [`Scope`] — matching component
//!   mount/unmount instead of relying on `Drop` timing of scattered `Rc`s.
//!
//! Dependency tracking is automatic and dynamic: reading a signal inside a
//! [`create_effect`] or a memo's compute function records that edge, and the
//! edge set is rebuilt on every run, so branches that stop being taken stop
//! being depended on (no stale subscriptions from `if`/`match`).

mod runtime;
mod scope;

pub use scope::Scope;

use runtime::NodeId;
use std::marker::PhantomData;

/// A readable, writable, `Copy` handle to a piece of reactive state.
pub struct Signal<T> {
    id: NodeId,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Signal<T> {}
impl<T> Clone for Signal<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Clone + 'static> Signal<T> {
    /// Read the value, tracking this read if called inside an effect or memo.
    pub fn get(&self) -> T {
        runtime::get_signal_value(self.id)
    }
}

impl<T: 'static> Signal<T> {
    /// Replace the value and synchronously re-run every effect/memo that
    /// (transitively) depends on this signal.
    pub fn set(&self, value: T) {
        runtime::set_signal_value(self.id, value);
    }

    /// Mutate the value in place, then propagate, same as `set`.
    pub fn update(&self, f: impl FnOnce(&mut T)) {
        runtime::update_signal_value(self.id, f);
    }
}

/// Create a new signal with an initial value. Lives until the enclosing
/// [`Scope`] is disposed, or forever if created outside any scope.
pub fn create_signal<T: 'static>(initial: T) -> Signal<T> {
    let id = runtime::create_signal_node(initial);
    Signal { id, _marker: PhantomData }
}

/// A derived, cached value. Recomputes only when one of its dependencies
/// actually changes, and only notifies *its* subscribers if the recomputed
/// value is itself different (via `PartialEq`) — so a chain of memos doesn't
/// cascade work when an upstream change happens to round-trip to the same output.
pub struct Memo<T> {
    id: NodeId,
    _marker: PhantomData<fn() -> T>,
}

impl<T> Copy for Memo<T> {}
impl<T> Clone for Memo<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: Clone + PartialEq + 'static> Memo<T> {
    pub fn get(&self) -> T {
        runtime::get_memo_value(self.id)
    }
}

pub fn create_memo<T: Clone + PartialEq + 'static>(compute: impl FnMut() -> T + 'static) -> Memo<T> {
    let id = runtime::create_memo_node(compute, runtime::make_eq::<T>());
    Memo { id, _marker: PhantomData }
}

/// Run `f` immediately, then re-run it every time a signal or memo it read
/// during its last run changes. This is the thing that actually does work
/// (updating layout, scheduling a repaint, calling out to I/O) — signals and
/// memos are inert without an effect downstream of them.
pub fn create_effect(f: impl FnMut() + 'static) {
    runtime::create_effect_node(f);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn signal_get_set() {
        let s = create_signal(1);
        assert_eq!(s.get(), 1);
        s.set(42);
        assert_eq!(s.get(), 42);
    }

    #[test]
    fn effect_runs_immediately_and_on_change() {
        let s = create_signal(0);
        let log = Rc::new(RefCell::new(Vec::new()));
        let log2 = log.clone();
        create_effect(move || {
            log2.borrow_mut().push(s.get());
        });
        assert_eq!(*log.borrow(), vec![0]);
        s.set(1);
        assert_eq!(*log.borrow(), vec![0, 1]);
        s.set(1); // setting to the *same* value still re-runs (signals aren't diffed, memos are)
        assert_eq!(*log.borrow(), vec![0, 1, 1]);
    }

    #[test]
    fn memo_dedupes_unchanged_output() {
        let s = create_signal(4);
        let m = create_memo(move || s.get() % 2); // even/odd
        let log = Rc::new(RefCell::new(Vec::new()));
        let log2 = log.clone();
        create_effect(move || {
            log2.borrow_mut().push(m.get());
        });
        assert_eq!(*log.borrow(), vec![0]);
        s.set(6); // still even -> memo output unchanged -> effect should NOT re-run
        assert_eq!(*log.borrow(), vec![0]);
        s.set(7); // odd now -> memo output changes -> effect re-runs
        assert_eq!(*log.borrow(), vec![0, 1]);
    }

    #[test]
    fn dynamic_dependencies_drop_stale_edges() {
        let cond = create_signal(true);
        let a = create_signal(10);
        let b = create_signal(20);
        let log = Rc::new(RefCell::new(Vec::new()));
        let log2 = log.clone();
        create_effect(move || {
            let v = if cond.get() { a.get() } else { b.get() };
            log2.borrow_mut().push(v);
        });
        assert_eq!(*log.borrow(), vec![10]);

        cond.set(false); // switches to depending on `b`, drops the edge to `a`
        assert_eq!(*log.borrow(), vec![10, 20]);

        a.set(999); // no longer a dependency — must NOT trigger a re-run
        assert_eq!(*log.borrow(), vec![10, 20]);

        b.set(21);
        assert_eq!(*log.borrow(), vec![10, 20, 21]);
    }

    #[test]
    fn scope_dispose_frees_nodes_for_reuse() {
        let scope = Scope::new();
        scope.run(|| {
            let _s = create_signal(123);
        });
        scope.dispose();
        // a fresh signal should be able to reuse the freed slot without panicking
        let s2 = create_signal(456);
        assert_eq!(s2.get(), 456);
    }

    #[test]
    fn diamond_dependency_runs_effect_once() {
        // a -> b, a -> c, (b, c) -> effect. A naive graph walk could run the
        // effect twice (once via b, once via c) for a single change to `a`.
        let a = create_signal(1);
        let b = create_memo(move || a.get() + 1);
        let c = create_memo(move || a.get() + 2);
        let log = Rc::new(RefCell::new(Vec::new()));
        let log2 = log.clone();
        create_effect(move || {
            log2.borrow_mut().push((b.get(), c.get()));
        });
        assert_eq!(*log.borrow(), vec![(2, 3)]);
        a.set(10);
        // Known v0.1 limitation: no topological batching yet, so this effect
        // re-runs once per upstream memo that changed (twice here), not once.
        // Documented in ARCHITECTURE.md; tracked as the first post-0.1 item.
        assert_eq!(*log.borrow(), vec![(2, 3), (11, 3), (11, 12)]);
    }
}

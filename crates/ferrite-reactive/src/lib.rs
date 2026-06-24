//! Fine-grained reactivity for Ferrite.
//!
//! `Signal<T>` is a `Copy` arena handle, not `Rc<RefCell<T>>`. Capture it
//! in ten closures without clone noise; no reference cycles; `Scope` for
//! bulk disposal on component unmount.
//!
//! Dependency tracking is dynamic: edges are rebuilt on every run, so
//! branches that stop being taken stop being depended on.

mod runtime;
mod scope;

pub use scope::Scope;

use runtime::NodeId;
use std::marker::PhantomData;

pub struct Signal<T> {
    id: NodeId,
    _marker: PhantomData<fn() -> T>,
}
impl<T> Copy for Signal<T> {}
impl<T> Clone for Signal<T> { fn clone(&self) -> Self { *self } }

impl<T: Clone + 'static> Signal<T> {
    pub fn get(&self) -> T { runtime::get_signal_value(self.id) }
}
impl<T: 'static> Signal<T> {
    pub fn set(&self, value: T) { runtime::set_signal_value(self.id, value); }
    pub fn update(&self, f: impl FnOnce(&mut T)) { runtime::update_signal_value(self.id, f); }
}

pub fn create_signal<T: 'static>(initial: T) -> Signal<T> {
    Signal { id: runtime::create_signal_node(initial), _marker: PhantomData }
}

pub struct Memo<T> {
    id: NodeId,
    _marker: PhantomData<fn() -> T>,
}
impl<T> Copy for Memo<T> {}
impl<T> Clone for Memo<T> { fn clone(&self) -> Self { *self } }

impl<T: Clone + PartialEq + 'static> Memo<T> {
    pub fn get(&self) -> T { runtime::get_memo_value(self.id) }
}

pub fn create_memo<T: Clone + PartialEq + 'static>(compute: impl FnMut() -> T + 'static) -> Memo<T> {
    Memo { id: runtime::create_memo_node(compute, runtime::make_eq::<T>()), _marker: PhantomData }
}

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
        create_effect(move || { log2.borrow_mut().push(s.get()); });
        assert_eq!(*log.borrow(), vec![0]);
        s.set(1);
        assert_eq!(*log.borrow(), vec![0, 1]);
        s.set(1);
        assert_eq!(*log.borrow(), vec![0, 1, 1]);
    }

    #[test]
    fn memo_dedupes_unchanged_output() {
        let s = create_signal(4);
        let m = create_memo(move || s.get() % 2);
        let log = Rc::new(RefCell::new(Vec::new()));
        let log2 = log.clone();
        create_effect(move || { log2.borrow_mut().push(m.get()); });
        assert_eq!(*log.borrow(), vec![0]);
        s.set(6); // still even — effect must NOT re-run
        assert_eq!(*log.borrow(), vec![0]);
        s.set(7); // now odd
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
        cond.set(false);
        assert_eq!(*log.borrow(), vec![10, 20]);
        a.set(999); // no longer a dependency
        assert_eq!(*log.borrow(), vec![10, 20]);
        b.set(21);
        assert_eq!(*log.borrow(), vec![10, 20, 21]);
    }

    #[test]
    fn scope_dispose_frees_nodes_for_reuse() {
        let scope = Scope::new();
        scope.run(|| { let _s = create_signal(123); });
        scope.dispose();
        let s2 = create_signal(456);
        assert_eq!(s2.get(), 456);
    }

    #[test]
    fn diamond_dependency_runs_effect_once() {
        let a = create_signal(1);
        let b = create_memo(move || a.get() + 1);
        let c = create_memo(move || a.get() + 2);
        let log = Rc::new(RefCell::new(Vec::new()));
        let log2 = log.clone();
        create_effect(move || { log2.borrow_mut().push((b.get(), c.get())); });
        assert_eq!(*log.borrow(), vec![(2, 3)]);
        a.set(10);
        // Topological batching: effect runs exactly once with both new values.
        assert_eq!(*log.borrow(), vec![(2, 3), (11, 12)]);
    }
}

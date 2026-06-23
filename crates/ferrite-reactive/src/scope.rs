use crate::runtime::{self, NodeId};
use std::cell::RefCell;

/// Groups every signal/memo/effect created while it runs, so they can all be
/// torn down together — e.g. when a component unmounts, its whole subtree of
/// reactive nodes is freed in one call instead of leaking.
pub struct Scope {
    ids: RefCell<Vec<NodeId>>,
}

impl Scope {
    pub fn new() -> Self {
        Scope { ids: RefCell::new(Vec::new()) }
    }

    /// Run `f`, capturing every reactive node it creates as belonging to this scope.
    /// Nested scopes are supported: an inner `Scope::run` only captures nodes
    /// created during its own call, not ones from an outer scope.
    pub fn run<R>(&self, f: impl FnOnce() -> R) -> R {
        runtime::push_scope();
        let result = f();
        let created = runtime::pop_scope();
        self.ids.borrow_mut().extend(created);
        result
    }

    /// Free every node this scope captured. Signals/memos/effects created here
    /// become unusable afterward (reading them will panic) — that's intentional:
    /// it's the same guarantee a dropped widget gives you.
    pub fn dispose(self) {
        for id in self.ids.into_inner() {
            runtime::dispose_node(id);
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}

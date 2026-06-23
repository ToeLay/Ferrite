use std::any::Any;
use std::cell::RefCell;

/// A tiny insertion-ordered set. We deliberately avoid `HashSet<NodeId>` here:
/// its iteration order is randomized per-process (`RandomState`), which would
/// make propagation order — and therefore effect run order on a diamond
/// dependency — nondeterministic between runs of the *same* program. Edge
/// counts per node are small (a handful at most), so linear contains/remove
/// is the right tradeoff for determinism over micro-optimization.
#[derive(Default)]
pub(crate) struct EdgeSet(Vec<NodeId>);

impl EdgeSet {
    fn insert(&mut self, id: NodeId) {
        if !self.0.contains(&id) {
            self.0.push(id);
        }
    }
    fn remove(&mut self, id: &NodeId) {
        self.0.retain(|x| x != id);
    }
    fn drain(&mut self) -> Vec<NodeId> {
        std::mem::take(&mut self.0)
    }
    fn iter(&self) -> impl Iterator<Item = &NodeId> {
        self.0.iter()
    }
}

/// A `Copy` handle into the reactive arena. Carries a generation counter so a
/// freed-and-reused slot can never be mistaken for the node that used to live there.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

type AnyEq = Box<dyn Fn(&dyn Any, &dyn Any) -> bool>;

pub(crate) enum NodeKind {
    Signal {
        value: Box<dyn Any>,
        subscribers: EdgeSet,
    },
    Memo {
        compute: Box<dyn FnMut() -> Box<dyn Any>>,
        eq: AnyEq,
        value: Option<Box<dyn Any>>,
        sources: EdgeSet,
        subscribers: EdgeSet,
    },
    Effect {
        run: Box<dyn FnMut()>,
        sources: EdgeSet,
    },
}

struct Slot {
    generation: u32,
    kind: Option<NodeKind>,
}

pub(crate) struct Runtime {
    slots: Vec<Slot>,
    free: Vec<u32>,
    observer_stack: Vec<NodeId>,
    scope_stack: Vec<Vec<NodeId>>,
}

impl Runtime {
    fn new() -> Self {
        Runtime {
            slots: Vec::new(),
            free: Vec::new(),
            observer_stack: Vec::new(),
            scope_stack: Vec::new(),
        }
    }

    fn alloc(&mut self, kind: NodeKind) -> NodeId {
        let id = if let Some(index) = self.free.pop() {
            let slot = &mut self.slots[index as usize];
            slot.kind = Some(kind);
            NodeId { index, generation: slot.generation }
        } else {
            let index = self.slots.len() as u32;
            self.slots.push(Slot { generation: 0, kind: Some(kind) });
            NodeId { index, generation: 0 }
        };
        if let Some(top) = self.scope_stack.last_mut() {
            top.push(id);
        }
        id
    }

    fn dispose(&mut self, id: NodeId) {
        if let Some(slot) = self.slots.get_mut(id.index as usize) {
            if slot.generation == id.generation && slot.kind.is_some() {
                slot.kind = None;
                slot.generation = slot.generation.wrapping_add(1);
                self.free.push(id.index);
            }
        }
    }

    pub(crate) fn get_kind(&self, id: NodeId) -> Option<&NodeKind> {
        self.slots
            .get(id.index as usize)
            .filter(|s| s.generation == id.generation)
            .and_then(|s| s.kind.as_ref())
    }

    pub(crate) fn get_kind_mut(&mut self, id: NodeId) -> Option<&mut NodeKind> {
        self.slots
            .get_mut(id.index as usize)
            .filter(|s| s.generation == id.generation)
            .and_then(|s| s.kind.as_mut())
    }

    fn remove_subscriber(&mut self, of: NodeId, subscriber: NodeId) {
        if let Some(kind) = self.get_kind_mut(of) {
            match kind {
                NodeKind::Signal { subscribers, .. } => { subscribers.remove(&subscriber); }
                NodeKind::Memo { subscribers, .. } => { subscribers.remove(&subscriber); }
                NodeKind::Effect { .. } => {}
            }
        }
    }
}

thread_local! {
    static RUNTIME: RefCell<Runtime> = RefCell::new(Runtime::new());
}

pub(crate) fn with_runtime<R>(f: impl FnOnce(&mut Runtime) -> R) -> R {
    RUNTIME.with(|rt| f(&mut rt.borrow_mut()))
}

/// Record a read: if a reactive computation is currently running (the top of the
/// observer stack), wire it up as a subscriber of `id` and `id` as one of its sources.
pub(crate) fn track(id: NodeId) {
    with_runtime(|rt| {
        let Some(&observer) = rt.observer_stack.last() else { return };
        if let Some(kind) = rt.get_kind_mut(id) {
            match kind {
                NodeKind::Signal { subscribers, .. } => { subscribers.insert(observer); }
                NodeKind::Memo { subscribers, .. } => { subscribers.insert(observer); }
                NodeKind::Effect { .. } => {}
            }
        }
        if let Some(kind) = rt.get_kind_mut(observer) {
            match kind {
                NodeKind::Memo { sources, .. } => { sources.insert(id); }
                NodeKind::Effect { sources, .. } => { sources.insert(id); }
                NodeKind::Signal { .. } => {}
            }
        }
    });
}

pub(crate) fn create_signal_node<T: 'static>(value: T) -> NodeId {
    with_runtime(|rt| {
        rt.alloc(NodeKind::Signal { value: Box::new(value), subscribers: EdgeSet::default() })
    })
}

pub(crate) fn create_memo_node<T, F>(compute: F, eq: AnyEq) -> NodeId
where
    T: 'static,
    F: FnMut() -> T + 'static,
{
    let mut compute = compute;
    let boxed_compute: Box<dyn FnMut() -> Box<dyn Any>> = Box::new(move || Box::new(compute()));
    let id = with_runtime(|rt| {
        rt.alloc(NodeKind::Memo {
            compute: boxed_compute,
            eq,
            value: None,
            sources: EdgeSet::default(),
            subscribers: EdgeSet::default(),
        })
    });
    run_memo(id);
    id
}

pub(crate) fn create_effect_node<F: FnMut() + 'static>(run: F) -> NodeId {
    let id = with_runtime(|rt| {
        rt.alloc(NodeKind::Effect { run: Box::new(run), sources: EdgeSet::default() })
    });
    run_effect(id);
    id
}

pub(crate) fn get_signal_value<T: Clone + 'static>(id: NodeId) -> T {
    track(id);
    with_runtime(|rt| match rt.get_kind(id) {
        Some(NodeKind::Signal { value, .. }) => value
            .downcast_ref::<T>()
            .cloned()
            .expect("ferrite-reactive: signal type mismatch (handle reused after dispose?)"),
        _ => panic!("ferrite-reactive: signal does not exist (disposed?)"),
    })
}

pub(crate) fn get_memo_value<T: Clone + 'static>(id: NodeId) -> T {
    track(id);
    with_runtime(|rt| match rt.get_kind(id) {
        Some(NodeKind::Memo { value: Some(v), .. }) => v
            .downcast_ref::<T>()
            .cloned()
            .expect("ferrite-reactive: memo type mismatch"),
        _ => panic!("ferrite-reactive: memo has no value yet"),
    })
}

pub(crate) fn set_signal_value<T: 'static>(id: NodeId, value: T) {
    with_runtime(|rt| {
        if let Some(NodeKind::Signal { value: slot, .. }) = rt.get_kind_mut(id) {
            *slot = Box::new(value);
        }
    });
    notify(id);
}

pub(crate) fn update_signal_value<T: 'static>(id: NodeId, f: impl FnOnce(&mut T)) {
    with_runtime(|rt| {
        if let Some(NodeKind::Signal { value, .. }) = rt.get_kind_mut(id) {
            if let Some(v) = value.downcast_mut::<T>() {
                f(v);
            }
        }
    });
    notify(id);
}

/// Detach old source edges, run the computation with the observer stack pushed
/// (so signals read inside record fresh edges), then reattach. This never holds
/// the runtime's RefCell borrow while user code runs, so re-entrant `.get()` calls
/// inside the closure are safe.
fn run_effect(id: NodeId) {
    let mut f: Box<dyn FnMut()> = with_runtime(|rt| {
        if let Some(NodeKind::Effect { sources, .. }) = rt.get_kind_mut(id) {
            let old: Vec<NodeId> = sources.drain();
            for src in old {
                rt.remove_subscriber(src, id);
            }
        }
        match rt.get_kind_mut(id) {
            Some(NodeKind::Effect { run, .. }) => std::mem::replace(run, Box::new(|| {})),
            _ => Box::new(|| {}),
        }
    });

    with_runtime(|rt| rt.observer_stack.push(id));
    f();
    with_runtime(|rt| { rt.observer_stack.pop(); });

    with_runtime(|rt| {
        if let Some(NodeKind::Effect { run, .. }) = rt.get_kind_mut(id) {
            *run = f;
        } else {
            // node was disposed while running; drop the closure
            let _ = &mut f;
        }
    });
}

/// Same dance as `run_effect`, but also compares the new value against the old
/// one (via the type-erased `eq`) so a memo whose output didn't change does not
/// wake up its own subscribers. Returns whether the value changed.
fn run_memo(id: NodeId) -> bool {
    let mut compute: Box<dyn FnMut() -> Box<dyn Any>> = with_runtime(|rt| {
        if let Some(NodeKind::Memo { sources, .. }) = rt.get_kind_mut(id) {
            let old: Vec<NodeId> = sources.drain();
            for src in old {
                rt.remove_subscriber(src, id);
            }
        }
        match rt.get_kind_mut(id) {
            Some(NodeKind::Memo { compute, .. }) => {
                std::mem::replace(compute, Box::new(|| -> Box<dyn Any> { Box::new(()) }))
            }
            _ => Box::new(|| -> Box<dyn Any> { Box::new(()) }),
        }
    });

    with_runtime(|rt| rt.observer_stack.push(id));
    let new_value = compute();
    with_runtime(|rt| { rt.observer_stack.pop(); });

    with_runtime(|rt| {
        if let Some(NodeKind::Memo { compute: c, eq, value, .. }) = rt.get_kind_mut(id) {
            *c = compute;
            let changed = match value {
                Some(old) => !eq(old.as_ref(), new_value.as_ref()),
                None => true,
            };
            *value = Some(new_value);
            changed
        } else {
            false
        }
    })
}

/// Walk the subscriber graph from `id` outward, re-running effects and
/// recomputing memos (which only propagate further if their value actually changed).
fn notify(id: NodeId) {
    let subs: Vec<NodeId> = with_runtime(|rt| match rt.get_kind(id) {
        Some(NodeKind::Signal { subscribers, .. }) => subscribers.iter().copied().collect(),
        Some(NodeKind::Memo { subscribers, .. }) => subscribers.iter().copied().collect(),
        _ => Vec::new(),
    });
    for sub in subs {
        let is_memo = with_runtime(|rt| matches!(rt.get_kind(sub), Some(NodeKind::Memo { .. })));
        if is_memo {
            if run_memo(sub) {
                notify(sub);
            }
        } else {
            run_effect(sub);
        }
    }
}

pub(crate) fn make_eq<T: PartialEq + 'static>() -> AnyEq {
    Box::new(|a: &dyn Any, b: &dyn Any| match (a.downcast_ref::<T>(), b.downcast_ref::<T>()) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    })
}

pub(crate) fn push_scope() {
    with_runtime(|rt| rt.scope_stack.push(Vec::new()));
}

pub(crate) fn pop_scope() -> Vec<NodeId> {
    with_runtime(|rt| rt.scope_stack.pop().unwrap_or_default())
}

pub(crate) fn dispose_node(id: NodeId) {
    with_runtime(|rt| rt.dispose(id));
}

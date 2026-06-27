use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct EdgeSet(Vec<NodeId>);

impl EdgeSet {
    fn insert(&mut self, id: NodeId) {
        if !self.0.contains(&id) { self.0.push(id); }
    }
    fn remove(&mut self, id: &NodeId) { self.0.retain(|x| x != id); }
    fn drain(&mut self) -> Vec<NodeId> { std::mem::take(&mut self.0) }
    fn iter(&self) -> impl Iterator<Item = &NodeId> { self.0.iter() }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NodeId {
    pub(crate) index: u32,
    pub(crate) generation: u32,
}

type AnyEq = Box<dyn Fn(&dyn Any, &dyn Any) -> bool>;

pub(crate) enum NodeKind {
    Signal  { value: Box<dyn Any>, subscribers: EdgeSet, mutations: Box<dyn Any>, revision: usize },
    Memo    { compute: Box<dyn FnMut() -> Box<dyn Any>>, eq: AnyEq,
               value: Option<Box<dyn Any>>, sources: EdgeSet, subscribers: EdgeSet },
    Effect  { run: Box<dyn FnMut()>, sources: EdgeSet },
}

struct Slot { generation: u32, kind: Option<NodeKind> }

pub(crate) struct Runtime {
    slots: Vec<Slot>,
    free: Vec<u32>,
    observer_stack: Vec<NodeId>,
    scope_stack: Vec<Vec<NodeId>>,
}

impl Runtime {
    fn new() -> Self {
        Runtime { slots: Vec::new(), free: Vec::new(), observer_stack: Vec::new(), scope_stack: Vec::new() }
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
        if let Some(top) = self.scope_stack.last_mut() { top.push(id); }
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
        self.slots.get(id.index as usize)
            .filter(|s| s.generation == id.generation)
            .and_then(|s| s.kind.as_ref())
    }
    pub(crate) fn get_kind_mut(&mut self, id: NodeId) -> Option<&mut NodeKind> {
        self.slots.get_mut(id.index as usize)
            .filter(|s| s.generation == id.generation)
            .and_then(|s| s.kind.as_mut())
    }
    fn remove_subscriber(&mut self, of: NodeId, subscriber: NodeId) {
        if let Some(kind) = self.get_kind_mut(of) {
            match kind {
                NodeKind::Signal { subscribers, .. } => { subscribers.remove(&subscriber); }
                NodeKind::Memo   { subscribers, .. } => { subscribers.remove(&subscriber); }
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

pub(crate) fn track(id: NodeId) {
    with_runtime(|rt| {
        let Some(&observer) = rt.observer_stack.last() else { return };
        if let Some(kind) = rt.get_kind_mut(id) {
            match kind {
                NodeKind::Signal { subscribers, .. } => { subscribers.insert(observer); }
                NodeKind::Memo   { subscribers, .. } => { subscribers.insert(observer); }
                NodeKind::Effect { .. } => {}
            }
        }
        if let Some(kind) = rt.get_kind_mut(observer) {
            match kind {
                NodeKind::Memo   { sources, .. } => { sources.insert(id); }
                NodeKind::Effect { sources, .. } => { sources.insert(id); }
                NodeKind::Signal { .. } => {}
            }
        }
    });
}

pub(crate) fn create_signal_node<T: 'static>(initial: T) -> NodeId {
    with_runtime(|rt| rt.alloc(NodeKind::Signal {
        value: Box::new(initial),
        subscribers: EdgeSet::default(),
        mutations: Box::new(()),
        revision: 0,
    }))
}

pub(crate) fn create_memo_node<T, F>(compute: F, eq: AnyEq) -> NodeId
where T: 'static, F: FnMut() -> T + 'static {
    let mut compute = compute;
    let boxed: Box<dyn FnMut() -> Box<dyn Any>> = Box::new(move || Box::new(compute()));
    let id = with_runtime(|rt| rt.alloc(NodeKind::Memo {
        compute: boxed, eq, value: None,
        sources: EdgeSet::default(), subscribers: EdgeSet::default(),
    }));
    run_memo(id);
    id
}

pub(crate) fn create_effect_node<F: FnMut() + 'static>(run: F) -> NodeId {
    let id = with_runtime(|rt| rt.alloc(NodeKind::Effect {
        run: Box::new(run), sources: EdgeSet::default(),
    }));
    run_effect(id);
    id
}

pub(crate) fn get_signal_value<T: Clone + 'static>(id: NodeId) -> T {
    track(id);
    with_runtime(|rt| match rt.get_kind(id) {
        Some(NodeKind::Signal { value, .. }) =>
            value.downcast_ref::<T>().cloned().expect("signal type mismatch"),
        _ => panic!("signal does not exist (disposed?)"),
    })
}

pub(crate) fn try_get_signal_value<T: Clone + 'static>(id: NodeId) -> Option<T> {
    track(id);
    with_runtime(|rt| match rt.get_kind(id) {
        Some(NodeKind::Signal { value, .. }) =>
            value.downcast_ref::<T>().cloned(),
        _ => None,
    })
}

pub(crate) fn get_memo_value<T: Clone + 'static>(id: NodeId) -> T {
    track(id);
    with_runtime(|rt| match rt.get_kind(id) {
        Some(NodeKind::Memo { value: Some(v), .. }) =>
            v.downcast_ref::<T>().cloned().expect("memo type mismatch"),
        _ => panic!("memo has no value yet"),
    })
}

pub(crate) fn set_signal_value<T: 'static>(id: NodeId, value: T) {
    with_runtime(|rt| {
        if let Some(NodeKind::Signal { value: slot, revision, .. }) = rt.get_kind_mut(id) { 
            *slot = Box::new(value); 
            *revision += 1;
        }
    });
    notify(id);
}

pub(crate) fn update_signal_value<T: 'static>(id: NodeId, f: impl FnOnce(&mut T)) {
    with_runtime(|rt| {
        if let Some(NodeKind::Signal { value, revision, .. }) = rt.get_kind_mut(id) {
            if let Some(v) = value.downcast_mut::<T>() {
                f(v);
                *revision += 1;
            }
        }
    });
    notify(id);
}

pub(crate) fn mutate_signal_vec<T: Clone + 'static>(id: NodeId, mutation: crate::ListMutation<T>) {
    with_runtime(|rt| {
        if let Some(NodeKind::Signal { value, mutations, revision, .. }) = rt.get_kind_mut(id) {
            if let Some(v) = value.downcast_mut::<Vec<T>>() {
                // Apply the mutation to the actual vector
                match &mutation {
                    crate::ListMutation::Push(item) => v.push(item.clone()),
                    crate::ListMutation::Insert(index, item) => v.insert(*index, item.clone()),
                    crate::ListMutation::Remove(index) => { v.remove(*index); },
                    crate::ListMutation::Clear => v.clear(),
                }
                
                // Record the mutation
                *revision += 1;
                let current_rev = *revision;
                
                // Downcast mutations to Vec<(usize, ListMutation<T>)>. 
                // If it fails (it was initialized to ()), replace it.
                if !mutations.is::<Vec<(usize, crate::ListMutation<T>)>>() {
                    *mutations = Box::new(Vec::<(usize, crate::ListMutation<T>)>::new());
                }
                
                if let Some(m_vec) = mutations.downcast_mut::<Vec<(usize, crate::ListMutation<T>)>>() {
                    m_vec.push((current_rev, mutation));
                }
            }
        }
    });
    notify(id);
}

pub(crate) fn get_signal_mutations<T: Clone + 'static>(id: NodeId, since_revision: usize) -> (usize, Vec<crate::ListMutation<T>>) {
    with_runtime(|rt| {
        if let Some(NodeKind::Signal { mutations, revision, .. }) = rt.get_kind(id) {
            if let Some(m_vec) = mutations.downcast_ref::<Vec<(usize, crate::ListMutation<T>)>>() {
                let recent: Vec<_> = m_vec.iter()
                    .filter(|(rev, _)| *rev > since_revision)
                    .map(|(_, muta)| muta.clone())
                    .collect();
                return (*revision, recent);
            }
            return (*revision, Vec::new());
        }
        (0, Vec::new())
    })
}

fn run_effect(id: NodeId) {
    let mut f: Box<dyn FnMut()> = with_runtime(|rt| {
        if let Some(NodeKind::Effect { sources, .. }) = rt.get_kind_mut(id) {
            let old = sources.drain();
            for src in old { rt.remove_subscriber(src, id); }
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
        if let Some(NodeKind::Effect { run, .. }) = rt.get_kind_mut(id) { *run = f; }
    });
}

fn run_memo(id: NodeId) -> bool {
    let mut compute: Box<dyn FnMut() -> Box<dyn Any>> = with_runtime(|rt| {
        if let Some(NodeKind::Memo { sources, .. }) = rt.get_kind_mut(id) {
            let old = sources.drain();
            for src in old { rt.remove_subscriber(src, id); }
        }
        match rt.get_kind_mut(id) {
            Some(NodeKind::Memo { compute, .. }) =>
                std::mem::replace(compute, Box::new(|| -> Box<dyn Any> { Box::new(()) })),
            _ => Box::new(|| -> Box<dyn Any> { Box::new(()) }),
        }
    });
    with_runtime(|rt| rt.observer_stack.push(id));
    let new_value = compute();
    with_runtime(|rt| { rt.observer_stack.pop(); });
    with_runtime(|rt| {
        if let Some(NodeKind::Memo { compute: c, eq, value, .. }) = rt.get_kind_mut(id) {
            *c = compute;
            let changed = match value { Some(old) => !eq(old.as_ref(), new_value.as_ref()), None => true };
            *value = Some(new_value);
            changed
        } else { false }
    })
}

// ── Topological batching ──────────────────────────────────────────────────────
//
// BFS from the changed node assigns each reachable node its maximum depth.
// Sorting by (is_effect, depth) ensures every memo runs before the effects
// that depend on it, and the effect runs exactly once even in diamond graphs.

fn get_subs(id: NodeId) -> Vec<NodeId> {
    with_runtime(|rt| match rt.get_kind(id) {
        Some(NodeKind::Signal { subscribers, .. }) => subscribers.iter().copied().collect(),
        Some(NodeKind::Memo   { subscribers, .. }) => subscribers.iter().copied().collect(),
        _ => Vec::new(),
    })
}

fn get_sources(id: NodeId) -> Vec<NodeId> {
    with_runtime(|rt| match rt.get_kind(id) {
        Some(NodeKind::Memo   { sources, .. }) => sources.iter().copied().collect(),
        Some(NodeKind::Effect { sources, .. }) => sources.iter().copied().collect(),
        _ => Vec::new(),
    })
}

fn notify(root: NodeId) {
    let mut depths: HashMap<NodeId, usize> = HashMap::new();
    let mut queue: std::collections::VecDeque<(NodeId, usize)> = std::collections::VecDeque::new();
    for sub in get_subs(root) { queue.push_back((sub, 1)); }
    while let Some((node, depth)) = queue.pop_front() {
        let entry = depths.entry(node).or_insert(0);
        if *entry < depth {
            *entry = depth;
            for sub in get_subs(node) { queue.push_back((sub, depth + 1)); }
        }
    }

    let mut order: Vec<(NodeId, usize, bool)> = depths.iter().map(|(&node, &depth)| {
        let is_effect = with_runtime(|rt| matches!(rt.get_kind(node), Some(NodeKind::Effect { .. })));
        (node, depth, is_effect)
    }).collect();
    order.sort_by_key(|&(_, depth, is_effect)| (is_effect as u8, depth));

    let mut changed: HashSet<NodeId> = [root].into();
    for (node, _, is_effect) in order {
        if !get_sources(node).iter().any(|s| changed.contains(s)) { continue; }
        if is_effect { run_effect(node); }
        else if run_memo(node) { changed.insert(node); }
    }
}

pub(crate) fn make_eq<T: PartialEq + 'static>() -> AnyEq {
    Box::new(|a: &dyn Any, b: &dyn Any| {
        match (a.downcast_ref::<T>(), b.downcast_ref::<T>()) {
            (Some(a), Some(b)) => a == b,
            _ => false,
        }
    })
}

pub(crate) fn push_scope() { with_runtime(|rt| rt.scope_stack.push(Vec::new())); }
pub(crate) fn pop_scope() -> Vec<NodeId> {
    with_runtime(|rt| rt.scope_stack.pop().unwrap_or_default())
}
pub(crate) fn dispose_node(id: NodeId) { with_runtime(|rt| rt.dispose(id)); }

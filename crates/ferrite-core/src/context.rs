//! A simple provide/inject context system for sharing state across components.
//!
//! Note: Context is currently **thread-local**, meaning it acts as global state
//! for the entire UI running on this thread, rather than being scoped to a specific
//! widget subtree. This matches the thread-local nature of the reactive runtime,
//! but means that multiple independent UI roots on the same thread will share
//! context. Use `reset_context()` between tests to avoid bleeding state.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    static CONTEXT: RefCell<HashMap<TypeId, Box<dyn Any>>> = RefCell::new(HashMap::new());
}

/// Provides a value of type `T` to the context.
pub fn provide<T: 'static>(value: T) {
    CONTEXT.with(|c| c.borrow_mut().insert(TypeId::of::<T>(), Box::new(value)));
}

/// Injects a value of type `T` from the context. Panics if no provider for `T` is found.
pub fn inject<T: Clone + 'static>() -> T {
    CONTEXT.with(|c| {
        c.borrow()
            .get(&TypeId::of::<T>())
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
            .expect("no provider found for this type — call provide::<T>() first")
    })
}

/// Clears the context map. Call this between tests to avoid bleeding state.
pub fn reset_context() {
    CONTEXT.with(|c| c.borrow_mut().clear());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provide_and_inject_round_trips() {
        reset_context();
        provide(42i32);
        assert_eq!(inject::<i32>(), 42);
    }

    #[test]
    fn provide_signal_inject_signal() {
        reset_context();
        use ferrite_reactive::create_signal;
        let s = create_signal(100);
        provide(s);
        let injected = inject::<ferrite_reactive::Signal<i32>>();
        assert_eq!(injected.get(), 100);
        s.set(200);
        assert_eq!(injected.get(), 200);
    }
}

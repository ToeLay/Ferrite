# Ferrite

A reactive GUI framework for Rust, built around one idea: `Copy` arena-backed
signals instead of `Rc<RefCell<T>>`, and a declarative view layer that maps
directly to a persistent widget tree. The reactive effects mutate this tree in
place instead of diffing against a fresh virtual DOM on every change. See
[`ARCHITECTURE.md`](./ARCHITECTURE.md) for the full reasoning.

## Quickstart

```rust
use ferrite::prelude::*;

fn main() {
    let count = use_state(0i32);

    ferrite::run(
        "Ferrite Counter",
        (300, 200),
        col([
            label(move || format!("Count: {}", count.get()))
                .size(32.0),
            row([
                button("-", move || count.update(|c| *c -= 1)),
                button("+", move || count.update(|c| *c += 1)),
            ])
            .gap(16.0)
            .justify(JustifyContent::Center),
        ])
        .padding(32.0)
        .gap(24.0)
        .align(AlignItems::Center),
    );
}
```

That's the entire `window_counter` example, word for word — see
`examples/window_counter/src/main.rs`.

## Running the examples

```sh
# A full showcase with tabs, sliders, and context injection
cargo run --bin counter

# A simple reactive counter
cargo run --bin window_counter

# A text input example
cargo run --bin text_input
```

## Crate map

| Crate                 | What it owns                                          |
|------------------------|--------------------------------------------------------|
| `ferrite`              | Facade + prelude — what an application depends on      |
| `ferrite-reactive`     | `Signal` / `Memo` / `create_effect`, no UI knowledge    |
| `ferrite-layout`       | `Style` → flexbox (via `taffy`) → resolved `Rect`       |
| `ferrite-core`         | Declarative view API, `Widget` trait, the widget tree  |
| `ferrite-render-skia`  | `DrawCommand` → pixels, via `tiny-skia` + `fontdue`     |
| `ferrite-window`       | Native window + event loop, via `winit` + `softbuffer`  |

Dependencies only point downward in that table. `ferrite-core` cannot import
a font library or a windowing crate even by accident — it's enforced by
`Cargo.toml`, not just convention.

## Status

v0.1. Real, tested, and demonstrably working. The framework features a zero-reconciliation
declarative UI layer powered by granular reactivity. While it is a solid foundation,
it is not production-ready (lacks proper text measurement in layout, topology batching
in the reactive graph, and complex input handling).

## License

MIT License. See `LICENSE`.

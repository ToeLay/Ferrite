# Ferrite

A reactive GUI framework for Rust, built around one idea: `Copy` arena-backed
signals instead of `Rc<RefCell<T>>`, and a widget tree that reactive effects
mutate in place instead of diffing against a fresh tree on every change. See
[`ARCHITECTURE.md`](./ARCHITECTURE.md) for the full reasoning — this README
is the quickstart.

Run `cargo run -p counter` after the font setup below and it'll write
`counter_initial.png` / `counter_after_clicks.png` into the repo root — a
quick way to confirm everything's wired up correctly without a display.

## Quickstart

```rust
use ferrite::prelude::*;

fn main() {
    let mut tree = LayoutTree::new();
    let count = create_signal(0i32);

    let label = text_dyn(&mut tree, move || format!("Count: {}", count.get()));
    let incr = button(&mut tree, "+", move || count.update(|c| *c += 1));

    let root = column(
        &mut tree,
        Style { width: Size::Px(300.0), height: Size::Px(200.0), ..Default::default() },
        vec![Box::new(label), Box::new(incr)],
    );

    let app = App::new(tree, Box::new(root));
    run_window(WindowConfig::default(), app);
}
```

That's the entire `window_counter` example, word for word — see
`examples/window_counter/src/main.rs` for the slightly longer version with
two buttons and styling.

## Font (one-time setup, do this before running anything)

`ferrite-render-skia` needs **IBM Plex Mono Regular** (SIL Open Font License
1.1) for its reference text rendering. It's not checked into the repo — grab
it once and drop it in place:

1. Download: https://raw.githubusercontent.com/google/fonts/main/ofl/ibmplexmono/IBMPlexMono-Regular.ttf
2. Save it to: `crates/ferrite-render-skia/assets/IBMPlexMono-Regular.ttf`

(The license text is already at `crates/ferrite-render-skia/assets/IBMPlexMono-OFL.txt`.)

Or from a terminal:

```sh
curl -o crates/ferrite-render-skia/assets/IBMPlexMono-Regular.ttf \
  https://raw.githubusercontent.com/google/fonts/main/ofl/ibmplexmono/IBMPlexMono-Regular.ttf
```

Once that file is in place, `cargo build` picks it up automatically via
`include_bytes!` — no other setup needed.

## Running the examples

```sh
# Headless: renders to PNG, no display needed. Good for CI/sandboxes.
cargo run -p counter

# Real OS window. Needs a display (or Xvfb).
cargo run -p window_counter
```

## Crate map

| Crate                 | What it owns                                          |
|------------------------|--------------------------------------------------------|
| `ferrite`              | Facade + prelude — what an application depends on      |
| `ferrite-reactive`     | `Signal` / `Memo` / `create_effect`, no UI knowledge    |
| `ferrite-layout`       | `Style` → flexbox (via `taffy`) → resolved `Rect`       |
| `ferrite-core`         | `Widget` trait, `App`, `DrawCommand`, the widget tree   |
| `ferrite-render-skia`  | `DrawCommand` → pixels, via `tiny-skia` + `fontdue`     |
| `ferrite-window`       | Native window + event loop, via `winit` + `softbuffer`  |

Dependencies only point downward in that table. `ferrite-core` cannot import
a font library or a windowing crate even by accident — it's enforced by
`Cargo.toml`, not just convention.

## Status

v0.1. Real, tested, and demonstrably working — every layer above has unit
tests or a runnable example backing its claims, and `ARCHITECTURE.md` lists
the honest gaps (no text measurement in layout yet, no topological batching
in the reactive graph, mouse-only input) rather than hiding them. Not
production-ready; a solid foundation to build the next pieces on top of.

## License

Dual-licensed under MIT or Apache-2.0, at your option. See `LICENSE-MIT` and
`LICENSE-APACHE`.

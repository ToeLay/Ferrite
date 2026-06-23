# Ferrite — architecture

This document explains *why* Ferrite is shaped the way it is, not just what
the code does. If you only read one section, read "The central bet" below —
everything else follows from it.

## The central bet

Every mature reactive UI framework (React, SolidJS, SwiftUI, Leptos) ends up
at the same place: a graph of state, a set of computations that depend on
that state, and a disciplined rule that data flows one way through that
graph. In garbage-collected languages, *getting there* takes real
engineering — that's what a virtual DOM diff, or a hook dependency array, is
for: a workaround for not having precise, scoped ownership of when something
is read and when it's safe to free.

Rust already has precise, scoped ownership. The bet behind Ferrite is that a
GUI framework built *with* that, instead of importing a pattern designed to
work around its absence in other languages, ends up simpler, not more
constrained. Two concrete decisions fall out of that bet:

1. **Signals are arena-indexed `Copy` handles, not `Rc<RefCell<T>>`.**
   See `crates/ferrite-reactive`. A `Signal<T>` is a 64-bit `(index,
   generation)` pair into a thread-local arena — copy it into ten closures,
   store it in a struct, no `.clone()` ceremony, no reference cycles to
   reason about. Disposal is explicit and bulk via `Scope`, matching
   component mount/unmount, instead of relying on the last `Rc` in a
   scattered graph happening to drop.

2. **The widget tree is mutated in place by effects, not diffed against a
   fresh tree on every state change.** See `crates/ferrite-core`. A
   component function runs *once* to build a persistent tree of widgets.
   Reactive bindings inside it (a `text_dyn`, a button's `on_click`) wire an
   effect directly to that one node. When a signal changes, the effect
   re-runs and mutates that node's content — no vdom, no reconciliation
   algorithm, no key-based diffing bugs. This is the same fine-grained
   approach SolidJS/Leptos use for the DOM, applied to a custom retained
   widget tree instead.

Neither decision is novel in isolation (Leptos already does `Copy` signals;
fine-grained-over-vdom is SolidJS's whole pitch). What's specifically *for
Rust* here is using `Copy` + an arena to get ergonomics other languages reach
for `Rc<RefCell<>>` to get, without the cycle/lifetime headaches that come
with it.

## Layer map

```
ferrite (facade + prelude)
   |
   +-- ferrite-window     winit (events/windowing) + softbuffer (present)
   +-- ferrite-render-skia  DrawCommand -> pixels, tiny-skia + fontdue
   +-- ferrite-core       Widget trait, App, DrawCommand, dirty flag
   |      +-- ferrite-layout   Style -> taffy -> resolved Rect
   |      +-- ferrite-reactive Signal/Memo/Effect (no UI knowledge at all)
```

The dependency arrows only point one way. `ferrite-reactive` has never heard
of a widget. `ferrite-core` has never heard of a pixel, a font, or a window.
That's enforced by crate boundaries, not just convention — `ferrite-core`'s
`Cargo.toml` literally cannot import `tiny-skia` or `winit` by accident.

## Why each layer looks the way it does

**`ferrite-reactive`** — covered above. The one thing worth adding: dependency
edges are rebuilt on *every* run of an effect/memo, so a signal read inside a
branch that stops being taken stops being depended on. See
`dynamic_dependencies_drop_stale_edges` in the test suite for the case this
guards against. Edge storage is a small insertion-ordered `Vec`, deliberately
not a `HashSet` — `HashSet`'s random iteration order would make propagation
order (and therefore effect run order on a diamond dependency) nondeterministic
between runs of the *same* program, which is a nasty class of bug to chase.

**`ferrite-layout`** is a thin translation layer over `taffy`, not a
reimplementation of flexbox. Flexbox is a solved, gnarly problem (see CSS's
own spec history); the value Ferrite adds here is a `Style` expressed in
plain logical pixels and `enum`s instead of taffy's CSS-shaped types, and a
default of `Direction::Column` rather than CSS's `Row` — because app UI is
overwhelmingly "stack of things," not inline text flow.

**`ferrite-core`** is the one place layout, reactivity, and drawing meet, and
it's built to minimize how much a widget author has to write. `Widget` is a
trait with five default methods and effectively two required ones
(`node_id`, plus `paint_self` or `children`/`children_mut` depending on
whether it's a leaf or a container) — the tree-walking logic for painting and
click hit-testing is written once, in the trait's default methods, not once
per widget.

**`ferrite-render-skia`** exists for three reasons, not as a placeholder:
- It's a genuinely useful headless backend (screenshot tests, server-side
  rendering of a panel) independent of any future GPU backend.
- It's a much smaller surface to get *correct* before windowing/event
  plumbing has to be debugged against it — see `ferrite-window`'s own
  development, where a real bug (see below) was caught by running an actual
  binary, something a GPU backend would have made far slower to iterate on.
- It proves the `DrawCommand` boundary is real. Nothing in this crate reaches
  back into widgets, layout, or signals — only `&[DrawCommand]` in, pixels
  out. A `ferrite-render-wgpu` crate is a drop-in replacement at that
  boundary, not a rewrite of anything upstream.

**`ferrite-window`** intentionally does *not* use wgpu. `winit` handles
windowing and input; `softbuffer` blits a CPU-rendered buffer straight to the
window's native surface. This was a deliberate choice to keep the
window/event code small enough to actually finish and verify (rather than
sketch) before reaching for a shader pipeline — and it was the right call:
running the very first build under a virtual display caught a real bug
(`softbuffer` requires `resize()` before the first `present()`, and a
`Resized` event isn't guaranteed to arrive before the first paint) that would
have been a much slower loop to find and fix with a wgpu pipeline in the way.

## Honest limitations (v0.1)

These aren't hidden anywhere else in the codebase — flagged here and at the
relevant call site:

- **No text measurement in layout.** `taffy` supports a measure-function hook
  for exactly this; Ferrite doesn't wire one up yet. `Text` nodes estimate
  their box from character count against the bundled font's known monospace
  advance width (`crates/ferrite-core/src/widgets.rs`), which is exact for
  that font and wrong the moment a proportional font or wrapped text shows
  up. Real fix: a measure function that calls into whatever the active
  render backend's font shaper reports.
- **No topological batching in the reactive graph.** A "diamond" dependency
  (one signal feeding two memos that both feed one effect) re-runs that
  effect once per upstream memo that changed, not once — see
  `diamond_dependency_runs_effect_once` in `ferrite-reactive`'s tests, which
  documents and pins down the current (correct-but-redundant) behavior rather
  than hiding it. The fix is a standard topological-sort propagation pass;
  it's the first thing to build past v0.1.
- **One window, one thread.** The reactive runtime is intentionally
  thread-local (`!Send`), matching how most native retained-mode UI
  toolkits work (GTK, single-threaded JS DOM). Multi-window support is a
  question of whether each window gets its own runtime instance or they
  share one with care taken around the `Scope` boundaries — not yet decided.
- **Hit testing has no clipping.** A child can currently be "clicked" outside
  its parent's visible bounds if the parent doesn't itself clip. Doesn't bite
  in practice yet because nothing in v0.1 scrolls or overflows.

## What to build next, in order

1. Topological batching in `ferrite-reactive` (fixes the diamond-dependency redundancy).
2. A `taffy` measure-function hook fed by the active render backend's font
   metrics, replacing the character-count estimate.
3. Keyboard input and focus (currently mouse-only).
4. A `ferrite-render-wgpu` backend behind the same `DrawCommand` boundary —
   purely a performance/feature upgrade at that point, not an architecture change.

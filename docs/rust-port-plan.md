# Rust Port Plan

This document captures the Rust migration direction and the initial bootstrap implementation that now lives under `rust/`.

## Goals

1. Replace the C app/runtime/widgets/pages with Rust.
2. Keep SDL3 as the graphics/input library using official Rust bindings (`sdl3` crate).
3. Maximize readability and maintainability by using Rust ownership, type safety, enums, and trait-based page contracts.
4. Keep the existing C code available as reference during migration, then remove it once Rust reaches feature parity.

## Current C Friction To Address

1. Event/focus/capture routing is duplicated across `ui_runtime` and `ui_window`.
2. Ownership/lifecycle is manually coordinated (`ui_runtime_add/remove`, widget destroy ops, page shell registration bookkeeping).
3. Page polymorphism uses `void *` page instances.
4. Page discovery requires CMake code generation.
5. `todo_page` complexity is elevated by callback context/index management.
6. Page shell uses a fixed registration capacity.

## Target Rust Architecture

### App Shell

- `App` owns:
  - SDL context
  - window + renderer
  - event loop timing
  - one runtime/page manager
- CLI supports startup page selection and viewport dimensions.

### Runtime/Tree Direction

- Move to one canonical event/update/render path.
- Represent the UI tree with typed node IDs (arena style) instead of raw pointers.
- Use enum-dispatched widget behavior (`WidgetKind`) rather than C ops tables.

### Page Layer

- Typed page API (trait-based), no `void *`.
- Explicit page IDs and registry in Rust code.
- Action-based communication from widgets to page state.

### Error Handling

- Use `Result` for recoverable startup/runtime surface errors.
- Panic/fail-fast only for invariant violations.

## First Bootstrap (Implemented)

Location: `rust/cui_app`

Implemented now:

1. Rust SDL3 application loop.
2. CLI option parsing:
   - `--page <id>`
   - `--width`, `--height`
   - `--help`
3. Startup page selection with typed page IDs.
4. Runtime page switching:
   - `1` = `todo`
   - `2` = `corners`
   - `3` = `showcase`
   - `Tab` = cycle pages
5. Resize handling and logical-size updates.
6. Simple placeholder visuals for each page using renderer primitives and `draw_debug_text`.

This is intentionally a thin prototype: it validates the Rust app shell and page switching model before porting the full TODO behavior and reusable widget/runtime system.

## Planned Migration Sequence

1. Establish Rust app shell + page registry (done).
2. Port core runtime routing and ownership model.
3. Port layout primitives (`layout_container`, `scroll_view`, anchoring behavior).
4. Port foundational widgets (`pane`, `text`, `button`, `checkbox`, `input`, etc.).
5. Port `showcase` with broad widget parity.
6. Port full `todo` interactions/model.
7. Port `corners` parity/resize behavior validation.
8. Replace C build/run path with Rust once parity is complete.

## Module Mapping (C -> Rust Direction)

- `main.c` -> `rust/cui_app/src/app.rs` + `rust/cui_app/src/main.rs`
- `include/system`, `src/system` -> future `rust/cui_app/src/runtime/`
- `include/ui`, `src/ui` -> future `rust/cui_app/src/ui/`
- `include/pages`, `src/pages` -> `rust/cui_app/src/pages/`
- `src/util/fail_fast.c` -> future typed error/invariant helpers in Rust

## Validation Notes

- UI bootstrap visual check captured with `scripts/capture_app_window.sh` to `/tmp`.
- This capture is ephemeral and not committed under `assets/`.

# Rust Implementation Plan

The repository migration is complete: the Rust workspace is now the primary and only implementation.

## Current Layout

- Workspace manifest: `/Cargo.toml`
- App crate: `/cui_app`
- App entry points:
  - `cui_app/src/main.rs`
  - `cui_app/src/app.rs`
- Page modules:
  - `cui_app/src/pages/mod.rs`
  - `cui_app/src/pages/test.rs`
  - `cui_app/src/pages/layout.rs`

## Completed Migration Outcomes

1. Removed C source tree and CMake build path.
2. Promoted Rust workspace from `/rust` to repository root.
3. Updated Makefile, CI, hooks, and editor tasks to Rust-only workflows.
4. Removed vendored SDL submodules from the repository.

## Near-Term Technical Priorities

1. Expand typed page registry beyond `test` page.
2. Continue layout engine correctness work (fit/fill/grow interactions).
3. Add runtime/widget abstractions as reusable modules under `cui_app/src/`.
4. Add broader automated tests for layout behavior and page rendering scenarios.

## Validation Baseline

- `make precommit` should pass locally.
- `cargo run -p cui_app -- --page test` should launch and render the stack layout test page.
- UI snapshots can be captured with `scripts/capture_app_window.sh` into `/tmp`.

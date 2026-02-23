# Rust Layout Engine Evolution Plan

This document captures the current plan for improving the Rust layout engine (`rust/cui_app/src/pages/layout.rs`) after the recent `grow`/`fit-content` experiments in `test.rs`.

The goal is to improve correctness and predictability across all pages, not to optimize only for the test page.

## Context

The current Rust prototype supports:

- `SizeMode::FitContent`
- `SizeMode::FillParent`
- `SizeMode::Fixed(px)`
- `SizeMode::Grow(weight)`

This is enough to demonstrate useful layouts, but mixed cases (especially `FitContent` + `Grow` + `FillParent` in nested stacks) expose a modeling gap:

- `Grow` and `FillParent` are currently used partly as measurement inputs and partly as arrangement rules.
- In a robust layout engine, these should be treated primarily as allocation behaviors, while intrinsic measurement should be based on content/base size.

## Problems We Have Observed

### 1. `FitContent` parent + `Grow` child negotiation

A child that says "grow as big as possible" cannot provide a single intrinsic width/height by itself. It needs a bound from the parent chain.

In this app, a bound usually exists (the viewport/window), but the engine still needs a principled rule for how `FitContent` interacts with `Grow`.

### 2. `FillParent` inflating intrinsic measurement

`FillParent` is not an intrinsic size. If intrinsic measurement treats it as "use all available size", parent stacks can over-measure children and overflow.

### 3. API ergonomics for mixed sizing

Calling `fixed_size(...)` and then overriding one axis to `Grow(...)` works, but is awkward and asks the wrong question when the user intent is "grow on this axis".

## Design Principles (Recommended)

1. Separate intrinsic measurement from final allocation.
2. Treat `Grow` and `FillParent` as arrangement/allocation behaviors.
3. Use child base size (intrinsic/min/preferred) for intrinsic measurement.
4. Distribute only extra space during final arrangement.
5. Be explicit about whether a parent is providing a definite bound.

## Complexity Options

### Option A: Small Patch (Low Complexity, Low Risk)

Scope:

- Patch current stack measurement logic to reduce obvious errors (for example, stop `FillParent` from inflating intrinsic measurement).
- Keep public API and overall traversal model mostly unchanged.

Pros:

- Fast to land
- Minimal disruption
- Fixes immediate bugs

Cons:

- Leaves the engine ad hoc
- More edge cases likely as layouts get richer
- Harder to reason about nested combinations over time

When to choose:

- If we need a quick stabilization pass before other work

### Option B: Internal Refactor (Medium Complexity, Recommended Near-Term)

Scope:

- Keep existing public API mostly intact (`SizeMode`, `VStack`, `HStack`, `ColorBlock`)
- Refactor internals to explicitly separate:
  - base/intrinsic measurement
  - flex/grow eligibility
  - final space allocation

Pros:

- Generalizes across pages
- Preserves momentum (no broad rewrite)
- Much cleaner mental model than incremental patches

Cons:

- Moderate refactor inside `layout.rs`
- Requires careful retesting of existing test cases

When to choose:

- Now, if we want correctness and algorithmic cleanliness without redesigning the whole API surface

### Option C: Formal Constraint-Based Layout Contract (Large Complexity, Cleanest Long-Term)

Scope:

- Introduce a more explicit internal layout protocol such as:
  - `measure(constraints)`
  - `arrange(rect)`
- Add constraint semantics (for each axis):
  - `Definite(px)`
  - `AtMost(px)`
  - optional `Unbounded`
- Potentially evolve `LayoutNode` and widget measurement contracts

Pros:

- Most principled / industry-aligned
- Best foundation for future containers/widgets
- Handles nested fit/fill/flex composition more cleanly

Cons:

- Larger rewrite
- Higher regression risk
- More changes to code shape and tests

When to choose:

- When the Rust layout engine is becoming central and we need stronger long-term guarantees

## Recommended Path

Take **Option B** now, while the Rust engine is still small, and preserve a path toward Option C later.

This gives us most of the algorithmic cleanliness we need without stalling development.

## Recommended Engine Semantics (Universal Rules)

These rules should apply across all pages and layout trees:

1. `Grow` and `FillParent` are allocation behaviors, not intrinsic sizes.
2. During intrinsic measurement, coerce `Grow`/`FillParent` on the measured axis to a non-expanding fallback (`FitContent`/base size policy).
3. Measure child base contributions first.
4. Parent computes intrinsic size from child base contributions plus spacing/padding.
5. During arrange, parent distributes only extra space to grow-capable children by weight.
6. Any policy that promotes `FitContent` to fill due to grow children should only apply when the parent has a definite bound on that axis.

## Clarifying the "FitContent + Grow" Policy

There are two valid policies:

### Policy 1: Strict `FitContent`

- `FitContent` always shrink-wraps intrinsic/base child size.
- `Grow` children only expand if the parent is explicitly larger (for example, parent is `FillParent`).

Pros:

- More predictable and compositional
- Matches many layout systems

Cons:

- Less ergonomic for the prototype if the desired behavior is "grow child implies fill"

### Policy 2: Pragmatic Promotion (Current Direction)

- If a stack is `FitContent` on its main axis and contains a main-axis `Grow` child, the stack may promote to the available size on that axis (effectively fill), provided that size is definite.

Pros:

- Matches the intended behavior discussed during `test.rs` exploration
- Ergonomic for stack-based layouts in this prototype

Cons:

- `FitContent` becomes context-sensitive
- Must be documented clearly to avoid surprises

Recommendation:

- Keep Policy 2 for the Rust prototype if it continues to match expected usage, but implement it on top of the stronger intrinsic-vs-allocation separation so it remains principled.

## API Direction (Ergonomics)

The API should let callers express per-axis behavior directly without fake fixed values.

Already added:

- `ColorBlock::size(width_mode, height_mode)`

Recommended additions (small ergonomic improvements):

- `ColorBlock::fixed_width(px)`
- `ColorBlock::fixed_height(px)`
- `ColorBlock::grow_width(weight)`
- `ColorBlock::grow_height(weight)`
- optional intrinsic/base setters if needed:
  - `ColorBlock::intrinsic_width(px)`
  - `ColorBlock::intrinsic_height(px)`

This keeps "layout behavior" separate from "intrinsic/base content size."

## Suggested Implementation Steps (Near-Term)

1. Refactor stack internals to use explicit "base measurement" for all children.
2. Make intrinsic measurement coerce both `Grow` and `FillParent` on the measured axis to non-expanding fallback behavior.
3. Keep final arrange responsible for flex and fill allocation only.
4. Add targeted `test.rs` rows for:
   - `FitContent` + fixed-only
   - `FitContent` + one grow child
   - `FitContent` + two grow children
   - `FillParent` + grow children
   - nested stacks mixing `FillParent` cross-axis + `Grow` main-axis
5. Document the chosen `FitContent` promotion policy in `README.md` once stabilized.

## Success Criteria

We should consider the refactor successful when:

- Existing `test.rs` scenarios remain visually correct
- No row/container overflows due to intrinsic measurement treating `FillParent` as intrinsic size
- Nested `VStack`/`HStack` combinations behave predictably
- The layout code is easier to explain in terms of "measure base size, then allocate extra space"


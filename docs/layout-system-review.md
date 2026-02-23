# Layout System Review

Review of the measure/arrange layout system on the `refactor/todo-window-root-layout` branch.

## Overall Verdict

The design is sound and well-motivated. The two-pass measure/arrange model is the industry-standard approach (WPF, Flutter, SwiftUI all use it), and the C implementation is clean and readable. The virtual dispatch through `ui_element_ops` gives proper polymorphism, and the opt-in design (NULL measure = fixed-size, NULL arrange = copy rect) is a smart default for leaf widgets.

There are a few spots where the abstraction is inconsistent with itself, which undercuts the goal of "one code flow path to master."

## What Works Well

### 1. The ops table contract is crisp

The measure/arrange split in `ui_element_ops` with well-defined defaults (`src/ui/ui_element.c:5-34`) is exactly right. Leaf widgets do nothing; containers opt in. Easy to learn.

### 2. `ui_layout_container` is the strongest piece

It has clean, proper two-pass behavior:

- `measure_vertical_children` iterates all children, calls their measure, accumulates total height.
- `arrange_vertical_children` positions them all with finalized rects.
- The container genuinely separates sizing from placement.

### 3. Parent-chain screen rect resolution

`compute_screen_rect` (`src/ui/ui_element.c:102-156`) is elegant. Alignment anchors (LEFT/CENTER/RIGHT, TOP/CENTER/BOTTOM) compose naturally through the recursive walk. This lets you anchor an FPS counter to the bottom-right without any special layout code.

### 4. The page-level layout struct

`todo_page_layout` is a nice pattern: compute all geometry into an immutable snapshot, then consume it in arrange. Clean data flow.

## Inconsistencies

### 1. `run_window_layout_pass` runs twice per layout cycle

`measure_window` calls `run_window_layout_pass` (line 206), and `arrange_window` also calls it (line 222). When `app_page_shell_arrange_root` triggers `app_page_shell_measure_and_arrange_element` on the window root, both measure and arrange fire, so the child iteration happens twice with identical results. This is harmless but wasteful, and more importantly confusing: a reader would expect measure and arrange to do different things.

### 2. The measure+arrange helper collapses the two passes into one

`app_page_shell_measure_and_arrange_element` (`src/pages/page_shell.c:94-111`) calls measure then immediately arrange on a single element. The todo page calls this helper on each top-level element sequentially. So in practice, the page-level layout is **not** "measure everything, then arrange everything" -- it's "measure-and-arrange element A, then measure-and-arrange element B, then C..." This works because the top-level elements are independently positioned, but it means the two-pass architecture doesn't actually buy you anything at the page level. It's a single-pass layout with two function calls per element.

### 3. `measure_page_layout` is arithmetic, not tree measurement

In the standard model, "measure" means asking the element subtree "how big do you want to be?" `measure_page_layout` (`src/pages/todo_page.c:713-740`) instead computes viewport-derived constants with plain math -- no element's `measure` op is ever called there. The actual element measurement happens inside `app_page_shell_measure_and_arrange_element` during what's conceptually the "arrange" phase. The naming suggests a two-phase pipeline, but the phases don't map to the element-level measure/arrange split.

### 4. Two mental models coexist

A reader needs to understand two distinct layout approaches:

- **Top-level elements**: manually positioned by page code using viewport-derived absolute coordinates (the `arrange_header_section`, `arrange_input_section`, etc. functions).
- **Container-managed elements**: automatically positioned by `ui_layout_container` via the recursive measure/arrange cascade.

The scroll view + layout container subtree is genuinely tree-walked. But the header, input row, stats, and footer are each independently positioned with hand-computed rects. This means "one code flow path" applies within containers but not at the page level.

## Suggestions

The cleanest path toward a single mental model would be to make the page-level layout also tree-driven. If all top-level sections (header, input, stats, list, footer) were children of the `ui_window` root and the window performed a real vertical stack layout (or the page used a top-level `ui_layout_container` for its sections), then the entire page would be laid out through the same recursive measure/arrange cascade. The page's `resize` would just update the root's rect and trigger a single `measure` + `arrange` on the root, which cascades down. No hand-positioned rects at the page level.

For a minimalist codebase where pages have bespoke layouts, the current hybrid is pragmatic and not hard to follow. The inconsistency is more of a "two paths" situation than a "broken" situation. Whether it's worth unifying depends on how many pages are expected and how consistent the learning curve should be.

## Summary

| Aspect | Assessment |
|---|---|
| Core algorithm (measure/arrange ops) | Clean, correct, industry-standard |
| `ui_layout_container` | Properly two-pass, elegant |
| `ui_scroll_view` | Clean delegation, correct clipping |
| `compute_screen_rect` alignment | Elegant, composable |
| Window layout pass | Runs twice unnecessarily |
| Page-level layout | Single-pass disguised as two-pass |
| `measure_page_layout` naming | Misleading -- geometry computation, not tree measurement |
| Unified mental model | Close but not quite -- two layout styles coexist |

The bones are good. The container/scroll-view tree-walking is the part that genuinely works as a two-pass system and is easy to understand. The page-level manual positioning is the part that breaks the "one path" aspiration.

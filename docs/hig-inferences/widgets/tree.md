# Tree

No libadwaita widget. After Mantine's [Tree](https://mantine.dev/core/tree/); the model GNOME uses is GTK's [`GtkTreeExpander`](https://docs.gtk.org/gtk4/class.TreeExpander.html) inside a `GtkListView` over a `GtkTreeListModel`.

## What the HIG says

Nothing specific: the HIG has **no tree-view pattern**. What it does say bounds when to use one:

- [Boxed lists](https://developer.gnome.org/hig/patterns/containers/boxed-lists.html): "appropriate for relatively small static lists"; "Expander rows expand to reveal additional rows below." For a **small, static** hierarchy the HIG's answer is an [ExpanderRow](expander-row.md) in a boxed list, not a tree. Larger or dynamic data belongs in a list view.
- [Sidebars](https://developer.gnome.org/hig/patterns/nav/sidebars.html): "a vertical panel which contains a list of different locations": one list, ordered by usefulness; the HIG sets no rule for nesting, so a sidebar should stay flat unless the app has a real hierarchy.

Use `Tree` for large, deep or dynamic hierarchies (files, outlines, data), not for a settings page. For a **hand-written** hierarchy of rows within a page, such as an "Advanced" group of preferences, use [TreeRow](tree-row.md); `Tree` is the data-driven one.

## What GTK says (followed here)

- **Rows indent by depth** (`indent-for-depth`); a **leaf keeps the width of the arrow** (`indent-for-icon`), so its label lines up with the labels of siblings that have one.
- The **arrow** is a `:checked`-state icon: it points right when closed and down when open. The widget is a button for accessibility (GTK 4.12).
- **Keys:** `+` or `*` expands; `-` or `/` collapses; Ctrl+Space toggles; Shift+Left / Right (swapped in right-to-left locales) expand and collapse. `primitives::tree::navigate` answers these and the arrow keys (Right opens, then enters; Left closes, then goes to the parent; Home, End).
- Hide the arrow on leaves; do not draw an arrow that does nothing.

## What Mantine has (all of it is here)

| Mantine | here |
| --- | --- |
| `data` (`value`, `label`, `children`) | `TreeNode` |
| `useTree`: expand, collapse, select, check | `TreeState` (`expand`, `expand_all`, `select`, `check`, `apply`, …) |
| `withLines`, `levelOffset` | `.lines(true)`, `.level_offset(px)` |
| `renderNode` | `.label(\|payload\| ..)`, `.icons(..)` |
| Checked state, indeterminate, `checkStrictly` | `.checkboxes(true)`, `TreeState::is_indeterminate`, `check_strictly` |
| `expandOnClick`, `selectOnClick` | `expand_on_click`; a click selects |
| Initial expanded state | `TreeState::set_expanded`, `reveal` |
| Async loading (`hasChildren`, `onLoadChildren`) | `TreeNode::lazy`, `Effect::Load`, `finish_load`; a spinner, the error and Retry |
| Search: highlight, filter, fuzzy | `tree::matching_ancestors`, `filter`, `filter_with(.., &fuzzy)`; `.highlight(..)`, `.fuzzy(true)` |
| Drag and drop, `allowDrop`, drag handle | `.draggable(DragMode::Row / Handle)`, `.allow_drop(..)`, `Event::Drop`, `move_node` |
| Virtualization (`flattenTreeData`) | `tree::visible` / `TreeState::rows` (the flat rows), `tree::window`, `.window(range)` |
| Examples: files tree, docs editor | the gallery's Tree pages |

## Adaptations

- **Selection** is the neutral flat-button highlight of libadwaita's sidebars, not an accent fill.
- **Drop** reuses the reorderable list's look: the accent line before or after a row (from its own indent), the row tinted and ringed for "inside", the held row lifted and opaque, its old place dimmed, and the not-allowed cursor over a refused target.
- **Search is external**, as in Mantine: put a search entry above the tree and filter the data.
- **Checkboxes** are drawn (rounded, accent when checked, a dash when indeterminate) since iced's checkbox has no indeterminate state.

## iced mapping

`primitives::tree` is pure and unit tested (data, expansion, selection with ranges, checking, search, async loading, moves and drop rules, windowing, keys); `widget::Tree` builds the rows from iced buttons and draws the drag in a private rows widget. The widget speaks one message, `TreeEvent`; the app applies it with `TreeState::apply(&mut data, event)`, which returns `Some(TreeEffect::Load(value))` when a branch has to be fetched.

**Virtualization.** No established virtualizing widget exists for iced (0.14 has `scrollable` with `on_scroll`, `sensor` and `lazy`, but no virtual list; the crates that do are young, grid-specific or headless engines), and libcosmic has none either: its `segmented_button` model is a flat ordered list with a per-item `indent`, which is the same flat-rows-plus-depth shape as `tree::visible`. So rows have a **fixed height** and `tree::window` picks the range for the scroll offset; the widget builds those rows and spaces the rest. Variable row heights would need a measuring engine such as the headless `virtualizer` crate.

Not included: keyboard navigation (iced has no focus model; the primitive has it), opening a branch by hovering a drag over it (GTK does), clearing the selection on an outside click, and a per-row hover menu.

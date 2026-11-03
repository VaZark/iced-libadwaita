# Autocomplete

libadwaita and GTK 4 have no autocomplete widget: `GtkEntryCompletion` is deprecated and completes from a synchronous `GtkTreeModel`; `GtkSearchEntry` only reports that the text changed (with a delay), and `AdwComboRow` can filter a model it already has. `Autocomplete` is the entry-with-a-suggestions-popover pattern of Mantine `Autocomplete`, Radix-based comboboxes and the HTML `combobox` role, for suggestions that arrive **asynchronously** from a database or HTTP call.

- The popover sits under the entry, as wide as it, aligned to its start edge (it flips above when there is no room). It opens when the user types or clicks the entry and there is something to show; a pick, Escape or a click elsewhere closes it.
- It shows what the lookup's status says: nothing while idle; "Searching…" with a spinner while the first results are pending; the previous results while the next ones load (the entry shows a spinner); the suggestions or "No results"; an error with a Retry button.
- Up / Down move the highlight (wrapping), Enter picks it, Escape closes; hovering a row highlights it and a click picks it. With no highlight, Enter does nothing here: the text belongs to the app.
- **Free text** (the default) accepts any text. With `.free_text(false)` the text must be a picked suggestion's label; the entry draws its error ring once `revealed`, and `primitives::lookup::validate_pick` gives the same `Validity` for a `Field` message or a form.

## iced mapping

`widget::Autocomplete` is a thin applier of `primitives::lookup::Lookup` (debounce, `min_chars`, stale-response discarding, cache, retry; pure and unit tested). The widget is **controlled**: the app owns the text and the `Lookup`; the widget reports `on_input(text)`, `on_select(item)`, `on_retry`. Running the request is the app's, through `crate::lookup::run` (an `iced::Task`) with any `Fn(Q) -> impl Future<Output = Result<Vec<T>, E>>`, so the library depends on no database or HTTP client and no executor.

The entry is an `Entry` with a right section that is a spinner while loading, a clear button when there is text, and an empty slot otherwise (so the text input keeps its place in the widget tree and its focus); `.search_icon(true)` adds a left section. The popover is its own `Overlay` (not `Popover`: `Popover` owns an open flag toggled by clicking its trigger, which a combobox cannot use) placed with `primitives::anchor::place`; its rows are laid out by the popover, with the highlight drawn behind them, and `Highlight` (also in `primitives::lookup`) holds the cursor and the window of at most `max_rows` rows.

Deviations:

- The popover reads Up / Down / Enter / Escape while it is open, whatever has focus. Reopening a closed popover with Down works only after the entry was clicked (iced's text input does not tell its parent whether it has focus). Typing reopens it.
- No scrollbar: at most `max_rows` (8) rows show and the highlight scrolls them; the mouse wheel does not scroll the list.
- No opening animation, and no exit animation (the app unmounts it by changing the lookup state).
- The popover does not follow the entry while the page scrolls under it (like `Popover`).
- Suggestions are plain rows with a highlight, not menu items: no check mark and no disabled rows.

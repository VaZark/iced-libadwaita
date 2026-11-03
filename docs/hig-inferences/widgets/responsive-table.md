# ResponsiveTable

No libadwaita widget; a table while the columns fit, a stack of cards when they would not (see `src/widget/responsive_table.rs` for sorting and the footer).

## Virtualization and async rows

Two shared primitives make a table of thousands of rows practical; the [tree](tree.md) uses the same ones.

- **Virtualization** ([`primitives::virtualize`](../../lib/primitives.md)). `virtualized(total, first, row_height)` says the rows you pass are `first..` of `total` rows of a fixed height; the table builds only those and pads the scrolled area above and below with exact spacers, so the scrollbar is true to the whole list. Keep a `Viewport`, update it from `on_scroll`, and pass `viewport.window(total, ResponsiveTable::pitch(row_height), overscan)`. Each row has a 1 px hairline, which `pitch` includes. A virtualized table is **always a table**: cards have no fixed height, so they cannot be windowed.
- **Async rows** ([`primitives::loading::Chunks`](../../lib/primitives.md)). A list fetched `size` rows at a time: `request(window)` returns the chunks the window needs that nobody has asked for (each once), `finish(chunk, result)` reports a fetch, and `slot(row)` says whether a row is ready, loading (draw a placeholder) or failed (draw an error row with Retry, which calls `retry(chunk)`). A chunk that failed is **not asked for again** until retried, so scrolling does not hammer a failing server.

The gallery's *Data > Table: virtualization* (100,000 rows) and *Table: async chunks* (5,000 rows, every seventh chunk failing once) show both.

## No established library

No established virtualizing widget exists for iced: 0.14 has `scrollable` (with `on_scroll`), `sensor` and `lazy`, but no virtual list, and the crates that do are young, grid-specific or headless engines. libcosmic has none either; its `segmented_button` model is a flat ordered list with a per-item `indent`. Rows therefore have a fixed height here. Variable heights need a measuring engine such as the headless `virtualizer` crate, which this does not do.

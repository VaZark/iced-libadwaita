# Pagination

libadwaita has no pager (GNOME apps scroll or load more). `Pagination` follows Mantine's [`Pagination`](https://mantine.dev/core/pagination/) and `usePagination`, and Bootstrap's pagination: previous / next and numbered page buttons with ellipses.

- **Controlled.** The app owns a `primitives::pagination::Pagination` (page, page size, total) and updates it from `on_change(page)` (1-based).
- **The window of pages** is Mantine's: `siblings` pages either side of the current one, `boundaries` pages at each end, `...` between runs that are not adjacent (never replacing a single page).
- **Edges.** `with_edges(true)` adds first / last; previous / next (and first / last) are disabled at the ends.
- **Equal buttons.** Every button is the same square, wide enough for the largest number shown, so the pager does not jitter as the page changes; the pager is centered in its row (`align` to change it). Sizes Small / Medium / Large; `disabled(true)` dims everything.
- **Helpers.** `pagination_label(&state)` is "Showing 21–40 of 187"; `page_size_dropdown(&sizes, current, on_select)` picks the page size. Compose them in a footer (the gallery's Data page).
- **Size change** keeps the first visible item on screen; a shrinking total clamps the page (`primitives::pagination`).

## iced mapping

`widget::Pagination` over `primitives::pagination` (pure, unit tested: the page window, clamping, ranges). Icons are the crate's own (`GoFirst`, `GoLast`, `GoPrevious`, `GoNext`); svg tints are solid mixes.

Deviations from Mantine: no `getItemProps` / `getControlProps`, no `radius` or color tokens (three sizes). The pager does not shrink its `siblings` on a narrow container by itself: it needs about 400 px for the default window. Put it in a [`MetaComponent`](../../lib/primitives.md) with a smaller `siblings` below a breakpoint if it must fit a phone.

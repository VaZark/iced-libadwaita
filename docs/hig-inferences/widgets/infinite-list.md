# Infinite list and Load more

GNOME apps either scroll a long list or offer a "Load more" row. `InfiniteList` is both, after Mantine's infinite-scroll patterns (`use-intersection` with a sentinel, or a "Load more" button) and Bootstrap's "Load more" buttons.

- **Controlled.** The app owns a `primitives::infinite::Infinite<Query, Item>`; the widget draws the rows it is given plus a **footer** for the state: loading, an error with **Retry**, "No more results" at the end, an empty message when there is nothing.
- **Triggers.** `LoadTrigger::Scroll` (default) sends `on_load_more` when the viewport comes within `threshold` px (default 200) of the end; `LoadTrigger::LoadMore` shows a button instead (the accessible alternative). It only fires when the primitive can load: never while a request is in flight, after a failure, or when exhausted.
- **Stale answers.** Each request has an id; `Infinite::reset(query)` (a new sort order or filter) clears the items and forgets the request in flight, so its late answer is discarded and counted (`stats()`), never appended to the new list.
- **Async glue.** `lookup::run_page` / `Infinite::load_next` run a request through an iced `Task` with any `Fn(Request) -> Future` (sqlx, reqwest, ...); the library adds no runtime.

## iced mapping

`widget::InfiniteList` over `primitives::infinite` (pure, unit tested: in-flight de-duplication, stale discard after reset, exhaustion, error and retry, the threshold arithmetic, append order). It wraps a scrollable and reads its viewport; `scrolled(false)` returns content and footer for an outer scrollable.

Deviations from Mantine: no intersection observer (the trigger is the scroll offset); the loading mark is static (`rotation` can be driven by a subscription); there is no virtualization: all loaded rows are laid out.

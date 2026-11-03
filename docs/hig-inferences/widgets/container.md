# Container

No libadwaita counterpart; after Mantine's [Container](https://mantine.dev/core/container/). The base of a content page.

## Anatomy and behavior

- Centers its content and limits its width to a named size: Xs 540, Sm 720, Md 960, Lg 1140, Xl 1320 (`Px` for any other). Default Md.
- The width includes the padding. The padding is horizontal (`Metrics::page_padding`) unless set.
- `fluid` drops the limit.
- Does not scroll: a page puts it in a scrolled window (`Page` does).

## iced mapping

A `container` with `center_x(Fill)` around one with `max_width(size)` and the padding. `AdwClamp` (`clamp`) is the same in pixels; `Page` is a scrolled `Container`.

# DateRangeInput

libadwaita and GTK 4 have no date-range picker. `DateRangeInput` is the interval sibling of [`DateInput`](date-input.md).

## What it is

One entry holding `YYYY-MM-DD – YYYY-MM-DD` (read by `DateRange::parse`, written by `DateRange::text`), with the calendar button in its left section. The calendar opens in a popover, or a dialog with `dialog(true)`.

## Parameters

Those of `DateInput` (`dialog`, `scrim`, `close_on_outside`, `min`, `max`, `level`, `calendar`, `max_level`, `today`, `revealed`, `disabled`, `width`), plus:

| Param      | Meaning                                              | Default |
| ---------- | ---------------------------------------------------- | ------- |
| `selected` | the `DateRange`                                      | empty   |
| `hover`    | day under the pointer, for the preview               | none    |
| `strict`   | the end must be a later day than the start           | off     |

There is no `min_level`: a range is always picked in days.

Events: `on_input(text)`, `on_pick(range)`, `on_view`, `on_nav`, `on_hover`.

## Behavior

- **Picking** (`DateRange::pick`): the first click sets the start. The second sets the end (swapped if earlier; the same day is a range of one day). A third starts over. Days outside `min`/`max` are ignored.
- **Band.** The range is one accent band: start and end filled and rounded on the outer side, the days between tinted. With only a start, the band previews up to the hovered day.
- **Validity.** A date that does not parse or is out of range, or an end before the start, is the error state ("End must not be before start."). With `strict`, the same day is an error too. Errors show once `revealed`.
- **Levels** as in `DateInput`. Days are picked only at the month level. Month and year cells always drill down.
- Controlled: the app owns the text (set it from `DateRange::text` on a pick), the range, the calendar position and the hovered day.

## iced mapping

An `Entry` with a `Popover` (`keep_open(true)`) in its left section. The month grid is shared with `DateInput`. The hover preview is app state (`on_hover`, `hover`) because iced buttons do not expose hover.

## Deviations

One month on show. No presets. English month names, ISO dates.

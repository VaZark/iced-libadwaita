# HistogramRange

No libadwaita or GTK counterpart. After the numeric facets of Esri's `HistogramRangeSlider`: the distribution shows where values cluster, which two bare thumbs cannot.

## Anatomy and behavior

- **Bars.** One per bin of a `Histogram` (`primitives::histogram`). A bar inside the selected range is accented; the others are dimmed. An empty bin keeps a 2px stub.
- **Picking.** Pressing a bar sends that bin's `(lo, hi)`.
- **Pairing.** Put a [`RangeSlider`](range-slider.md) under it for exact thumbs. [`ComposedComponent`](composed-component.md) joins the two on one range.

## iced mapping

A row of `mouse_area` bars; the bin rules are in `primitives::histogram`.

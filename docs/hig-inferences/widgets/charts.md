# Charts

No libadwaita counterpart. After d3 (scales, shape generators, layouts), Chart.js (series, one chart type per layout) and TanStack Charts (the chart is state, drawing is a separate step).

## Three layers

| Layer | Where | Holds |
| --- | --- | --- |
| Headless | `primitives::chart` | `scale` (linear, band, nice ticks), `shape` (line, area, curves), `stack`, `pie`, `funnel`, `sankey`, `hierarchy` (sunburst). Data and a size in, numbers out. Unit tested |
| Visual | `components::chart` | what only draws: series colors, path conversion, axes and gridlines, tooltip, legend |
| Widget | `widget::{LineChart, BarChart, PieChart, Funnel, Sankey, Sunburst}` | data, the layout, the drawing pieces, and the hover |

`components/` is for visual pieces that need composing with primitives into a widget; they take the theme and numbers, never application state.

## Behavior

- Charts own their data and have no events. They remember only which mark the pointer is over, for the tooltip and highlight.
- Colors are categorical, eight GNOME hues (lighter in dark schemes), by series index; a sunburst colors by branch and fades by depth.
- Line: straight, step or monotone curves, area, stacked, 100%. Bar: grouped, stacked, 100%, horizontal, histogram (`BarChart::histogram`, with `.selected(..)` to highlight bins and `.on_pick(..)` to press one). Pie: donut with `.donut(hole)`. Funnel: continuous or blocks. Sankey: nodes made as links name them.
- Missing or non-finite values count as zero.

## Not yet

Click events, zoom, animation, time and log scales, secondary axes.

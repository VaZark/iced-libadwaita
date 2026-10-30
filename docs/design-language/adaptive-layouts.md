# Adaptive layouts

Source: [adaptive layouts](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/adaptive-layouts.html), [`AdwBreakpoint`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Breakpoint.html), [`AdwBreakpointBin`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.BreakpointBin.html), [`AdwWindow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.Window.html).

## Rules
* A window declares its **smallest supported size** (default 360×200). A widget
  that uses breakpoints has no intrinsic minimum.
* Breakpoints change widget properties when a size condition holds
  (`max-width: 400sp`, min/max width/height, aspect ratio). The last matching
  breakpoint wins; none matching = original values.
* **Labels must ellipsize or wrap and buttons must be able to shrink**
  (`can-shrink`), to survive translations and larger text.
* Clamp caps a child's maximum width but lets it shrink; padding disappears
  when space is tight.

## Standard collapse thresholds

| Pattern | Breakpoint | At narrow width |
|---|---|---|
| ViewSwitcher | `max-width: 550sp` | header switcher replaced by a revealed `ViewSwitcherBar` at the bottom |
| NavigationSplitView | `max-width: 400sp` | `collapsed`; sidebar becomes a page (`show-content` selects pane) **or, here by default, a drawer over the content** |
| OverlaySplitView | `max-width: 400sp` | `collapsed`, sidebar becomes an overlay, sidebar toggle shown |
| Tabs | `max-width: 500px` | tab bar hidden, overview button shown |
| Multi-layout | `max-width: 400sp` | switch to bottom-sheet layout |

Triple pane: outer collapses at 860sp, both at 500sp.

Dialogs are floating when there is room and a bottom sheet otherwise; never
larger than the parent window.

## Clamp
Preferences-style pages: 24px top/bottom, 12px start/end margins, 24px
spacing between groups; maximum 600.

## iced mapping
* `Breakpoint::max_width(px)` / `breakpoint_bin` (iced `responsive`).
* Gallery root uses `max-width: 400` to collapse the navigation split view and
  shows the content page with a back button.
* Row titles/subtitles wrap by word, falling back to glyphs for unbreakable
  text such as e-mail addresses (`Wrapping::WordOrGlyph`), so nothing is clipped.
* Button, tab, sidebar, header and menu labels never wrap (`typography::label`,
  `Wrapping::None`); buttons keep their natural width (`can-shrink` false).
  iced cannot ellipsize, so overflowing text is clipped at its container edge.
* `NavigationSplitView::adapt(width)` applies the docs rules: collapse at
  `max-width: 400`, sidebar = 25% clamped to 180..280.
* The header bar never squeezes start/end slots (see header-bar.md).
* **Rows flex-wrap their suffix**: when the control and title can't share a line
  (title would drop below `row_wrap_basis`, 128px), the control wraps below the
  title instead of overlapping or crushing it. This is the default for every row
  (a project extension; libadwaita itself lets the title wrap/ellipsize).

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

## Collapsed sidebar modes (`CollapsedSidebar`)

| Mode | Collapsed behavior | Sidebar color |
|---|---|---|
| `Drawer` (default) | slides over the content with a scrim; never the whole screen | sidebar |
| `FirstPage` | the landing page of a navigation stack (settings-style); choosing an item pushes the content, with a back button | **content** |

Beside the content (not collapsed) the sidebar is always a different color from the content.
Config: `[navigation] mobile-sidebar=drawer|first-page`.

Header bars are continuous: a bar has no color of its own and takes the pane behind it (sidebar
header = sidebar color, content header = content color). Headers exist to align the two panes and
to hold menu items.

### Sidebar presentation (`SidebarList`)

The sidebar's entries are described once (`SidebarItem`: icon, title, optional subtitle, selected,
message) and presented by `SidebarStyle`:

| Style | Used for | Look |
|---|---|---|
| `List` (default) | beside the content, and in a drawer | compact `.navigation-sidebar` rows on the sidebar color; subtitles hidden |
| `Cards` | the collapsed `FirstPage` sidebar | scrolling page padding, one boxed list on the content color; each row has its icon, title, **subtitle** and a chevron |

Choose `Cards` when `collapsed && mode == FirstPage`; the gallery does exactly that.

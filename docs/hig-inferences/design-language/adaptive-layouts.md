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
* **Deviation: we collapse much earlier than libadwaita.** libadwaita collapses at `400sp`, which
  leaves a sidebar beside a phone-width content area. Content must never feel squeezed, so a
  split view collapses below 768px (`md` of the Bootstrap/Mantine scale, see `breakpoints`) **or**
  whenever the content beside the sidebar would be narrower than 520px
  (`NavigationSplitView::min_content_width`, `[navigation] min-content-width`). Apps branch on
  `split_collapsed(width)` to agree with it. Mobile mode starts below 576px (`sm`) instead of 450.
* Row titles/subtitles wrap by word, falling back to glyphs for unbreakable
  text such as e-mail addresses (`Wrapping::WordOrGlyph`), so nothing is clipped.
* Button, tab, sidebar, header and menu labels never wrap (`typography::label`,
  `Wrapping::None`); buttons keep their natural width (`can-shrink` false).
  iced cannot ellipsize, so overflowing text is clipped at its container edge.
* `NavigationSplitView::adapt(width)`: sidebar = 25% clamped to 180..280 (the docs rule);
  collapse thresholds are our own, above.
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

## Other references

libadwaita is the reference for GNOME behavior, but its responsive rules are thin (a few `sp`
breakpoints on split views and dialogs). For responsive behavior in general we use **Mantine** and
**Bootstrap** as alternative references: to fill gaps Adwaita does not cover, and where its
choices are not optimal for us. They do not override the HIG; they inform decisions the HIG leaves
open, and each use is recorded here or in the component's *iced mapping*.

* **Bootstrap 5 and Mantine, the width scale.** 576, 768, 992, 1200, 1400 (Mantine's names are
  shifted by one). Used for the mobile and collapse defaults (`breakpoints`).
* **Content must keep a usable width.** The web kits collapse navigation well before the content
  gets narrow; Adwaita's 400sp leaves a phone-width content area beside a sidebar. We add a
  minimum content width on top of the scale.
* **Mantine AppShell** (header, navbar, aside, footer, main): the navbar has a `breakpoint` below
  which it is `collapsed` into an overlay, separate `collapsed` flags for mobile and desktop, and
  a transition on the change. We take the navbar breakpoint, the mobile overlay (our drawer), the
  collapse as a first-class animated state, and the main area's guaranteed width. We do not take
  per-breakpoint navbar widths, an aside and a footer (the HIG has bottom bars and secondary
  sidebars, handled separately), or CSS-timed transitions (we use the drawer's spring).
* Not yet used, candidates for later: responsive props per breakpoint (Mantine's `{ base, sm, lg }`
  style values), container queries (sizing by the parent, not the window).

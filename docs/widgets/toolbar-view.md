# ToolbarView

libadwaita: [`ToolbarView`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.ToolbarView.html)

## What it is

Content with top and/or bottom bars (1.4). Bars flat by default with scroll undershoot; `top-bar-style` flat/raised/raised-border adds opaque bg + persistent shadow. Content can extend under bars; bars can be revealed/hidden.

## Anatomy and behavior

Supported bars: HeaderBar, TabBar, ViewSwitcherBar, ActionBar, SearchBar, any `.toolbar` box. Bars act as window handles.

## CSS nodes and classes

`toolbarview`; classes `.toolbar`, `.undershoot-top/bottom`.

## Spec

Bar spacing 6.

## iced mapping

`toolbar_view(content).top_bar().bottom_bar().top_bar_style(BarStyle::{Flat,Raised,RaisedBorder})`. Flat (default, no line) matches Adwaita; Raised draws the `headerbar-shade` shadow as a 6px fading strip; RaisedBorder a 1px `--headerbar-darker-shade-color` line.

## Deviations / gaps

No scroll undershoot (Adwaita's flat-bar scroll shadow); no extend-under.

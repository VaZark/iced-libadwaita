# Breakpoint and BreakpointBin

libadwaita: [`BreakpointBin`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.BreakpointBin.html)

## What it is

Conditions on width/height/aspect that apply setters; `BreakpointBin` scopes them to a subtree. Adding a breakpoint removes minimum size, so request one explicitly. Last match wins.

## Anatomy and behavior

Condition syntax e.g. `max-width: 400px`; unit `sp` scales with text.

## Spec

Default window min 360×200.

## iced mapping

`Breakpoint::{max_width,min_width}`, `breakpoint_bin(bp, narrow, wide)` (iced `responsive`).

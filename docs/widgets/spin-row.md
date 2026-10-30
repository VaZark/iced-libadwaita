# SpinRow

libadwaita: [`SpinRow`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.SpinRow.html)

## What it is

An action row with an embedded spin button.

## Anatomy and behavior

Adjustment (range, step), `digits`, `numeric`, `snap-to-ticks`, `wrap`, `climb-rate`. Role: spin button.

## CSS nodes and classes

Same as ActionRow plus `.spin`.

## Spec

Suffix holds `−` value `+` (linked flat buttons).

## iced mapping

`SpinRow::new(title, value, range, step, on_change).digits(n)`.

## Deviations / gaps

No wrap / climb-rate.

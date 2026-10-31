# PreferencesPage

libadwaita: [`PreferencesPage`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.PreferencesPage.html)

## What it is

A scrollable page of preference groups (page of a PreferencesDialog).

## Anatomy and behavior

`title`, `description`, `description-centered`, `banner` (1.7), `icon-name`. Scrolls; groups stacked.

## CSS nodes and classes

`preferencespage`.

## Spec

Clamped to 600 max, margins 24 vertical / 12 horizontal, 24 between groups (docs example).

## iced mapping

`PreferencesPage::new().push(group)` → scrollable + `clamp(600)`.

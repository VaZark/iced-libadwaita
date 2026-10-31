# NavigationView

libadwaita: [`NavigationView`](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.NavigationView.html)

## What it is

Page stack with push/pop/replace; header bar gets a back button automatically (1.4).

## Anatomy and behavior

Back: Esc, Alt+Left, mouse back, swipe. Forward only if the app provides the next page. `can-pop`, `animate-transitions`, homogeneous options.

## CSS nodes and classes

`navigation-view`; pages: `navigation-view-page`.

## iced mapping

`navigation_view(pages, on_pop)` shows the top page with a back button when depth > 1.

## Deviations / gaps

No gestures/animations/keyboard shortcuts.

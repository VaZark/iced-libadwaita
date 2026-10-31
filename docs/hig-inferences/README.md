# HIG inferences

What we target, inferred from the [GNOME HIG](https://developer.gnome.org/hig/), GTK4 and
libadwaita. Read these before changing a component.

#### Contents

* [design-language/](design-language/README.md): colors, typography, style classes, metrics,
  shadows, animations, adaptive layouts, windows, menus.
* [widgets/](widgets/): one file per element.
* [gtk4/](gtk4/): the plain GTK4 widgets Adwaita builds on (scrollbars, entries, popovers…).
  Where GTK4 and libadwaita differ, the HIG decides and libadwaita is the tie-breaker.

#### How to read them

* The primary sources are the [libadwaita docs](https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/index.html)
  (v1.11 alpha), the HIG and the GTK4 docs. They describe behavior, structure (CSS nodes) and
  colors but rarely pixel sizes.
* A size that is not in the docs is marked **✓** (verified against libadwaita's stylesheet,
  `src/stylesheet/**/*.scss`) or **‡** (from memory, unverified). Anything unspecified is
  recorded as "not specified" rather than guessed.
* Each file ends with an **iced mapping** section: how we build it, and where we deviate from the
  reference and why. Deviations are *usually* recorded there.

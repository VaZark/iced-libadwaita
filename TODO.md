# TODO

1. Clip bg extending past window when it's not intended behaviour : test weave's full-screen bottom sheet overriding border-radius
2. Icon naming and resolution
   1. General:
      1. **Remove all first-party icon assets and bundled icon collections from the library.**
      2. Replace existing icon implementations with an XDG-compliant icon resolver.
      3. Preserve the existing icon API and enums where possible.
      4. Use freedesktop.org specifications as the source of truth for icon naming and resolution.
      5. Icon assets must be supplied by the installed system icon theme.
      6. Missing icons must not cause application failures.
      7. Preserve compatibility with existing widgets and applications wherever possible.

   2. XDG implementation:
      1. Follow the Icon Naming Specification for standardized icon names.
      2. Follow the Icon Theme Specification for:
         1. Theme discovery and inheritance.
         2. Icon lookup, size, and scale selection.
         3. `hicolor` and unthemed fallback.
      3. Support regular and symbolic icons.
      4. Implement symbolic fallback:
         1. Prefer `<name>-symbolic` when symbolic is requested.
         2. Fall back to `<name>` if unavailable.
         3. Return `None` if neither resolves.
      5. Support symbolic recoloring based on the widget's current foreground color and state.
      6. Cache resolved icons and invalidate the cache when the active theme changes.
      7. Avoid GNOME, KDE, or COSMIC-specific icon integrations.
   
   3.  Symbolic icon colors and contrast:
      1. Resolve symbolic icons using the installed XDG icon theme.
      2. Recolor symbolic icons using the widget's computed foreground color by default.
      3. Automatically update icon colors when widget state, theme, or background changes.
      4. Support explicit foreground color overrides.
      5. Ensure the styling system selects foreground colors with sufficient contrast against the effective background.
      6. Preserve semantic colors (success, warning, error) where supported by the symbolic icon format.
      7. Apply the same behavior to HeaderLayer window controls and all other widgets.

   4. Migration:
      1. Audit existing icon assets, constants, enums, and widget dependencies.
      2. Map existing icon constants to standardized XDG identifiers wherever possible.
      3. Replace direct references to bundled assets with calls to the icon resolver.
      4. Update widgets, examples, and documentation to use the new resolver.
      5. Remove obsolete icon assets, embedded fonts, dependencies, and resource-loading code.
      6. Verify icon resolution using installed Linux themes.
      7. Test missing icons, symbolic fallback, recoloring, and theme changes.

   5. References:
      1. https://specifications.freedesktop.org/icon-naming/latest/
      2. https://specifications.freedesktop.org/icon-theme/latest/

3. Linux platform integration
   1. General architecture:
      1. Target Linux as the primary supported platform.
      2. Use freedesktop.org specifications and XDG portals as the source of truth for system integration.
      3. Prefer existing Iced/Winit functionality over implementing platform integrations from scratch.
      4. Avoid desktop-environment-specific integrations when standardized interfaces are available.
      5. Keep platform integrations separate from widget presentation and application logic.
      6. Unsupported platform functionality must fail gracefully.
      7. Do not introduce cross-platform service registries, provider mappings, or custom backend abstractions without a concrete requirement.
      8. Do not implement Windows or macOS-specific integrations at this stage.

   2. Window decorations:
      1. Use client-side decorations (CSD) exclusively.
      2. Use the existing HeaderLayer as the primary window layout primitive.
      3. Keep HeaderBar optional and independent of HeaderLayer.
      4. Support edge-to-edge content and backgrounds beneath the header.
      5. Integrate window controls, dragging, resizing, and window state with Iced/Winit.
      6. Avoid additional decoration libraries unless required.
      7. Do not implement SSD support or decoration-mode switching.

   3. File chooser:
      1. Provide an API for:
         1. Opening files.
         2. Saving files.
         3. Selecting folders.
         4. Selecting multiple files.
         5. File type filters.
      2. Use the XDG FileChooser portal.
      3. Support asynchronous operations and cancellation.
      4. Return structured results for successful selection, cancellation, and errors.
      5. Avoid implementing custom file chooser widgets when a system dialog is available.

   4. System appearance:
      1. Detect system light/dark mode.
      2. Support system accent colors and high-contrast preferences where available.
      3. Use the XDG Settings portal where applicable.
      4. React to system appearance changes at runtime.
      5. Allow applications to override system preferences.
      6. Fall back to library defaults when system settings are unavailable.
      7. Keep appearance detection separate from theme and styling implementation.

   5. Other system integration:
      1. Prefer XDG portals for supported operations.
      2. Support system integration where needed for:
         1. Opening URLs and files with default applications.
         2. Desktop notifications.
         3. Native system dialogs.
      3. Delegate clipboard, drag-and-drop, accessibility, and other functionality to Iced/Winit where already supported.
      4. Keep optional integrations separate from the core widget library.
      5. Do not implement features without a concrete application requirement.

   6. References:
      1. https://flatpak.github.io/xdg-desktop-portal/
      2. https://docs.rs/iced/latest/iced/window/
      3. https://docs.rs/winit/latest/winit/
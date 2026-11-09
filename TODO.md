# TODO

1. Clip bg extending past window when it's not intended behaviour : test weave's full-screen bottom sheet overriding border-radius
 
2. Linux platform integration
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
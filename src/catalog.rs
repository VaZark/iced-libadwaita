//! A machine-readable catalog of every component.
//!
//! This is the contract a declarative layer (XML/Blueprint, JSX-like, a macro) binds to: for
//! each component it lists the **tag**, the libadwaita/GTK class it mirrors, the Rust
//! **constructor**, its **properties** (builder methods, named after libadwaita's properties),
//! its child **slots**, and its **events** (callbacks). All components follow the conventions
//! in `docs/components.md`:
//!
//! * constructed with `Component::new(required…)` or the free function of the same name;
//! * properties are chained builder methods; `.metrics(..)` is accepted everywhere sizes matter;
//! * children go in slots (`.push(..)`, `.child(..)`, `.start(..)`, …);
//! * events are `on_*` callbacks producing the app's message;
//! * state is **controlled**: the app owns it (`active`, `expanded`, `value`…) and updates it
//!   from the events;
//! * every component converts `Into<Element>`.

/// One component's binding metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Component {
    /// The tag a layout uses, e.g. `action_row`.
    pub tag: &'static str,
    /// The libadwaita / GTK class mirrored (or `-` for project components).
    pub adw: &'static str,
    /// The Rust constructor path under `iced_libadwaita::widget`.
    pub ctor: &'static str,
    /// Properties: builder methods.
    pub props: &'static [&'static str],
    /// Child slots: methods that add children.
    pub slots: &'static [&'static str],
    /// Events: `on_*` callbacks.
    pub events: &'static [&'static str],
}

const fn c(
    tag: &'static str,
    adw: &'static str,
    ctor: &'static str,
    props: &'static [&'static str],
    slots: &'static [&'static str],
    events: &'static [&'static str],
) -> Component {
    Component { tag, adw, ctor, props, slots, events }
}

/// Every component.
pub static COMPONENTS: &[Component] = &[
    // -- windows & bars
    c("window", "AdwWindow", "window(content)", &[], &["content"], &[]),
    c("header_bar", "AdwHeaderBar", "header_bar()", &["title", "subtitle", "flat", "window_radius", "window_corners", "show_start_title_buttons", "show_end_title_buttons", "window_controls", "metrics"], &["pack_start", "pack_end", "title_widget"], &["on_drag", "on_double_click"]),
    c("window_title", "AdwWindowTitle", "window_title(title, subtitle)", &[], &[], &[]),
    c("toolbar_view", "AdwToolbarView", "toolbar_view(content)", &["top_bar_style", "bottom_bar_style"], &["top_bar", "bottom_bar", "content"], &[]),
    c("action_bar", "GtkActionBar", "action_bar()", &[], &["pack_start", "center", "pack_end"], &[]),
    c("tab_bar", "AdwTabBar", "tab_bar(tabs, active, on_select, on_close)", &["tabs", "active"], &[], &["on_select", "on_close"]),
    c("tab_button", "AdwTabButton", "tab_button(count, on_press)", &["count"], &[], &["on_press"]),
    c("path_bar", "-", "path_bar(segments, current, on_select)", &["segments", "current"], &[], &["on_select"]),
    // -- navigation
    c("navigation_split_view", "AdwNavigationSplitView", "NavigationSplitView::new(sidebar, content)", &["collapsed", "show_content", "sidebar_width", "sidebar_width_fraction", "min_sidebar_width", "max_sidebar_width", "collapsed_sidebar", "show_sidebar", "window_radius", "adapt"], &["sidebar", "content"], &["on_close_drawer"]),
    c("sidebar_list", "GtkListBox.navigation-sidebar", "SidebarList::new(items)", &["style", "window_radius"], &["items"], &[]),
    c("overlay_split_view", "AdwOverlaySplitView", "OverlaySplitView::new(sidebar, content)", &["collapsed", "show_sidebar", "sidebar_width"], &["sidebar", "content"], &["on_close"]),
    c("navigation_view", "AdwNavigationView", "navigation_view(pages, on_pop)", &["pages"], &["pages"], &["on_pop"]),
    c("navigation_sidebar", "GtkListBox.navigation-sidebar", "navigation_sidebar(rows)", &[], &["rows"], &[]),
    c("sidebar_row", "AdwSidebarItem", "sidebar_row(icon, label, selected, on_press)", &["icon", "label", "selected"], &[], &["on_press"]),
    c("view_switcher", "AdwViewSwitcher", "view_switcher(items, active, on_select)", &["items", "active", "width", "homogeneous"], &[], &["on_select"]),
    c("view_switcher_bar", "AdwViewSwitcherBar", "view_switcher_bar(items, active, on_select)", &["items", "active"], &[], &["on_select"]),
    c("toggle_group", "AdwToggleGroup", "toggle_group(items, active, on_select)", &["items", "active", "width", "homogeneous"], &["items"], &["on_select"]),
    c("carousel", "AdwCarousel", "carousel(pages, index, on_navigate)", &["index", "indicator", "spacing", "interactive"], &["pages"], &["on_navigate"]),
    c("breakpoint_bin", "AdwBreakpointBin", "breakpoint_bin(breakpoint, narrow, wide)", &["breakpoint"], &["narrow", "wide"], &[]),
    c("bottom_sheet", "AdwBottomSheet", "bottom_sheet_in_window(content, sheet, open, on_close, radius)", &["open"], &["content", "sheet"], &["on_close"]),
    // -- boxed lists & rows
    c("boxed_list", "GtkListBox.boxed-list", "boxed_list(rows)", &[], &["rows"], &[]),
    c("action_row", "AdwActionRow", "ActionRow::new(title)", &["subtitle", "chevron", "wrap_suffix", "collapse", "suffix_align", "suffix_basis", "metrics"], &["prefix", "suffix"], &["on_activate"]),
    c("switch_row", "AdwSwitchRow", "SwitchRow::new(title, active, on_toggle)", &["subtitle", "metrics"], &["prefix"], &["on_toggle"]),
    c("combo_row", "AdwComboRow", "ComboRow::new(title, options, selected, on_select)", &["subtitle"], &[], &["on_select"]),
    c("expander_row", "AdwExpanderRow", "ExpanderRow::new(title, expanded, on_toggle)", &["subtitle"], &["prefix", "push"], &["on_toggle"]),
    c("entry_row", "AdwEntryRow", "EntryRow::new(title, value)", &["password", "collapse", "metrics"], &["suffix"], &["on_input", "on_submit"]),
    c("password_entry_row", "AdwPasswordEntryRow", "password_entry_row(title, value)", &[], &["suffix"], &["on_input", "on_submit"]),
    c("spin_row", "AdwSpinRow", "SpinRow::new(title, value, range, step, on_change)", &["subtitle", "digits"], &[], &["on_change"]),
    c("button_row", "AdwButtonRow", "ButtonRow::new(title)", &["role"], &[], &["on_activate"]),
    c("slider_row", "-", "SliderRow::new(title, value, range, on_change)", &["subtitle", "step", "basis", "collapse"], &[], &["on_change"]),
    c("toggle_group_row", "-", "ToggleGroupRow::new(title, items, active, on_select)", &["subtitle", "homogeneous", "collapse"], &[], &["on_select"]),
    c("preferences_group", "AdwPreferencesGroup", "preferences_group()", &["title", "description"], &["header_suffix", "push"], &[]),
    c("preferences_page", "AdwPreferencesPage", "preferences_page()", &[], &["push"], &[]),
    c("flex_row", "-", "FlexRow::new(main, aside)", &["spacing", "line_spacing", "main_basis", "aside_basis", "aside_align", "collapse"], &["main", "aside"], &[]),
    // -- buttons & controls
    c("button", "GtkButton", "raised_button(content)", &["style"], &["content"], &["on_press"]),
    c("flat_button", "GtkButton.flat", "flat_button(content)", &[], &["content"], &["on_press"]),
    c("suggested_button", "GtkButton.suggested-action", "suggested_button(content)", &[], &["content"], &["on_press"]),
    c("destructive_button", "GtkButton.destructive-action", "destructive_button(content)", &[], &["content"], &["on_press"]),
    c("pill_button", "GtkButton.pill", "pill_button(content)", &[], &["content"], &["on_press"]),
    c("circular_button", "GtkButton.circular", "circular_button(icon)", &[], &[], &["on_press"]),
    c("icon_button", "GtkButton (icon)", "icon_button(icon)", &[], &[], &["on_press"]),
    c("button_content", "AdwButtonContent", "ButtonContent::new(label)", &["icon", "metrics"], &[], &[]),
    c("split_button", "AdwSplitButton", "split_button(content, on_press, on_menu)", &[], &["content"], &["on_press", "on_menu"]),
    c("menu_button", "GtkMenuButton", "menu_button(icon, menu)", &["presentation", "tooltip", "window_radius"], &["menu"], &[]),
    c("primary_menu", "GtkMenuButton (primary)", "primary_menu(menu)", &["presentation", "window_radius"], &["menu"], &[]),
    c("link_button", "GtkLinkButton", "link_button(label)", &[], &[], &["on_press"]),
    c("card_button", "GtkButton.card", "card_button(content)", &[], &["content"], &["on_press"]),
    c("switch", "GtkSwitch", "switch(active)", &[], &[], &["on_toggle"]),
    c("check_button", "GtkCheckButton", "check_button(label, active)", &[], &[], &["on_toggle"]),
    c("radio_button", "GtkCheckButton (group)", "radio_button(label, value, selected, on_select)", &[], &[], &[]),
    c("scale", "GtkScale", "scale(range, value, on_change)", &["step", "width"], &[], &["on_change"]),
    c("progress_bar", "GtkProgressBar", "progress_bar(range, value)", &["style"], &[], &[]),
    c("entry", "GtkEntry", "entry(placeholder, value)", &["state", "password", "width", "metrics"], &[], &["on_input", "on_submit"]),
    c("password_entry", "GtkPasswordEntry", "password_entry(placeholder, value)", &["state", "width"], &[], &["on_input", "on_submit"]),
    c("text_view", "GtkTextView", "text_view(content)", &["placeholder", "height"], &[], &["on_action"]),
    c("dropdown", "GtkDropDown", "dropdown(options, selected, on_select)", &["appearance", "align", "width", "popover_width", "presentation"], &[], &["on_select"]),
    c("spinner", "AdwSpinner", "spinner(size, rotation)", &[], &[], &[]),
    // -- dialogs & feedback
    c("dialog", "AdwDialog", "dialog_in_window(base, dialog, on_backdrop, radius)", &[], &["base", "dialog"], &["on_backdrop"]),
    c("alert_dialog", "AdwAlertDialog", "AlertDialog::new(heading)", &["body", "metrics"], &["extra_child", "response"], &["response"]),
    c("about_dialog", "AdwAboutDialog", "AboutDialog::new(name)", &["developer_name", "version", "comments", "website", "license"], &["application_icon"], &["on_close"]),
    c("shortcuts_dialog", "AdwShortcutsDialog", "shortcuts_dialog(sections, on_close)", &["sections"], &[], &["on_close"]),
    c("preferences_dialog", "AdwPreferencesDialog", "preferences_dialog(title, page, on_close)", &["title"], &["page"], &["on_close"]),
    c("banner", "AdwBanner", "banner(title)", &["revealed", "text_color"], &[], &["button"]),
    c("toast_overlay", "AdwToastOverlay", "toast_overlay(content, toast)", &["role", "status", "indicator", "background", "text_color"], &["content"], &["action", "on_dismiss"]),
    c("status_page", "AdwStatusPage", "status_page()", &["title", "description", "compact", "metrics"], &["icon", "child"], &[]),
    c("avatar", "AdwAvatar", "avatar(size, text)", &["show_initials", "color", "text_color"], &[], &[]),
    c("tag", "-", "tag(label, role)", &[], &[], &[]),
    c("shortcut_label", "AdwShortcutLabel", "shortcut_label(accelerator)", &[], &[], &[]),
    // -- layout & surfaces
    c("clamp", "AdwClamp", "clamp(content, maximum)", &["maximum"], &["content"], &[]),
    c("wrap_box", "AdwWrapBox", "wrap_box(children, spacing)", &["spacing"], &["children"], &[]),
    c("bin", "AdwBin", "bin(content)", &[], &["content"], &[]),
    c("card", "GtkBox.card", "Card::new(content)", &["padding", "width", "height", "separators"], &["content", "push"], &[]),
    c("page", "-", "Page::new(content)", &["maximum_size", "metrics"], &["content"], &[]),
    c("scrolled_window", "GtkScrolledWindow", "scrolled_window(content)", &[], &["content"], &[]),
    c("color_swatch", "-", "color_swatch(label, surface)", &[], &[], &[]),
    c("accent_picker", "-", "accent_picker(selected, on_select)", &["selected"], &[], &["on_select"]),
    // -- motion
    c("reveal", "GtkRevealer", "reveal(content, revealed)", &["revealed", "spec"], &["content"], &[]),
    c("drawer_layer", "AdwOverlaySplitView (overlay)", "drawer_layer(pane, open, width)", &["open", "window_radius"], &["pane"], &["on_close"]),
    c("entrance", "-", "entrance(content, offset)", &["spec", "scrim"], &["content"], &[]),
];

/// Looks a component up by tag.
pub fn find(tag: &str) -> Option<&'static Component> {
    COMPONENTS.iter().find(|c| c.tag == tag)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn tags_are_unique_snake_case() {
        let mut seen = HashSet::new();
        for c in COMPONENTS {
            assert!(seen.insert(c.tag), "duplicate tag {}", c.tag);
            assert!(
                c.tag.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_' || ch.is_ascii_digit()),
                "{} is not snake_case",
                c.tag
            );
            assert!(!c.ctor.is_empty() && !c.adw.is_empty());
        }
    }

    #[test]
    fn events_follow_the_naming_convention() {
        // `response` (AlertDialog) is the one non-`on_*` event name: it is a slot-like builder.
        for c in COMPONENTS {
            for e in c.events {
                assert!(
                    e.starts_with("on_") || ["response", "button", "action"].contains(e),
                    "{}: event `{}` should be on_*",
                    c.tag,
                    e
                );
            }
        }
    }

    #[test]
    fn lookup_and_coverage() {
        assert_eq!(find("action_row").unwrap().adw, "AdwActionRow");
        assert!(find("nope").is_none());
        // The libadwaita gallery's headline widgets are all present.
        for tag in [
            "action_row", "switch_row", "combo_row", "expander_row", "entry_row", "spin_row", "button_row",
            "preferences_group", "preferences_page", "header_bar", "toolbar_view", "navigation_split_view",
            "status_page", "banner", "avatar", "toggle_group", "alert_dialog", "about_dialog", "clamp",
        ] {
            assert!(find(tag).is_some(), "{tag} missing from the catalog");
        }
        assert!(COMPONENTS.len() >= 60);
    }
}

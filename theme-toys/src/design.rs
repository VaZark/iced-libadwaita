//! Versioned, app-owned geometry schema. Theme files override every value in logical px.
use iced_adaptive_design::theme_config::{Inheritance, Schema, ThemeConfig, Value, VariableType};

pub const GEOMETRY: &[(&str, &str, f32)] = &[
    ("toy.win95", "width", 404.0),
    ("toy.win95", "height", 448.0),
    ("toy.win95", "tab-panel-gap", 0.0),
    ("toy.win95", "list-height", 112.0),
    ("toy.win95", "list-row-height", 18.0),
    ("toy.win95", "command-width", 75.0),
    ("toy.win95", "panel-padding", 12.0),
    ("toy.win95", "group-width", 100.0),
    ("toy.win95", "group-padding", 6.0),
    ("toy.win95", "preview-gap", 16.0),
    ("toy.win95", "list-inset", 1.0),
    ("toy.win95", "monitor-width", 178.0),
    ("toy.win95", "screen-width", 154.0),
    ("toy.win95", "screen-height", 112.0),
    ("toy.win95", "bezel-padding", 8.0),
    ("toy.win95", "bezel-radius", 4.0),
    ("toy.win95", "screen-radius", 1.0),
    ("toy.win95", "indicator-padding", 6.0),
    ("toy.win95", "indicator-gap", 2.0),
    ("toy.win95", "border-width", 1.0),
    ("toy.win95", "neck-width", 32.0),
    ("toy.win95", "neck-height", 5.0),
    ("toy.win95", "foot-width", 90.0),
    ("toy.win95", "foot-height", 4.0),
    ("toy.win95", "tile-width", 16.0),
    ("toy.win95", "tile-height", 16.0),
    ("toy.win95", "tile-gap", 5.0),
    ("toy.win95", "picture-width", 64.0),
    ("toy.win95", "picture-height", 42.0),
    ("toy.win95", "wait-width", 60.0),
    ("toy.winxp", "width", 1024.0),
    ("toy.winxp", "height", 738.0),
    ("toy.winxp", "task-row-height", 20.0),
    ("toy.winxp", "task-heading-height", 24.0),
    ("toy.winxp", "sidebar-width", 210.0),
    ("toy.winxp", "task-padding", 12.0),
    ("toy.winxp", "task-gap", 14.0),
    ("toy.winxp", "task-body-padding", 7.0),
    ("toy.winxp", "menu-height", 22.0),
    ("toy.winxp", "toolbar-height", 40.0),
    ("toy.winxp", "address-height", 22.0),
    ("toy.winxp", "section-height", 24.0),
    ("toy.winxp", "item-height", 72.0),
    ("toy.winxp", "item-width", 220.0),
    ("toy.winxp", "item-icon-size", 40.0),
    ("toy.winxp", "command-icon-size", 22.0),
    ("toy.win7", "width", 1280.0),
    ("toy.win7", "height", 720.0),
    ("toy.win7", "menu-width", 498.0),
    ("toy.win7", "menu-height", 598.0),
    ("toy.win7", "programs-width", 304.0),
    ("toy.win7", "menu-padding", 8.0),
    ("toy.win7", "menu-radius", 7.0),
    ("toy.win7", "program-row-height", 23.0),
    ("toy.win7", "place-row-height", 42.0),
    ("toy.win7", "search-height", 54.0),
    ("toy.win7", "taskbar-height", 48.0),
    ("toy.win7", "task-button-width", 60.0),
    ("toy.win7", "program-indent", 24.0),
    ("toy.win7", "program-padding", 4.0),
    ("toy.win7", "user-height", 86.0),
    ("toy.win7", "user-icon-size", 48.0),
    ("toy.win7", "desktop-icon-size", 32.0),
    ("toy.mate", "width", 1280.0),
    ("toy.mate", "height", 675.0),
    ("toy.mate", "panel-height", 18.0),
    ("toy.mate", "desktop-padding", 36.0),
    ("toy.mate", "editor-x", 114.0),
    ("toy.mate", "editor-y", 86.0),
    ("toy.mate", "editor-width", 408.0),
    ("toy.mate", "editor-height", 410.0),
    ("toy.mate", "help-x", 798.0),
    ("toy.mate", "help-y", 32.0),
    ("toy.mate", "help-width", 444.0),
    ("toy.mate", "help-height", 410.0),
    ("toy.mate", "title-height", 20.0),
    ("toy.mate", "toolbar-height", 32.0),
    ("toy.mate", "menu-height", 20.0),
    ("toy.mate", "tab-height", 20.0),
    ("toy.mate", "window-border", 1.0),
    ("toy.mate", "window-radius", 4.0),
    ("toy.mate", "document-padding", 8.0),
    ("toy.mate", "heading-size", 22.0),
    ("toy.mate", "desktop-item-width", 110.0),
    ("toy.mate", "desktop-item-height", 62.0),
    ("toy.mate", "desktop-icon-size", 28.0),
    ("toy.mate", "editor-font-size", 10.0),
];

pub fn schema() -> Schema {
    let mut schema = Schema::new();
    for &(section, key, default) in GEOMETRY {
        schema = schema.variable(
            section,
            key,
            VariableType::Length {
                min: Some(bounds(key, default).0.into()),
                max: Some(bounds(key, default).1.into()),
            },
            Some(Value::Length(default.into())),
            "Reference scene geometry, in logical px",
        );
    }
    schema = schema.variable(
        "toy",
        "overflow",
        VariableType::String {
            values: ["grow", "scroll", "scale-down"].map(String::from).to_vec(),
            accepts_random: false,
        },
        Some(Value::Text("grow".into())),
        "Natural scene overflow: grow, two-axis scroll, or contain without upscaling",
    );
    for name in ["desktop", "taskbar", "start-menu"] {
        schema = schema.variable(
            "gradients",
            name,
            VariableType::Gradient,
            None,
            "Reference scene background gradient",
        );
    }
    schema
}

pub fn scene_text(design: usize) -> &'static str {
    match design {
        0 => include_str!("../themes/win95.ron"),
        1 => include_str!("../themes/winxp.ron"),
        2 => include_str!("../themes/win7.ron"),
        _ => include_str!("../themes/mate.ron"),
    }
}

/// Individual token validation; whole-scene negotiation belongs to the overflow layout.
pub fn bounds(key: &str, default: f32) -> (f32, f32) {
    match key {
        "width" | "height" => (default, default * 1.5),
        "border-width" | "window-border" => (0.0, 4.0),
        "editor-font-size" => (8.0, 24.0),
        "heading-size" => (12.0, 40.0),
        key if key.ends_with("radius") => (0.0, 24.0),
        key if key.ends_with("padding") || key.ends_with("gap") || key.ends_with("inset") => {
            (0.0, (default * 2.0).max(12.0))
        }
        _ => ((default * 0.5).max(1.0), default * 1.5),
    }
}

fn build(design: usize, overlay: &str) -> ThemeConfig {
    ThemeConfig::builder("org.iad.ThemeToys")
        .files(false)
        .schema(schema())
        .inheritance(Inheritance::OptOut)
        .base_theme(scene_text(design))
        .app_text(overlay)
        .build()
}

/// Actual resolved IAD lengths, not the manifest constants, determine containment.
pub fn layout_fits(config: &ThemeConfig, design: usize) -> bool {
    let section = ["toy.win95", "toy.winxp", "toy.win7", "toy.mate"][design];
    let n = |key: &str| config.number(section, key).expect("declared geometry");
    let w = n("width");
    let h = n("height");
    match design {
        0 => {
            let vertical_delta =
                n("screen-height") - 112.0 + 2.0 * (n("bezel-padding") - 8.0) + n("preview-gap")
                    - 16.0
                    + n("list-height")
                    - 112.0
                    + 2.0 * (n("panel-padding") - 12.0)
                    + n("tab-panel-gap")
                    + n("neck-height")
                    - 5.0
                    + n("foot-height")
                    - 4.0;
            n("screen-width") + 2.0 * (n("bezel-padding") + n("border-width")) <= n("monitor-width")
                && n("monitor-width") + 2.0 * n("panel-padding") <= w
                && n("group-width") + 2.0 * n("panel-padding") + 120.0 <= w
                && 448.0 + vertical_delta <= h
        }
        1 => {
            n("sidebar-width") + 2.0 * n("item-width") <= w
                && n("menu-height")
                    + n("toolbar-height")
                    + n("address-height")
                    + 4.0 * (n("section-height") + n("item-height"))
                    <= h
        }
        2 => {
            n("menu-width") <= w
                && n("menu-height") + n("taskbar-height") <= h
                && n("programs-width") + 2.0 * n("menu-padding") + 140.0 <= n("menu-width")
                && n("user-height") + 9.0 * n("place-row-height") + n("search-height")
                    <= n("menu-height")
        }
        _ => ["editor", "help"].iter().all(|name| {
            let value = |suffix| n(&format!("{name}-{suffix}"));
            value("x") + value("width") <= w
                && value("y") + value("height") <= h - 2.0 * n("panel-height")
                && n("title-height") + n("toolbar-height") + n("tab-height") < value("height")
        }),
    }
}

pub fn resolve(design: usize, overlay: &str) -> ThemeConfig {
    let config = build(design, overlay);
    if layout_fits(&config, design) {
        return config;
    }
    // Keep editor palette/font/native-widget edits, but reject an impossible scene rectangle.
    let mut geometry = false;
    let safe: String = overlay
        .lines()
        .filter(|line| {
            let line = line.trim();
            if line.starts_with('[') {
                geometry = line.contains("toy.");
            }
            !geometry
        })
        .map(|line| format!("{line}\n"))
        .collect();
    eprintln!(
        "theme-toys: scene geometry exceeds its bounds; using base geometry, keeping native theme edits"
    );
    build(design, &safe)
}

pub fn config(design: usize) -> ThemeConfig {
    let overlay = match std::env::var_os("IAD_THEME") {
        Some(path) => std::fs::read_to_string(&path).unwrap_or_else(|error| {
            eprintln!(
                "theme-toys: cannot read IAD_THEME {:?}: {error}; using base",
                path
            );
            String::new()
        }),
        None => String::new(),
    };
    resolve(design, &overlay)
}

pub fn px(section: &str, key: &str) -> f32 {
    iced_adaptive_design::theme_config::current()
        .number(section, key)
        .unwrap_or_else(|| {
            GEOMETRY
                .iter()
                .find(|(s, k, _)| *s == section && *k == key)
                .unwrap_or_else(|| panic!("undeclared geometry {section}.{key}"))
                .2
        })
}

pub fn size(design: usize) -> iced::Size {
    let section = ["toy.win95", "toy.winxp", "toy.win7", "toy.mate"][design];
    iced::Size::new(px(section, "width"), px(section, "height"))
}

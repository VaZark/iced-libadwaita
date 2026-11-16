//! Reference manifest v1. Metadata and measurements only; no copyrighted pixels are bundled.
//! Coordinates are measured/estimated logical-pixel targets, not automated pixel equivalence.
#[allow(dead_code)]
pub struct Reference {
    pub key: &'static str,
    pub source: &'static str,
    pub image: &'static str,
    pub raster: (u32, u32),
    pub logical: (f32, f32),
    pub body_px: f32,
    pub scene: &'static str,
}
#[allow(dead_code)]
pub const VERSION: u32 = 1;
#[allow(dead_code)]
pub const TARGETS: &[Reference] = &[
    Reference {
        key: "win95",
        source: "https://guidebookgallery.org/screenshots/winserv2003web",
        image: "https://guidebookgallery.org/pics/gui/settings/appearance/winserv2003web-1-1.png",
        raster: (404, 448),
        logical: (404.0, 448.0),
        body_px: 11.0,
        scene: "Server 2003 Classic Display Properties Desktop tab; classic visual target, not Windows 95 RTM. 178px monitor, 75px commands, 2px bevels; title gradient 90deg navy to pale blue.",
    },
    Reference {
        key: "winxp",
        source: "https://i.pinimg.com/1200x/71/65/b4/7165b40068a1b127276e482f04901a07.jpg",
        image: "https://i.pinimg.com/1200x/71/65/b4/7165b40068a1b127276e482f04901a07.jpg",
        raster: (1024, 768),
        logical: (1024.0, 738.0),
        body_px: 11.0,
        scene: "Maximized My Computer; 30px desktop taskbar excluded. Sidebar 210/1024 = 20.51%; toolbar40, menubar22, address22; four stacked sections, no notebook or usage bars. Sidebar gradient #7ba2e7 to #6375d6.",
    },
    Reference {
        key: "win7",
        source: "https://windows7lesson.blogspot.com/",
        image: "https://cdn.neowin.com/news/images/uploaded/2023/04/1680968851_windows-7-start-menu.jpg",
        raster: (1280, 720),
        logical: (1280.0, 720.0),
        body_px: 14.0,
        scene: "All Programs Start menu at bottom left. Menu498x598, programs304/498=61.04%, taskbar48px; 23px program rows,42px places rows. Blue desktop gradient is an approximation, no proprietary wallpaper or real glass blur.",
    },
    Reference {
        key: "mate",
        source: "https://mate-desktop.org/gallery/1.26/english/",
        image: "https://mate-desktop.org/gallery/1.26/english/mate-1.26_7.png",
        raster: (1280, 675),
        logical: (1280.0, 675.0),
        body_px: 10.0,
        scene: "Official MATE1.26 Pluma left and Desktop User Guide right; downloaded source is 1280x675, mapped 1:1 to logical scene coordinates. Panels18px, editor408x410 at114,104 including top panel, help444x410 at798,50. Silver title gradient #eeeeee to #c5c5c5; blue wallpaper approximated by a gradient.",
    },
];

/// Secondary classic source: adapter page, NOT the same scene/era as Server 2003 Desktop.
#[allow(dead_code)]
pub const CLASSIC_SECONDARY: (&str, &str, (u32, u32)) = (
    "http://www.playtool.com/pages/uninstallgraphics/uninstall.html",
    "http://www.playtool.com/pages/uninstallgraphics/win9x1.gif",
    (416, 468),
);

/// Expected CSS stops are tested through IAD's actual typed gradient parser.
#[allow(dead_code)]
pub const GRADIENTS: &[(&str, &str, &str)] = &[
    (
        "win95",
        "titlebar",
        "linear-gradient(90deg, #0a246a 0%, #a6caf0 100%)",
    ),
    (
        "winxp",
        "titlebar",
        "linear-gradient(180deg, #0997ff 0%, #0053ee 8%, #0050ee 40%, #0066ff 88%, #0054e3 100%)",
    ),
    (
        "winxp",
        "sidebar",
        "linear-gradient(180deg, #7ba2e7 0%, #6375d6 100%)",
    ),
    (
        "win7",
        "start-menu",
        "linear-gradient(180deg, #36627a 0%, #254658 48%, #38778e 100%)",
    ),
    (
        "mate",
        "titlebar",
        "linear-gradient(180deg, #eeeeee 0%, #c5c5c5 100%)",
    ),
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design;
    use iced_adaptive_design::theme_config::ThemeConfig;
    #[test]
    fn native_theme_compatibility_and_app_scene_schemas_are_separate() {
        use iced_adaptive_design::theme_config::Schema;
        for text in [
            include_str!("../../themes/win95.ron"),
            include_str!("../../themes/winxp.ron"),
            include_str!("../../themes/win7.ron"),
            include_str!("../../themes/mate.ron"),
            include_str!("../../themes/gnome2.ron"),
        ] {
            let report = Schema::toolkit().validate(text);
            assert!(report.errors.is_empty(), "{:?}", report.errors);
            assert!(report.unknown.is_empty(), "{:?}", report.unknown);
        }
        for index in 0..TARGETS.len() {
            let config = design::resolve(index, "");
            assert!(config.warnings().is_empty(), "{:?}", config.warnings());
            assert!(config.unknown().is_empty(), "{:?}", config.unknown());
            assert!(design::layout_fits(&config, index));
        }
        for &(section, key, default) in design::GEOMETRY {
            let invalid = format!("{{ {section:?}: {{ {key:?}: -1 }} }}");
            let config = ThemeConfig::builder("invalid-geometry-test")
                .files(false)
                .schema(design::schema())
                .app_text(invalid)
                .build();
            assert!(!config.warnings().is_empty(), "{section}.{key}");
            assert_eq!(config.number(section, key), Some(default));
        }
    }

    #[test]
    fn editor_menu_overlays_preserve_each_independent_base_even_when_geometry_is_rejected() {
        use iced_adaptive_design::theme_config::Source;
        for (index, reference) in TARGETS.iter().enumerate() {
            let base = design::resolve(index, "");
            let width = base.number("menu", "popover-width").unwrap()
                + base.number("metrics", "space-m").unwrap();
            let overlay = format!(
                "\"menu\": {{ \"popover-width\": {width} }}, \"colors\": {{ \"popover-bg\": \"#123456\" }}"
            );
            for text in [
                format!("{{ {overlay} }}"),
                format!(
                    "{{ {overlay}, \"toy.{}\": {{ \"width\": 1 }} }}",
                    reference.key
                ),
            ] {
                let config = design::resolve(index, &text);
                assert_eq!(config.text("theme", "key"), Some(reference.key));
                assert_eq!(config.number("menu", "popover-width"), Some(width));
                assert_eq!(config.source("menu", "popover-width"), Source::App);
                assert_eq!(config.source("colors", "popover-bg"), Source::App);
                assert_eq!(
                    config.get("typography", "base-size"),
                    base.get("typography", "base-size")
                );
                assert!(design::layout_fits(&config, index));
            }
        }
    }

    #[test]
    fn reference_manifest_drives_shipped_layout_boundaries_and_font_sizes() {
        assert_eq!(VERSION, 1);
        for (index, reference) in TARGETS.iter().enumerate() {
            let config = design::resolve(index, "");
            assert!(
                config.warnings().is_empty(),
                "{}: {:?}",
                reference.key,
                config.warnings()
            );
            assert!(
                config.unknown().is_empty(),
                "{}: {:?}",
                reference.key,
                config.unknown()
            );
            for (_, surface, expected) in
                GRADIENTS.iter().filter(|(key, _, _)| *key == reference.key)
            {
                assert_eq!(
                    config.get("gradients", surface),
                    Some(&iced_adaptive_design::theme_config::Value::Gradient(
                        iced_adaptive_design::theme_config::Gradient::parse(expected).unwrap()
                    ))
                );
            }
            let section = format!("toy.{}", reference.key);
            assert_eq!(config.number(&section, "width"), Some(reference.logical.0));
            assert_eq!(config.number(&section, "height"), Some(reference.logical.1));
            assert_eq!(
                config.number("typography", "base-size"),
                Some(reference.body_px)
            );
            assert_eq!(
                design::size(index),
                iced::Size::new(reference.logical.0, reference.logical.1)
            );
            if reference.key == "win7" {
                let menu = config.number(&section, "menu-width").unwrap();
                let left = config.number(&section, "programs-width").unwrap();
                assert!((left / menu - 0.6104).abs() < 0.001);
                assert!(
                    config.number(&section, "menu-height").unwrap()
                        + config.number(&section, "taskbar-height").unwrap()
                        < reference.logical.1
                );
            }
            if reference.key == "mate" {
                for name in ["editor", "help"] {
                    let x = config.number(&section, &format!("{name}-x")).unwrap();
                    let y = config.number(&section, &format!("{name}-y")).unwrap();
                    let w = config.number(&section, &format!("{name}-width")).unwrap();
                    let h = config.number(&section, &format!("{name}-height")).unwrap();
                    assert!(x + w <= reference.logical.0 && y + h <= reference.logical.1 - 36.0);
                }
            }
        }
    }
}

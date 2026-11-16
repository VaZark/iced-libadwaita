use super::*;

#[path = "regression_tests.rs"]
mod regressions;
use std::os::unix::fs::symlink;

#[test]
fn geometry_sections_have_searchable_friendly_labels() {
    let mut e = editor();
    // Use the same schema path as toolkit additions, without duplicating their defaults.
    let schema = Schema::new()
        .number("tree", "level-offset", 20.0, "Indentation per level")
        .number("tree-row", "indent", 20.0, "Nested child indentation")
        .number("navigation", "sidebar-fraction", 0.25, "Sidebar share");
    e.specs.extend(schema.specs().iter().cloned());
    for (section, query) in [
        ("tree", "hierarchy"),
        ("tree-row", "nested settings"),
        ("navigation", "drawers"),
    ] {
        e.search = query.into();
        let specs: Vec<_> = e.specs.iter().filter(|s| s.section == section).collect();
        assert!(!specs.is_empty());
        assert!(specs.iter().all(|s| e.visible(s)));
        let grouped: usize = e
            .groups_of(section)
            .iter()
            .map(|(_, specs)| specs.len())
            .sum();
        assert_eq!(grouped, specs.len());
        e.modified_only = true;
        assert!(specs.iter().all(|s| !e.visible(s)));
        e.modified_only = false;
    }
}

#[test]
fn preview_geometry_uses_scheme_tokens_without_changing_editor_chrome() {
    let text = "[metrics]\nradius-m=7\n[dark:metrics]\nradius-m=9\n";
    let light = preview::Preview::build(text, false);
    let dark = preview::Preview::build(text, true);
    assert_eq!(light.metrics.radius_m, 7.0);
    assert_eq!(dark.metrics.radius_m, 9.0);
    let _ = light.view();
    let _ = dark.view();
}

#[test]
fn named_widget_variants_are_discoverable_separately_from_schemes() {
    let mut e = editor();
    for (widget, variant) in Schema::toolkit().variants_declared() {
        if !["tree", "tree-row", "navigation", "tabs", "toolbar"].contains(&widget.as_str()) {
            continue;
        }
        let section = format!("{widget}.{variant}");
        assert!(e.sections.contains(&section), "{section}");
        assert!(tokens::section_label(&section).contains("widget variant"));
        e.search = "widget variant".into();
        assert!(
            e.specs
                .iter()
                .filter(|s| s.section == section)
                .all(|s| e.visible(s))
        );
    }
}

#[test]
fn breakpoint_helper_adds_edits_removes_and_undoes_rules() {
    let mut e = editor();
    let _ = e.update(Msg::Rule("tabs".into(), 0, Some((None, None))));
    assert!(e.generate().contains("breakpoint-rules=xs:"));
    let _ = e.update(Msg::Rule(
        "tabs".into(),
        0,
        Some((Some("md".into()), Some("outline".into()))),
    ));
    assert!(e.generate().contains("breakpoint-rules=md:outline"));
    let _ = e.update(Msg::Rule("tabs".into(), 0, None));
    assert!(!e.generate().contains("md:outline"));
    let _ = e.update(Msg::Undo);
    assert!(e.generate().contains("md:outline"));
    let _ = e.view();
}

#[test]
fn invalid_breakpoint_rules_never_enter_values_or_save_from_loaded_files() {
    let mut e = editor();
    for rules in ["md:missing", "bad:default", "xs:default,xs:outline"] {
        let _ = e.update(Msg::Input(
            "tabs".into(),
            "breakpoint-rules".into(),
            rules.into(),
        ));
        assert!(!e.generate().contains(rules));
    }
    let dir = temp_dir("invalid-rules");
    let path = dir.join("theme.conf");
    e.values.insert(
        ("tabs".into(), "breakpoint-rules".into()),
        "md:missing".into(),
    );
    assert!(!e.save_to(path.clone()));
    assert!(!path.exists());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn preview_width_rules_use_local_tokens_and_inclusive_breakpoints() {
    let p = preview::Preview::build(
        "[tabs]\nbreakpoint-rules=xs:pills,md:outline\n[breakpoints]\nmd=800\n",
        false,
    );
    let boundary = p.scale().md;
    assert_eq!(p.selected("tabs", boundary - 1.0), "pills");
    assert_eq!(p.selected("tabs", boundary), "outline");
    let _ = p.view_at(boundary - 1.0);
    let _ = p.view_at(boundary);
}

#[test]
fn preview_tree_layout_uses_local_geometry_and_scheme_overrides() {
    use iced::advanced::{layout, renderer::Headless, widget::Tree};
    let renderer = iced::futures::executor::block_on(iced::Renderer::new(
        iced::Font::DEFAULT,
        iced::Pixels(14.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let source = "[tree.default]\nrow-height=47px\nlevel-offset=31px\n[dark:tree.default]\nrow-height=61px\n";
    for (dark, expected) in [(false, 47.0), (true, 61.0)] {
        let preview = preview::Preview::build(source, dark);
        let mut element = preview.tree_sample(800.0);
        let mut state = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut state,
            &renderer,
            &layout::Limits::new(iced::Size::ZERO, iced::Size::new(800.0, 1000.0)),
        );
        // Project, Sources, main.rs, theme.conf: all expanded in the actual sample.
        assert_eq!(node.size().height, expected * 4.0);
    }
}

#[test]
fn preview_navigation_resolves_all_density_dimensions_locally() {
    let source = "[navigation]\nbreakpoint-rules=xs:sm,md:lg\n[navigation.sm]\nrow-height=33px\nrow-padding=7px\ngap=3px\ninset=5px\n[navigation.lg]\nrow-height=63px\nrow-padding=17px\ngap=13px\ninset=15px\n[dark:navigation.lg]\nrow-height=71px\n";
    let light = preview::Preview::build(source, false);
    let dark = preview::Preview::build(source, true);
    let boundary = light.scale().md;
    assert_eq!(
        light.navigation_geometry(boundary - 1.0),
        preview::NavigationGeometry {
            row_height: 33.0,
            row_padding: 7.0,
            gap: 3.0,
            inset: 5.0,
        }
    );
    assert_eq!(
        light.navigation_geometry(boundary),
        preview::NavigationGeometry {
            row_height: 63.0,
            row_padding: 17.0,
            gap: 13.0,
            inset: 15.0,
        }
    );
    assert_eq!(dark.navigation_geometry(boundary).row_height, 71.0);
    // A second preview must not mutate the first one's tokens or geometry.
    assert_eq!(light.navigation_geometry(boundary).row_height, 63.0);
}

#[test]
fn tabs_and_toolbar_preview_use_local_rules_and_geometry_without_global_writes() {
    use iced::advanced::{layout, renderer::Headless, widget::Tree};
    let global_theme = iad::theme_config::current();
    let global_config = iad::config::current();
    let renderer = iced::futures::executor::block_on(iced::Renderer::new(
        iced::Font::DEFAULT,
        iced::Pixels(14.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let layout = |mut element: iad::Element<'_, Msg>| {
        let mut state = Tree::new(&element);
        element.as_widget_mut().layout(
            &mut state,
            &renderer,
            &layout::Limits::new(iced::Size::ZERO, iced::Size::new(800.0, 600.0)),
        )
    };
    fn has_height(node: &layout::Node, height: f32) -> bool {
        (node.size().height - height).abs() < 0.01
            || node.children().iter().any(|n| has_height(n, height))
    }
    let a = preview::Preview::build(
        "[tabs]\nbreakpoint-rules=xs:default,md:pills\n[tabs.default]\nindicator-height=17px\n[toolbar]\nbreakpoint-rules=xs:raised,md:flat\n[toolbar.raised]\nshadow-height=23px\n",
        false,
    );
    let b = preview::Preview::build(
        "[tabs.default]\nindicator-height=3px\n[toolbar]\nbreakpoint-rules=xs:raised\n[toolbar.raised]\nshadow-height=7px\n",
        false,
    );
    let narrow = a.scale().md - 1.0;
    let wide = a.scale().md;
    assert_eq!(a.selected("tabs", narrow), "default");
    assert_eq!(a.selected("tabs", wide), "pills");
    assert_eq!(a.selected("toolbar", narrow), "raised");
    assert_eq!(a.selected("toolbar", wide), "flat");
    let tabs_a = layout(a.tabs_sample(narrow));
    let tabs_b = layout(b.tabs_sample(narrow));
    assert!((tabs_a.size().height - tabs_b.size().height - 14.0).abs() < 0.01);
    assert!(has_height(&layout(a.toolbar_sample(narrow)), 23.0));
    assert!(has_height(&layout(b.toolbar_sample(narrow)), 7.0));
    assert!(!has_height(&layout(a.toolbar_sample(wide)), 23.0));
    assert!(!has_height(&layout(a.tabs_sample(wide)), 17.0));
    assert!(std::sync::Arc::ptr_eq(
        &global_theme,
        &iad::theme_config::current()
    ));
    assert!(std::sync::Arc::ptr_eq(
        &global_config,
        &iad::config::current()
    ));
}

#[test]
fn overrides_only_excludes_base_and_other_scheme_values_and_combines_filters() {
    let mut e = with_text(
        "[colors]\nwindow-bg=#112233\nwindow-fg=#ffffff\n[light:colors]\nwindow-bg=#112233\n[dark:colors]\nwindow-fg=#111111\n[dark:metrics]\nradius-m=9\n",
    );
    let visible = |e: &Editor, section, key| e.visible(e.spec(section, key).unwrap());
    let _ = e.update(Msg::Variant(Variant::Light));
    let _ = e.update(Msg::OverridesOnly(true));
    assert!(
        visible(&e, "colors", "window-bg"),
        "an explicit value equal to base is still an override"
    );
    assert!(
        !visible(&e, "colors", "window-fg"),
        "base is inherited, dark does not count"
    );
    assert!(!visible(&e, "metrics", "radius-m"));
    let _ = e.update(Msg::ModifiedOnly(true));
    assert!(visible(&e, "colors", "window-bg"));
    let _ = e.update(Msg::Search("radius".into()));
    assert!(!visible(&e, "colors", "window-bg"));
    let _ = e.update(Msg::Variant(Variant::Dark));
    assert!(e.overrides_only);
    assert!(visible(&e, "metrics", "radius-m"));
    let _ = e.update(Msg::Search(String::new()));
    assert!(visible(&e, "colors", "window-fg"));
    assert!(!visible(&e, "colors", "window-bg"));
    let _ = e.update(Msg::OverridesOnly(false));
    assert!(
        visible(&e, "colors", "window-bg"),
        "modified-only still includes base values"
    );
}

#[test]
fn resetting_an_override_updates_filter_results_without_removing_the_base() {
    let mut e = with_text("[colors]\nwindow-bg=#112233\n[dark:colors]\nwindow-bg=#445566\n");
    let _ = e.update(Msg::Section(tokens::ALL_SECTIONS.into()));
    let _ = e.update(Msg::Variant(Variant::Dark));
    let _ = e.update(Msg::OverridesOnly(true));
    assert_eq!(e.set_count("colors"), 1);
    let _ = e.update(Msg::Reset("colors".into(), "window-bg".into()));
    let spec = e.spec("colors", "window-bg").unwrap();
    assert!(!e.visible(spec));
    assert_eq!(e.shown(spec), ("#112233".into(), false));
    assert_eq!(e.set_count("colors"), 0);
    let _ = e.update(Msg::Undo);
    assert!(e.visible(e.spec("colors", "window-bg").unwrap()));
    assert!(e.overrides_only);
    assert_eq!(e.set_count(tokens::ALL_SECTIONS), 1);
}

#[test]
fn base_disables_and_clears_overrides_only() {
    let mut e = editor();
    let _ = e.update(Msg::OverridesOnly(true));
    assert!(!e.overrides_only);
    let _ = e.update(Msg::Variant(Variant::Dark));
    let _ = e.update(Msg::OverridesOnly(true));
    assert!(e.overrides_only);
    let _ = e.update(Msg::Variant(Variant::Base));
    assert!(!e.overrides_only);
    let _ = e.view();
}

#[test]
fn all_page_visits_each_schema_token_once_without_changing_saved_values() {
    let mut e = with_text("[colors]\nwindow-bg=#112233\n[dark:metrics]\nradius-m=9\n");
    let before = e.generate();
    assert_eq!(
        e.section_choices()
            .filter(|s| *s == tokens::ALL_SECTIONS)
            .count(),
        1
    );
    let scroll = e.update(Msg::Section(tokens::ALL_SECTIONS.into()));
    assert!(
        scroll.units() > 0,
        "All navigation resets the token scroll position"
    );
    let mut addresses = Vec::new();
    for section in e.page_sections() {
        for (_, specs) in e.groups_of(section) {
            for spec in specs {
                addresses.push((spec.section.clone(), spec.key.clone()));
            }
        }
    }
    let unique: BTreeSet<_> = addresses.iter().collect();
    assert_eq!(addresses.len(), e.specs.len());
    assert_eq!(unique.len(), addresses.len());
    assert_eq!(e.generate(), before);
    assert!(!e.sections.iter().any(|s| s == tokens::ALL_SECTIONS));
    let _ = e.view();
    let _ = e.update(Msg::Section("tabs".into()));
    assert_eq!(e.page_sections(), vec!["tabs"]);
    let _ = e.update(Msg::Search("radius".into()));
    assert_eq!(e.effective_section(), tokens::ALL_SECTIONS);
    assert_eq!(e.page_sections().len(), e.sections.len());
    let _ = e.update(Msg::Search(String::new()));
    assert_eq!(e.effective_section(), "tabs");
    let _ = e.update(Msg::Section("not-a-section".into()));
    assert_eq!(e.section, "tabs");
}

#[test]
fn all_search_and_override_filters_share_the_same_single_page() {
    let mut e = with_text(
        "[metrics]\nradius-m=7\n[dark:metrics]\nradius-m=9\n[dark:colors]\nwindow-bg=#112233\n",
    );
    let _ = e.update(Msg::Section(tokens::ALL_SECTIONS.into()));
    let _ = e.update(Msg::Variant(Variant::Dark));
    let _ = e.update(Msg::OverridesOnly(true));
    let _ = e.update(Msg::ModifiedOnly(true));
    let scroll = e.update(Msg::Search("radius-m".into()));
    assert!(scroll.units() > 0);
    let shown: Vec<_> = e
        .page_sections()
        .into_iter()
        .flat_map(|section| e.groups_of(section))
        .flat_map(|(_, specs)| specs)
        .filter(|spec| e.visible(spec))
        .collect();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].section, "metrics");
    assert_eq!(shown[0].key, "radius-m");
    assert_eq!(e.set_count(tokens::ALL_SECTIONS), 2);
    let _ = e.page(iad::Metrics::desktop());
}

#[test]
fn all_navigation_is_remembered_across_runs() {
    let d = temp_dir("all-section-state");
    let state = d.join("state.conf");
    let mut e = Editor::with(Some(state.clone()), None).0;
    let _ = e.update(Msg::Section(tokens::ALL_SECTIONS.into()));
    let restored = Editor::with(Some(state), None).0;
    assert_eq!(restored.section, tokens::ALL_SECTIONS);
    assert_eq!(restored.page_sections().len(), restored.sections.len());
    std::fs::remove_dir_all(d).unwrap();
}

fn editor() -> Editor {
    let mut e = Editor::with(None, None).0;
    // Tests never look at, or write into, the real data dirs.
    e.theme_dirs = Vec::new();
    e.presets.clear();
    e
}

fn with_text(text: &str) -> Editor {
    let mut e = editor();
    e.values = iad::config::parse_keyfile(text).into_iter().collect();
    e.clean = e.generate();
    e.refresh();
    e
}

fn temp_dir(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let d = std::env::temp_dir().join(format!(
        "iad-editor-test-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn read(p: &std::path::Path) -> String {
    std::fs::read_to_string(p).unwrap()
}

#[test]
fn the_shipped_themes_round_trip_through_the_editor() {
    let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../themes"));
    let files = scan_folder(&dir);
    assert!(files.len() >= 6);
    for f in files {
        let text = read(&f);
        let e = with_text(&text);
        let out = e.generate();
        let a: BTreeMap<_, _> = iad::config::parse_keyfile(&text).into_iter().collect();
        let b: BTreeMap<_, _> = iad::config::parse_keyfile(&out).into_iter().collect();
        assert_eq!(a, b, "{}", f.display());
        let report = Schema::toolkit().validate(&out);
        assert!(
            report.errors.is_empty() && report.unknown.is_empty(),
            "{}",
            f.display()
        );
    }
}

#[test]
fn only_what_is_set_is_written_and_a_variant_goes_in_its_own_section() {
    let mut e = editor();
    assert!(!e.generate().lines().any(|l| l.starts_with('[')));
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::Variant(Variant::Dark));
    let _ = e.update(Msg::Input(
        "colors".into(),
        "window-bg".into(),
        "#101010".into(),
    ));
    let out = e.generate();
    assert!(out.contains("[metrics]\nradius-m=7\n"));
    assert!(out.contains("[dark:colors]\nwindow-bg=#101010\n"));
    assert!(!out.contains("[colors]"));
    // An unset dark value shows the base's, and reset returns to it.
    let _ = e.update(Msg::Reset("colors".into(), "window-bg".into()));
    assert!(!e.generate().lines().any(|l| l.starts_with("[dark:")));
}

#[test]
fn a_value_that_is_not_valid_yet_is_not_written() {
    let mut e = editor();
    let _ = e.update(Msg::Input(
        "colors".into(),
        "window-bg".into(),
        "#12".into(),
    ));
    assert!(!e.generate().contains("window-bg"));
    let _ = e.update(Msg::Input(
        "windows".into(),
        "button-size".into(),
        "30".into(),
    ));
    assert!(e.generate().contains("button-size=30px"));
}

#[test]
fn unknown_keys_are_kept() {
    let e = with_text("[player]\nartwork=disc\n[colors]\naccent=#112233\n");
    let out = e.generate();
    assert!(out.contains("[player]\nartwork=disc\n"));
    assert!(out.contains("accent=#112233"));
}

// ---------------------------------------------------------- the dot

#[test]
fn the_unsaved_dot_shows_only_when_there_are_unsaved_changes() {
    let d = temp_dir("dot");
    let f = d.join("t.conf");
    std::fs::write(&f, "[metrics]\nradius-m=1\n").unwrap();
    let mut e = editor();
    e.load(f.clone());
    assert!(!e.dirty());
    assert_eq!(e.dot(), "");
    assert!(!e.title().contains('●') && e.title().starts_with("t.conf"));
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "9".into()));
    assert_eq!(e.dot(), "● ");
    assert!(e.title().starts_with("● t.conf"));
    let _ = e.update(Msg::SavedAs(Some(f.clone())));
    assert_eq!(e.dot(), "");
    assert!(!e.title().contains('●'));
    let _ = std::fs::remove_dir_all(d);
}

// ------------------------------------------------------------ saving

#[test]
fn save_upserts_the_original_file() {
    let d = temp_dir("save");
    let f = d.join("mine.conf");
    let mut e = editor();
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::SavedAs(Some(f.clone())));
    // Created.
    assert!(read(&f).contains("[metrics]\nradius-m=7\n"));
    assert!(!e.dirty());
    // Replaced, by Save, at the same path.
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "8".into()));
    assert!(e.dirty());
    let _ = e.update(Msg::Save);
    assert!(read(&f).contains("radius-m=8") && !read(&f).contains("radius-m=7"));
    assert_eq!(e.path, Some(f));
    assert!(!e.dirty());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn save_writes_through_a_symlink_to_its_target() {
    let d = temp_dir("through");
    let real = d.join("real.conf");
    let link = d.join("link.conf");
    std::fs::write(&real, "[metrics]\nradius-m=1\n").unwrap();
    symlink(&real, &link).unwrap();
    let mut e = editor();
    e.load(link.clone());
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "5".into()));
    let _ = e.update(Msg::Save);
    assert!(is_link(&link), "the link is still a link");
    assert!(read(&real).contains("radius-m=5"));
    assert!(!e.dirty());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn save_without_a_path_asks_where_instead_of_writing() {
    let mut e = editor();
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    assert!(e.path.is_none());
    let task = e.update(Msg::Save);
    assert!(task.units() > 0, "Save as opens the file dialog");
    assert!(e.path.is_none() && e.dirty());
}

#[test]
fn launch_choices_are_independent_of_scheme_token_editing() {
    let mut e = editor();
    assert_eq!(e.launch_scheme, toys::LaunchScheme::System);
    assert_eq!(e.launch_target, "gallery");
    let _ = e.update(Msg::LaunchTarget("win95".into()));
    let _ = e.update(Msg::LaunchScheme(toys::LaunchScheme::Dark));
    let _ = e.update(Msg::Variant(Variant::Light));
    assert_eq!(e.launch_target, "win95");
    assert_eq!(e.launch_scheme, toys::LaunchScheme::Dark);
    assert!(
        !e.runner.running("win95"),
        "choosing an action must not launch it"
    );
    let _ = e.update(Msg::ThemePicker(true));
    let _ = e.view();
    let _ = e.update(Msg::ThemePicker(false));
    let _ = e.update(Msg::Section("tabs".into()));
    assert_eq!(e.section, "tabs");
    assert!(!e.drawer);
}

#[test]
fn theme_picker_bounds_and_wraps_long_names_and_paths_at_narrow_widths() {
    use iced::advanced::{layout, renderer::Headless, widget::Tree};
    let renderer = iced::futures::executor::block_on(iced::Renderer::new(
        iced::Font::DEFAULT,
        iced::Pixels(14.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let mut e = editor();
    e.presets = vec![Preset {
        group: "Folders",
        name: "夜空のテーマ العربية Ελληνικά हिन्दी café 🎨 AnUnbrokenVeryLongThemeName".repeat(16),
        path: PathBuf::from(format!("/themes/{}.conf", "long-directory-name".repeat(24))),
        target: Some(PathBuf::from(format!(
            "/linked/{}.conf",
            "another-long-name".repeat(24)
        ))),
        is_default: true,
    }];
    fn assert_horizontal_bounds(node: &layout::Node, parent_x: f32, width: f32) {
        let bounds = node.bounds();
        let x = parent_x + bounds.x;
        assert!(
            x >= -0.1 && x + bounds.width <= width + 0.1,
            "picker child overflows: x={x}, width={}, viewport={width}",
            bounds.width
        );
        for child in node.children() {
            assert_horizontal_bounds(child, x, width);
        }
    }
    e.path = Some(e.presets[0].path.clone());
    let short = Preset {
        group: "Folders",
        name: "Theme".into(),
        path: "/themes/theme.conf".into(),
        target: None,
        is_default: false,
    };
    for width in [320.0, 480.0, 760.0] {
        let props = crate::view::theme_item_props(width);
        assert_eq!(
            props.label.overflow,
            Some(iad::components::TextOverflow::Wrap)
        );
        assert_eq!(props.container.max_width.unwrap().0, width);
        let measure = |mut element: iad::Element<'_, Msg>| {
            let mut state = Tree::new(&element);
            element.as_widget_mut().layout(
                &mut state,
                &renderer,
                &layout::Limits::new(iced::Size::ZERO, iced::Size::new(width, f32::INFINITY)),
            )
        };
        let selected_long = measure(e.theme_selector(width));
        let mut short_editor = editor();
        short_editor.path = Some(short.path.clone());
        short_editor.presets = vec![short.clone()];
        let selected_short = measure(short_editor.theme_selector(width));
        assert_horizontal_bounds(&selected_long, 0.0, width);
        assert!(
            selected_long.size().height > selected_short.size().height,
            "selected multilingual theme must wrap, not truncate or overflow"
        );
        let long_layout = measure(e.theme_option(&e.presets[0], width));
        let short_layout = measure(e.theme_option(&short, width));
        assert_horizontal_bounds(&long_layout, 0.0, width);
        assert!(
            long_layout.size().height > short_layout.size().height,
            "multilingual option must grow vertically rather than overflow or truncate"
        );
        let size = iced::Size::new(width, 480.0);
        let mut element = e.theme_picker_at(size);
        let mut state = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut state,
            &renderer,
            &layout::Limits::new(iced::Size::ZERO, size),
        );
        assert_horizontal_bounds(&node, 0.0, width);
        assert!(node.size().height <= size.height);
    }
}

#[test]
fn theme_picker_keeps_unsaved_confirmation_accessible() {
    let d = temp_dir("picker-confirm");
    let target = d.join("other.conf");
    std::fs::write(&target, "[metrics]\nradius-m=2\n").unwrap();
    let mut e = editor();
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::ThemePicker(true));
    let _ = e.update(Msg::Pick(target.clone()));
    assert!(e.theme_picker && e.pending.is_some() && e.dirty());
    let _ = e.update(Msg::Pick(target.clone()));
    assert!(!e.theme_picker && e.pending.is_none());
    assert_eq!(e.path, Some(target));
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn launch_scheme_overwrites_or_removes_preexisting_child_overrides() {
    for scheme in toys::LaunchScheme::ALL {
        let mut command = std::process::Command::new("sh");
        command
            .env("TOYS_SCHEME", "dark")
            .env("GALLERY_SCHEME", "light");
        scheme.apply_environment(&mut command);
        let environment: BTreeMap<_, _> = command.get_envs().collect();
        let expected = match scheme {
            toys::LaunchScheme::System => None,
            toys::LaunchScheme::Light => Some(std::ffi::OsStr::new("light")),
            toys::LaunchScheme::Dark => Some(std::ffi::OsStr::new("dark")),
        };
        for key in ["TOYS_SCHEME", "GALLERY_SCHEME"] {
            assert_eq!(environment[std::ffi::OsStr::new(key)], expected);
        }
    }
}

#[test]
fn gallery_and_toys_receive_launch_scheme_only_in_child_environment() {
    let d = temp_dir("launch-schemes");
    let inherited_toys = std::env::var_os("TOYS_SCHEME");
    let inherited_gallery = std::env::var_os("GALLERY_SCHEME");
    let mut runner = toys::Runner::default();
    for kind in ["gallery", "win95"] {
        for scheme in toys::LaunchScheme::ALL {
            let output = d.join(format!("{kind}-{}", scheme.label()));
            let bin = script(
                &d,
                "capture-scheme",
                &format!(
                    "printf 'toys=%s\\ngallery=%s\\n' \"${{TOYS_SCHEME-unset}}\" \"${{GALLERY_SCHEME-unset}}\" > {0}.tmp\nif [ -n \"${{ICED_ADAPTIVE_DESIGN_CONFIG-}}\" ]; then cat \"$ICED_ADAPTIVE_DESIGN_CONFIG\" >> {0}.tmp; fi\nmv {0}.tmp {0}",
                    output.display()
                ),
            );
            let bins = toys::Bins {
                gallery: Some(bin.clone()),
                toys: Some(bin),
            };
            runner
                .launch_with_scheme(kind, "[metrics]\nradius-m=7\n", &bins, scheme)
                .unwrap();
            let seen = wait_for(&output);
            match scheme {
                toys::LaunchScheme::System => {
                    assert!(seen.starts_with("toys=unset\ngallery=unset\n"))
                }
                toys::LaunchScheme::Light => assert!(
                    seen.starts_with("toys=light\ngallery=light\n")
                        && seen.contains("color-scheme=light")
                ),
                toys::LaunchScheme::Dark => assert!(
                    seen.starts_with("toys=dark\ngallery=dark\n")
                        && seen.contains("color-scheme=dark")
                ),
            }
        }
    }
    assert_eq!(std::env::var_os("TOYS_SCHEME"), inherited_toys);
    assert_eq!(std::env::var_os("GALLERY_SCHEME"), inherited_gallery);
    drop(runner);
    std::fs::remove_dir_all(d).unwrap();
}

// -------------------------------------------------------- try it out

/// An executable shell script in `dir`.
fn script(dir: &std::path::Path, name: &str, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

/// Waits for a file a started program writes.
fn wait_for(path: &std::path::Path) -> String {
    for _ in 0..100 {
        if let Ok(t) = std::fs::read_to_string(path) {
            if !t.is_empty() {
                return t;
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(30));
    }
    panic!("{} never appeared", path.display());
}

#[test]
fn try_it_out_runs_the_gallery_on_a_temp_file_and_leaves_the_original_alone() {
    let d = temp_dir("try");
    let f = d.join("orig.conf");
    std::fs::write(&f, "[metrics]\nradius-m=1\n").unwrap();
    let mut e = editor();
    e.load(f.clone());
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let out = d.join("gallery.out");
    e.bins.gallery = Some(script(
        &d,
        "gallery.sh",
        &format!(
            "echo \"$IAD_THEME\" > {o}.tmp; cat \"$IAD_THEME\" >> {o}.tmp; mv {o}.tmp {o}",
            o = out.display()
        ),
    ));
    e.try_out("gallery");
    let seen = wait_for(&out);
    assert!(seen.contains("radius-m=7"), "the unsaved value: {seen}");
    assert!(seen.contains("iad-theme-editor-"), "a temp file of its own");
    assert_eq!(
        read(&f),
        "[metrics]\nradius-m=1\n",
        "the original is untouched"
    );
    assert!(e.dirty() && e.path == Some(f));
    assert!(e.status.starts_with("Tried it out"), "{}", e.status);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_toy_window_starts_with_its_name_and_the_unsaved_values() {
    let d = temp_dir("toy");
    let mut e = editor();
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "3".into()));
    let out = d.join("toy.out");
    e.bins.toys = Some(script(
        &d,
        "theme-toys.sh",
        &format!(
            "echo \"$@\" > {o}.tmp; cat \"$IAD_THEME\" >> {o}.tmp; mv {o}.tmp {o}",
            o = out.display()
        ),
    ));
    let _ = e.update(Msg::TryOut("win95".into()));
    let seen = wait_for(&out);
    assert!(seen.starts_with("win95"), "{seen}");
    assert!(seen.contains("radius-m=3"), "{seen}");
    assert!(e.dirty() && e.path.is_none(), "nothing was saved");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn starting_a_toy_again_replaces_the_old_window() {
    let d = temp_dir("again");
    let mut runner = toys::Runner::default();
    let bins = toys::Bins {
        gallery: None,
        toys: Some(script(&d, "t.sh", "sleep 30")),
    };
    runner.launch("again", "", &bins).unwrap();
    assert!(runner.running("again"));
    runner
        .launch("again", "[metrics]\nradius-m=1\n", &bins)
        .unwrap();
    assert!(runner.running("again"), "the new one runs");
    assert!(!runner.running("other"), "other kinds are not started");
    drop(runner);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_toy_list_normalizes_stale_installs_or_uses_the_canonical_four() {
    let d = temp_dir("list");
    let bins = toys::Bins {
        gallery: None,
        toys: Some(script(
            &d,
            "t.sh",
            "printf 'win95  Windows 95\\nmac: Mac OS\\n# a comment\\ngnome2\\nmate: Stale label\\nunknown: Unknown\\nwin95: Duplicate\\n'",
        )),
    };
    let toys = toys::list_toys(&bins);
    assert_eq!(
        toys.iter()
            .map(|t| (t.name.as_str(), t.label.as_str()))
            .collect::<Vec<_>>(),
        [("win95", "Windows 95"), ("mate", "MATE")]
    );
    let fixed = toys::list_toys(&toys::Bins::default());
    assert_eq!(
        fixed.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(),
        ["win95", "winxp", "win7", "mate"]
    );
    let broken = toys::Bins {
        gallery: None,
        toys: Some(script(&d, "bad.sh", "exit 3")),
    };
    assert_eq!(toys::list_toys(&broken), toys::fallback());
    for (name, output) in [
        ("removed", "mac: Mac OS"),
        ("empty", ""),
        ("invalid", "not/a/target"),
    ] {
        let bins = toys::Bins {
            gallery: None,
            toys: Some(script(&d, name, &format!("printf '%s' '{output}'"))),
        };
        assert_eq!(toys::list_toys(&bins), toys::fallback());
    }
    let alias = toys::Bins {
        gallery: None,
        toys: Some(script(&d, "alias", "printf 'gnome2: GNOME 2'")),
    };
    assert_eq!(toys::list_toys(&alias)[0].name, "mate");
    assert_eq!(toys::list_toys(&alias)[0].label, "MATE");
    let _ = std::fs::remove_dir_all(d);
}

// ------------------------------------------------------- registering

#[test]
fn register_makes_a_symlink_and_replaces_an_old_one() {
    let d = temp_dir("reg");
    let (data, other) = (d.join("data"), d.join("else"));
    std::fs::create_dir_all(&other).unwrap();
    let a = other.join("mine.conf");
    let b = other.join("sub");
    std::fs::create_dir_all(&b).unwrap();
    let b = b.join("mine.conf");
    std::fs::write(&a, "").unwrap();
    std::fs::write(&b, "").unwrap();
    assert!(!is_registered(&data, &a));
    let link = register(&data, &a).unwrap();
    assert_eq!(link, data.join("mine.conf"));
    assert!(is_link(&link) && canon(&link) == canon(&a));
    assert!(is_registered(&data, &a));
    assert_eq!(register(&data, &a), Err("already registered".to_string()));
    // Another file with the same name replaces the old symlink.
    register(&data, &b).unwrap();
    assert_eq!(canon(&data.join("mine.conf")), canon(&b));
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn register_never_overwrites_a_regular_file() {
    let d = temp_dir("regfile");
    let data = d.join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("mine.conf"), "keep me").unwrap();
    let file = d.join("mine.conf");
    std::fs::write(&file, "").unwrap();
    let err = register(&data, &file).unwrap_err();
    assert!(err.contains("regular file"), "{err}");
    assert_eq!(read(&data.join("mine.conf")), "keep me");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn unregister_removes_only_symlinks() {
    let d = temp_dir("unreg");
    let data = d.join("data");
    std::fs::create_dir_all(&data).unwrap();
    let file = d.join("mine.conf");
    std::fs::write(&file, "x").unwrap();
    register(&data, &file).unwrap();
    assert_eq!(unregister(&data, &file), Ok(1));
    assert!(!data.join("mine.conf").exists() && file.exists());
    assert!(unregister(&data, &file).is_err());
    // A regular file inside the data dir is registered but has no symlink to remove.
    let inside = data.join("own.conf");
    std::fs::write(&inside, "x").unwrap();
    assert!(is_registered(&data, &inside));
    assert!(unregister(&data, &inside).is_err());
    assert!(inside.exists());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn set_as_default_points_default_conf_at_the_file_and_the_list_tags_it() {
    let d = temp_dir("default");
    let data = d.join("data");
    std::fs::create_dir_all(&data).unwrap();
    let (a, b) = (d.join("a.conf"), d.join("b.conf"));
    std::fs::write(&a, "[theme]\nname=Alpha\nkey=a\n").unwrap();
    std::fs::write(&b, "[theme]\nname=Beta\nkey=b\n").unwrap();
    register(&data, &a).unwrap();
    register(&data, &b).unwrap();
    set_default(&data, &a).unwrap();
    assert_eq!(default_target(&data), Some(canon(&a)));
    let list = list_presets(&[data.clone()], &[], &[]);
    assert_eq!(list.len(), 2, "default.conf is not a third entry");
    let tagged: Vec<_> = list
        .iter()
        .filter(|p| p.is_default)
        .map(|p| p.name.as_str())
        .collect();
    assert_eq!(tagged, ["Alpha"]);
    // Replaced by another file.
    set_default(&data, &b).unwrap();
    let list = list_presets(&[data.clone()], &[], &[]);
    assert_eq!(
        list.iter()
            .filter(|p| p.is_default)
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>(),
        ["Beta"]
    );
    // A regular default.conf is not overwritten.
    let other = temp_dir("default2");
    std::fs::write(other.join("default.conf"), "mine").unwrap();
    assert!(
        set_default(&other, &a)
            .unwrap_err()
            .contains("regular file")
    );
    assert_eq!(read(&other.join("default.conf")), "mine");
    let _ = std::fs::remove_dir_all(d);
    let _ = std::fs::remove_dir_all(other);
}

#[test]
fn a_default_that_is_not_registered_still_shows_up_tagged() {
    let d = temp_dir("loosedefault");
    let data = d.join("data");
    std::fs::create_dir_all(&data).unwrap();
    let f = d.join("far.conf");
    std::fs::write(&f, "").unwrap();
    set_default(&data, &f).unwrap();
    let list = list_presets(&[data], &[], &[]);
    assert_eq!(list.len(), 1);
    assert!(list[0].is_default && list[0].group == "Registered");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_registry_buttons_report_in_the_status_line_and_rescan() {
    let d = temp_dir("buttons");
    let data = d.join("data");
    let f = d.join("t.conf");
    std::fs::write(&f, "[theme]\nname=T\nkey=t\n").unwrap();
    let mut e = editor();
    e.theme_dirs = vec![data.clone()];
    let _ = e.update(Msg::Register);
    assert!(e.status.contains("Save the theme"), "no file yet");
    e.load(f.clone());
    let _ = e.update(Msg::Register);
    assert!(e.status.starts_with("Registered"), "{}", e.status);
    assert!(
        e.presets
            .iter()
            .any(|p| p.group == "Registered" && p.name == "T")
    );
    let _ = e.update(Msg::SetDefault);
    assert!(e.presets.iter().any(|p| p.is_default));
    let _ = e.update(Msg::Unregister);
    assert!(!is_registered(&data, &f));
    let _ = std::fs::remove_dir_all(d);
}

// ------------------------------------------------------------- list

#[test]
fn a_clean_environment_lists_nothing() {
    let d = temp_dir("empty");
    assert!(list_presets(&[d.clone()], &[], &[]).is_empty());
    assert!(editor().presets.is_empty());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_list_has_the_registered_files_the_folders_and_the_recent_ones() {
    let d = temp_dir("list");
    let data = d.join("data");
    let folder = d.join("folder");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("mine.conf"), "[theme]\nname=Mine\nkey=mine\n").unwrap();
    std::fs::write(folder.join("plain.conf"), "").unwrap();
    std::fs::write(data.join("own.conf"), "[theme]\nname=Own\nkey=own\n").unwrap();
    let far = temp_dir("recent").join("far.conf");
    std::fs::write(&far, "").unwrap();
    register(&data, &folder.join("plain.conf")).unwrap();
    let list = list_presets(
        &[data.clone()],
        &[folder.clone()],
        &[far.clone(), folder.join("mine.conf"), d.join("gone.conf")],
    );
    let group = |n: &str| list.iter().find(|p| p.name == n).map(|p| p.group);
    assert_eq!(group("Own"), Some("Registered"));
    assert_eq!(
        group("plain"),
        Some("Registered"),
        "registered, so not repeated under Folders"
    );
    assert_eq!(group("Mine"), Some("Folders"));
    assert_eq!(group("far"), Some("Recent"));
    assert_eq!(list.iter().filter(|p| p.name == "Mine").count(), 1);
    assert!(
        list.iter().all(|p| p.path.is_file()),
        "only real files, none missing"
    );
    // The symlink entry shows where it points.
    let plain = list.iter().find(|p| p.name == "plain").unwrap();
    assert_eq!(plain.target, Some(canon(&folder.join("plain.conf"))));
    assert_eq!(list.iter().find(|p| p.name == "Own").unwrap().target, None);
    // No embedded, shipped or repository group exists.
    assert!(
        list.iter()
            .all(|p| ["Registered", "Folders", "Recent"].contains(&p.group))
    );
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_folder_scan_lists_only_conf_files_by_name() {
    let d = temp_dir("scan");
    for f in ["b.conf", "a.conf", "notes.txt", "c.conf.bak"] {
        std::fs::write(d.join(f), "").unwrap();
    }
    std::fs::create_dir(d.join("sub.conf")).unwrap();
    symlink(d.join("nowhere.conf"), d.join("broken.conf")).unwrap();
    let names: Vec<_> = scan_folder(&d)
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["a.conf", "b.conf"]);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn switching_with_unsaved_changes_needs_a_second_click() {
    let d = temp_dir("switch");
    let (a, b) = (d.join("a.conf"), d.join("b.conf"));
    std::fs::write(&a, "[metrics]\nradius-m=1\n").unwrap();
    std::fs::write(&b, "[metrics]\nradius-m=2\n").unwrap();
    let mut e = editor();
    e.pick(Target::File(a.clone()));
    assert_eq!(e.path, Some(a.clone()));
    assert!(!e.dirty());
    // Clean: switching is at once.
    e.pick(Target::File(b.clone()));
    assert_eq!(e.path, Some(b.clone()));
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "9".into()));
    assert!(e.dirty());
    e.pick(Target::File(a.clone()));
    assert_eq!(e.path, Some(b.clone()), "the first click only warns");
    assert!(e.status.contains("click again"));
    assert_eq!(e.values[&("metrics".into(), "radius-m".into())], "9");
    e.pick(Target::File(a.clone()));
    assert_eq!(e.path, Some(a.clone()));
    assert_eq!(e.values[&("metrics".into(), "radius-m".into())], "1");
    // New is a switch too: it starts empty, with no path, and is not dirty.
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "4".into()));
    let _ = e.update(Msg::New);
    assert_eq!(e.path, Some(a.clone()), "the first click only warns");
    let _ = e.update(Msg::New);
    assert!(e.path.is_none() && e.values.is_empty() && !e.dirty());
    let _ = std::fs::remove_dir_all(d);
}

// ------------------------------------------------------------ state

#[test]
fn the_state_round_trips() {
    let state = State {
        last_file: Some("/a/one.conf".into()),
        folders: vec!["/x".into(), "/y".into()],
        recent: vec!["/a/one.conf".into(), "/b/two.conf".into()],
        section: Some("metrics".into()),
        variant: Some(Variant::Dark),
    };
    assert_eq!(State::parse(&state.to_text()), state);
    assert_eq!(State::parse(""), State::default());
    assert!(
        state
            .to_text()
            .starts_with("[state]\nlast-file=/a/one.conf\n")
    );
}

#[test]
fn the_recent_list_has_no_duplicates_and_a_limit() {
    let mut s = State::default();
    for i in 0..12 {
        s.remember(std::path::Path::new(&format!("/t/{i}.conf")));
    }
    s.remember(std::path::Path::new("/t/5.conf"));
    assert_eq!(s.recent.len(), RECENT_MAX);
    assert_eq!(s.recent[0], PathBuf::from("/t/5.conf"));
    assert_eq!(
        s.recent
            .iter()
            .filter(|p| **p == PathBuf::from("/t/5.conf"))
            .count(),
        1
    );
}

#[test]
fn the_open_file_section_and_variant_are_remembered_across_runs() {
    let d = temp_dir("state");
    let state = d.join("state.conf");
    let theme = d.join("t.conf");
    std::fs::write(&theme, "[theme]\nname=T\nkey=t\n").unwrap();
    let (mut e, _) = Editor::with(Some(state.clone()), None);
    e.theme_dirs = Vec::new();
    let _ = e.update(Msg::Pick(theme.clone()));
    let _ = e.update(Msg::Section("metrics".into()));
    let _ = e.update(Msg::Variant(Variant::Light));
    let _ = e.update(Msg::FolderAdded(Some(d.clone())));
    let (again, _) = Editor::with(Some(state.clone()), None);
    assert_eq!(again.path, Some(theme.clone()), "reopens the last file");
    assert_eq!(again.section, "metrics");
    assert_eq!(again.variant, Variant::Light);
    assert_eq!(again.state.folders, vec![d.clone()]);
    assert!(again.presets.iter().any(|p| p.path == theme));
    // A file on the command line wins over the last one.
    let other = d.join("o.conf");
    std::fs::write(&other, "").unwrap();
    let (cli, _) = Editor::with(Some(state), Some(other.clone()));
    assert_eq!(cli.path, Some(other));
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn saving_remembers_the_file_as_last_and_recent() {
    let d = temp_dir("remember");
    let state = d.join("state.conf");
    let (mut e, _) = Editor::with(Some(state.clone()), None);
    e.theme_dirs = Vec::new();
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let file = d.join("mine.conf");
    let _ = e.update(Msg::SavedAs(Some(file.clone())));
    assert_eq!(e.state.last_file, Some(file.clone()));
    assert_eq!(e.state.recent.first(), Some(&file));
    assert!(read(&state).contains("mine.conf"));
    assert!(!read(&state).contains("embedded"));
    let _ = std::fs::remove_dir_all(d);
}

// ---------------------------------------------------------- save as + register

#[test]
fn save_as_writes_a_new_file_and_registers_it() {
    let d = temp_dir("saveas");
    let data = d.join("data");
    let file = d.join("elsewhere").join("mine.conf");
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let mut e = editor();
    e.theme_dirs = vec![data.clone()];
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::SavedAs(Some(file.clone())));
    assert!(
        read(&file).contains("radius-m=7"),
        "the new file is written"
    );
    assert!(
        !e.dirty() && e.path == Some(file.clone()),
        "the editor edits the new file"
    );
    let link = data.join("mine.conf");
    assert!(
        is_link(&link) && canon(&link) == canon(&file),
        "registered by a symlink"
    );
    assert!(
        e.status.starts_with("Saved and registered as"),
        "{}",
        e.status
    );
    assert!(
        e.presets.iter().any(|p| p.path == link),
        "the list shows it"
    );
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn save_as_inside_the_registry_needs_no_symlink() {
    let d = temp_dir("saveas-in");
    let data = d.join("data");
    std::fs::create_dir_all(&data).unwrap();
    let file = data.join("inside.conf");
    let mut e = editor();
    e.theme_dirs = vec![data.clone()];
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::SavedAs(Some(file.clone())));
    assert!(!is_link(&file) && file.is_file());
    assert!(e.status.contains("registered already"), "{}", e.status);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn save_as_does_not_overwrite_a_regular_file_in_the_registry_and_says_so() {
    let d = temp_dir("saveas-clash");
    let data = d.join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("mine.conf"), "[metrics]\nradius-m=1\n").unwrap();
    let file = d.join("mine.conf");
    let mut e = editor();
    e.theme_dirs = vec![data.clone()];
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::SavedAs(Some(file.clone())));
    assert!(file.is_file(), "the file is saved anyway");
    assert_eq!(read(&data.join("mine.conf")), "[metrics]\nradius-m=1\n");
    assert!(e.status.contains("could not register"), "{}", e.status);
    assert!(e.toast.as_ref().is_some_and(|t| t.error));
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn save_only_updates_the_file_and_registers_nothing() {
    let d = temp_dir("save-only");
    let data = d.join("data");
    let file = d.join("a.conf");
    std::fs::write(&file, "").unwrap();
    let mut e = editor();
    e.theme_dirs = vec![data.clone()];
    e.load(file.clone());
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::Save);
    assert!(read(&file).contains("radius-m=7") && !e.dirty());
    assert!(!data.exists(), "Save does not register");
    let _ = std::fs::remove_dir_all(d);
}

// ------------------------------------------------------------------- undo

#[test]
fn undo_and_redo_walk_the_edits_and_typing_in_one_token_is_one_step() {
    let mut e = editor();
    let set = |e: &mut Editor, section: &str, key: &str, v: &str| {
        let _ = e.update(Msg::Input(section.into(), key.into(), v.into()));
    };
    set(&mut e, "metrics", "radius-m", "1");
    set(&mut e, "metrics", "radius-m", "12");
    set(&mut e, "metrics", "radius-l", "5");
    assert_eq!(e.undo.len(), 2, "the two edits of radius-m are one step");
    let _ = e.update(Msg::Undo);
    assert!(e.generate().contains("radius-m=12") && !e.generate().contains("radius-l"));
    let _ = e.update(Msg::Undo);
    assert!(!e.generate().contains("radius-m"));
    let _ = e.update(Msg::Redo);
    assert!(e.generate().contains("radius-m=12"));
    // A new edit drops what could have been redone.
    set(&mut e, "metrics", "radius-s", "2");
    assert!(e.redo.is_empty());
    // The shown text follows undo.
    let _ = e.update(Msg::Undo);
    assert_eq!(e.texts[&("metrics".into(), "radius-m".into())], "12");
}

#[test]
fn undo_has_a_limit_and_opening_a_file_starts_it_over() {
    let mut e = editor();
    for i in 0..(UNDO_MAX + 20) {
        let _ = e.update(Msg::Toggle(
            "colors".into(),
            "auto-contrast".into(),
            i % 2 == 0,
        ));
    }
    assert_eq!(e.undo.len(), UNDO_MAX);
    let d = temp_dir("undo-open");
    let f = d.join("a.conf");
    std::fs::write(&f, "").unwrap();
    e.load(f);
    assert!(e.undo.is_empty() && e.redo.is_empty());
    let _ = std::fs::remove_dir_all(d);
}

// ------------------------------------------------- finding tokens, groups

#[test]
fn the_search_looks_at_names_descriptions_and_sections() {
    let mut e = editor();
    e.search = "radius".into();
    let found: Vec<&Spec> = e.specs.iter().filter(|s| e.visible(s)).collect();
    assert!(found.iter().any(|s| s.key == "radius-m"));
    assert!(found.iter().all(|s| s.key.contains("radius")
        || s.doc.to_lowercase().contains("radius")
        || s.section.contains("radius")));
    e.search = "NO SUCH TOKEN ANYWHERE".into();
    assert!(e.specs.iter().all(|s| !e.visible(s)));
    e.search.clear();
    assert!(e.specs.iter().all(|s| e.visible(s)));
}

#[test]
fn modified_only_shows_what_any_variant_sets_and_the_counts_follow() {
    let mut e = with_text("[metrics]\nradius-m=4\n[dark:colors]\nwindow-bg=#101010\n");
    e.modified_only = true;
    let shown: Vec<(&str, &str)> = e
        .specs
        .iter()
        .filter(|s| e.visible(s))
        .map(|s| (s.section.as_str(), s.key.as_str()))
        .collect();
    assert_eq!(shown, [("colors", "window-bg"), ("metrics", "radius-m")]);
    assert_eq!(e.set_count("metrics"), 1);
    assert_eq!(e.set_count("colors"), 1, "a dark value counts");
    assert_eq!(e.set_count("typography"), 0);
}

#[test]
fn resetting_a_group_removes_only_what_the_current_variant_sets() {
    let mut e =
        with_text("[metrics]\nradius-m=4\nradius-l=5\nspace-m=9\n[dark:metrics]\nradius-m=2\n");
    let _ = e.update(Msg::ResetGroup("metrics".into(), "radius".into()));
    let out = e.generate();
    assert!(
        !out.contains("radius-m=4") && !out.contains("radius-l"),
        "{out}"
    );
    assert!(
        out.contains("space-m=9") && out.contains("[dark:metrics]"),
        "{out}"
    );
    let _ = e.update(Msg::Undo);
    assert!(
        e.generate().contains("radius-l=5"),
        "reset is one undo step"
    );
}

#[test]
fn copy_from_base_fills_only_what_the_variant_does_not_set() {
    let mut e = with_text(
        "[theme]\nname=X\n[colors]\nwindow-bg=#111111\nview-bg=#222222\n[dark:colors]\nview-bg=#999999\n",
    );
    let _ = e.update(Msg::Variant(Variant::Base));
    assert_eq!(e.copy_from_base(), 0, "nothing to copy into the base");
    let _ = e.update(Msg::Variant(Variant::Dark));
    let _ = e.update(Msg::CopyFromBase);
    let out = e.generate();
    assert!(
        out.contains("[dark:colors]\nwindow-bg=#111111\nview-bg=#999999\n"),
        "{out}"
    );
    assert!(!out.contains("[dark:theme]"), "the identity is not copied");
    assert!(e.status.contains("Copied 1"), "{}", e.status);
}

// ----------------------------------------------------------------- gradients

#[test]
fn a_gradient_is_edited_by_angle_stops_and_colors() {
    let mut e = editor();
    let spec = e
        .specs
        .iter()
        .find(|s| s.ty.kind() == iad::theme_config::Kind::Gradient)
        .cloned()
        .expect("the schema has gradient tokens");
    let (s, k) = (spec.section.clone(), spec.key.clone());
    assert!(
        !e.values.contains_key(&(s.clone(), k.clone())),
        "unset to begin with"
    );
    let _ = e.update(Msg::Gradient(s.clone(), k.clone(), GradEdit::Start));
    assert!(
        e.generate().contains(&format!("{k}=linear-gradient(90deg")),
        "{}",
        e.generate()
    );
    let _ = e.update(Msg::Gradient(
        s.clone(),
        k.clone(),
        GradEdit::Angle("45".into()),
    ));
    assert!(
        e.generate().contains(&format!("{k}=linear-gradient(45deg")),
        "{}",
        e.generate()
    );
    let _ = e.update(Msg::Gradient(s.clone(), k.clone(), GradEdit::Add));
    let g = iad::theme_config::Gradient::parse(&e.values[&(s.clone(), k.clone())]).unwrap();
    assert_eq!(g.stops.len(), 3);
    let _ = e.update(Msg::Gradient(
        s.clone(),
        k.clone(),
        GradEdit::Color(0, "#ff0000".into()),
    ));
    let _ = e.update(Msg::Gradient(s.clone(), k.clone(), GradEdit::Remove(1)));
    let g = iad::theme_config::Gradient::parse(&e.values[&(s.clone(), k.clone())]).unwrap();
    assert_eq!(g.stops.len(), 2);
    assert_eq!(g.stops[0].1, iced::Color::from_rgb(1.0, 0.0, 0.0));
    let _ = e.update(Msg::Gradient(s.clone(), k.clone(), GradEdit::Remove(0)));
    assert_eq!(
        iad::theme_config::Gradient::parse(&e.values[&(s, k)])
            .unwrap()
            .stops
            .len(),
        2,
        "two stops stay"
    );
}

#[test]
fn the_main_menu_has_the_shortcuts_and_disables_what_does_not_apply() {
    let e = editor();
    let menu = e.main_menu();
    let accels: Vec<String> = menu.accelerators().into_iter().map(|(a, _)| a).collect();
    for a in ["Ctrl+N", "Ctrl+O", "Ctrl+S", "Ctrl+Shift+S"] {
        assert!(accels.iter().any(|x| x == a), "{a} missing from {accels:?}");
    }
    assert!(
        !accels.iter().any(|x| x == "Ctrl+Z"),
        "undo is disabled with nothing to undo"
    );
    let mut e = editor();
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    assert!(
        e.main_menu()
            .accelerators()
            .iter()
            .any(|(a, _)| a == "Ctrl+Z")
    );
}

#[test]
fn the_views_build_in_every_mode() {
    let mut e = editor();
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.view();
    e.search = "radius".into();
    let _ = e.view();
    e.theme_picker = true;
    e.variant = Variant::Dark;
    let _ = e.view();
    e.toast = Some(Notice {
        text: "x".into(),
        error: true,
    });
    let _ = e.view();
}

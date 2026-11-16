use super::*;

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
    let text = r##"{
    "metrics": {
        "radius-m": 7,
    },
    "dark:metrics": {
        "radius-m": 9,
    },
}
"##;
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
    assert!((tok(&e.generate(), "breakpoint-rules").is_some_and(|t| t.starts_with("xs:"))));
    let _ = e.update(Msg::Rule(
        "tabs".into(),
        0,
        Some((Some("md".into()), Some("outline".into()))),
    ));
    assert!((tok(&e.generate(), "breakpoint-rules").as_deref() == Some("md:outline")));
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
    let path = dir.join("theme.ron");
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
        r##"{
    "tabs": {
        "breakpoint-rules": "xs:pills,md:outline",
    },
    "breakpoints": {
        "md": 800,
    },
}
"##,
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
    let source = r##"{
    "tree.default": {
        "row-height": 47,
        "level-offset": 31,
    },
    "dark:tree.default": {
        "row-height": 61,
    },
}
"##;
    for (dark, expected) in [(false, 47.0), (true, 61.0)] {
        let preview = preview::Preview::build(source, dark);
        let mut element = preview.tree_sample(800.0);
        let mut state = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut state,
            &renderer,
            &layout::Limits::new(iced::Size::ZERO, iced::Size::new(800.0, 1000.0)),
        );
        // Project, Sources, main.rs, theme.ron: all expanded in the actual sample.
        assert_eq!(node.size().height, expected * 4.0);
    }
}

#[test]
fn preview_navigation_resolves_all_density_dimensions_locally() {
    let source = r##"{
    "navigation": {
        "breakpoint-rules": "xs:sm,md:lg",
    },
    "navigation.sm": {
        "row-height": 33,
        "row-padding": 7,
        "gap": 3,
        "inset": 5,
    },
    "navigation.lg": {
        "row-height": 63,
        "row-padding": 17,
        "gap": 13,
        "inset": 15,
    },
    "dark:navigation.lg": {
        "row-height": 71,
    },
}
"##;
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
        r##"{
    "tabs": {
        "breakpoint-rules": "xs:default,md:pills",
    },
    "tabs.default": {
        "indicator-height": 17,
    },
    "toolbar": {
        "breakpoint-rules": "xs:raised,md:flat",
    },
    "toolbar.raised": {
        "shadow-height": 23,
    },
}
"##,
        false,
    );
    let b = preview::Preview::build(
        r##"{
    "tabs.default": {
        "indicator-height": 3,
    },
    "toolbar": {
        "breakpoint-rules": "xs:raised",
    },
    "toolbar.raised": {
        "shadow-height": 7,
    },
}
"##,
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
        r##"{
    "colors": {
        "window-bg": "#112233",
        "window-fg": "#ffffff",
    },
    "light:colors": {
        "window-bg": "#112233",
    },
    "dark:colors": {
        "window-fg": "#111111",
    },
    "dark:metrics": {
        "radius-m": 9,
    },
}
"##,
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
    let mut e = with_text(r##"{
    "colors": {
        "window-bg": "#112233",
    },
    "dark:colors": {
        "window-bg": "#445566",
    },
}
"##);
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
    let mut e = with_text(r##"{
    "colors": {
        "window-bg": "#112233",
    },
    "dark:metrics": {
        "radius-m": 9,
    },
}
"##);
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
        r##"{
    "metrics": {
        "radius-m": 7,
    },
    "dark:metrics": {
        "radius-m": 9,
    },
    "dark:colors": {
        "window-bg": "#112233",
    },
}
"##,
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
    let state = d.join("state.ron");
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

/// The gradient a token's RON holds.
fn gradient_of(v: &ron::Value) -> iad::theme_config::Gradient {
    iad::theme_config::Gradient::from_ron(v).expect("a valid gradient")
}

/// The typed text a token has in a theme file's text (`12px`, `#112233`), looked up by key
/// in any section.
fn tok(text: &str, key: &str) -> Option<String> {
    let doc = iad::theme_config::Document::parse(text).ok()?;
    let schema = Schema::toolkit();
    doc.entries().find(|(_, k, _)| *k == key).map(|(section, k, v)| {
        let base = section.split_once(':').map_or(section, |(_, b)| b);
        schema
            .get(base, k)
            .and_then(|spec| spec.ty.from_ron(v).ok())
            .map(|t| t.to_text().trim_end_matches("px").to_string())
            .unwrap_or_else(|| ron::ser::to_string(v).unwrap_or_default())
    })
}

/// [`tok`] in one section (`"dark:colors"`).
fn tok_in(text: &str, section: &str, key: &str) -> Option<String> {
    let doc = iad::theme_config::Document::parse(text).ok()?;
    let base = section.split_once(':').map_or(section, |(_, b)| b);
    let v = doc.get(section, key)?;
    Some(
        Schema::toolkit()
            .get(base, key)
            .and_then(|spec| spec.ty.from_ron(v).ok())
            .map(|t| t.to_text().trim_end_matches("px").to_string())
            .unwrap_or_else(|| ron::ser::to_string(v).unwrap_or_default()),
    )
}

/// Whether the file text has a section of that name.
fn has_section(text: &str, section: &str) -> bool {
    iad::theme_config::Document::parse(text)
        .is_ok_and(|d| d.entries().any(|(s, _, _)| s == section))
}

fn with_text(text: &str) -> Editor {
    let mut e = editor();
    e.values = super::values_of(text);
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

/// The listed theme files, without the bundled entries every list has.
fn file_presets(dirs: &[PathBuf], folders: &[PathBuf], recent: &[PathBuf]) -> Vec<Preset> {
    list_presets(dirs, folders, recent)
        .into_iter()
        .filter(|p| p.bundled.is_none())
        .collect()
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
        // Everything the file sets is still set after a round trip (and nothing else).
        let keys = |t: &str| -> Vec<_> { super::values_of(t).into_keys().collect() };
        assert_eq!(keys(&text), keys(&out), "{}", f.display());
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
    assert_eq!(super::values_of(&e.generate()).len(), 0, "nothing set, nothing written");
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::Variant(Variant::Dark));
    let _ = e.update(Msg::Input(
        "colors".into(),
        "window-bg".into(),
        "#101010".into(),
    ));
    let out = e.generate();
    assert_eq!(tok(&out, "radius-m").as_deref(), Some("7"));
    assert!(has_section(&out, "dark:colors"));
    assert_eq!(tok(&out, "window-bg").as_deref(), Some("#101010"));
    assert!(!has_section(&out, "colors"));
    // An unset dark value shows the base's, and reset returns to it.
    let _ = e.update(Msg::Reset("colors".into(), "window-bg".into()));
    assert!(!has_section(&e.generate(), "dark:colors"));
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
        "window-decorations".into(),
        "button-size".into(),
        "30".into(),
    ));
    assert!((tok(&e.generate(), "button-size").as_deref() == Some("30")));
}

#[test]
fn unknown_keys_are_kept() {
    let e = with_text(r##"{
    "player": {
        "artwork": "disc",
    },
    "colors": {
        "accent": "#112233",
    },
}
"##);
    let out = e.generate();
    assert_eq!(tok(&out, "artwork").as_deref(), Some("\"disc\""));
    assert!(has_section(&out, "player"));
    assert!(tok(&out, "accent").as_deref() == Some("#112233"));
}

// ---------------------------------------------------------- the dot

#[test]
fn the_unsaved_dot_shows_only_when_there_are_unsaved_changes() {
    let d = temp_dir("dot");
    let f = d.join("t.ron");
    std::fs::write(&f, r##"{
    "metrics": {
        "radius-m": 1,
    },
}
"##).unwrap();
    let mut e = editor();
    e.load(f.clone());
    assert!(!e.dirty());
    assert_eq!(e.dot(), "");
    assert!(!e.title().contains('●') && e.title().starts_with("t.ron"));
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "9".into()));
    assert_eq!(e.dot(), "● ");
    assert!(e.title().starts_with("● t.ron"));
    let _ = e.update(Msg::SavedAs(Some(f.clone())));
    assert_eq!(e.dot(), "");
    assert!(!e.title().contains('●'));
    let _ = std::fs::remove_dir_all(d);
}

// ------------------------------------------------------------ saving

#[test]
fn save_upserts_the_original_file() {
    let d = temp_dir("save");
    let f = d.join("mine.ron");
    let mut e = editor();
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::SavedAs(Some(f.clone())));
    // Created.
    assert_eq!(tok(&read(&f), "radius-m").as_deref(), Some("7"));
    assert!(!e.dirty());
    // Replaced, by Save, at the same path.
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "8".into()));
    assert!(e.dirty());
    let _ = e.update(Msg::Save);
    assert!((tok(&read(&f), "radius-m").as_deref() == Some("8")) && !(tok(&read(&f), "radius-m").as_deref() == Some("7")));
    assert_eq!(e.path, Some(f));
    assert!(!e.dirty());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn save_writes_through_a_symlink_to_its_target() {
    let d = temp_dir("through");
    let real = d.join("real.ron");
    let link = d.join("link.ron");
    std::fs::write(&real, r##"{
    "metrics": {
        "radius-m": 1,
    },
}
"##).unwrap();
    symlink(&real, &link).unwrap();
    let mut e = editor();
    e.load(link.clone());
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "5".into()));
    let _ = e.update(Msg::Save);
    assert!(is_link(&link), "the link is still a link");
    assert!((tok(&read(&real), "radius-m").as_deref() == Some("5")));
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
        bundled: None,
        group: "Folders",
        name: "夜空のテーマ العربية Ελληνικά हिन्दी café 🎨 AnUnbrokenVeryLongThemeName".repeat(16),
        path: PathBuf::from(format!("/themes/{}.ron", "long-directory-name".repeat(24))),
        target: Some(PathBuf::from(format!(
            "/linked/{}.ron",
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
        bundled: None,
        name: "Theme".into(),
        path: "/themes/theme.ron".into(),
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
    let target = d.join("other.ron");
    std::fs::write(&target, r##"{
    "metrics": {
        "radius-m": 2,
    },
}
"##).unwrap();
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

// ------------------------------------------------------------- list

#[test]
fn a_clean_environment_lists_only_the_bundled_themes() {
    let d = temp_dir("empty");
    assert!(file_presets(&[d.clone()], &[], &[]).is_empty());
    let bundled: Vec<_> = list_presets(&[d.clone()], &[], &[])
        .into_iter()
        .filter_map(|p| p.bundled)
        .collect();
    assert_eq!(
        bundled,
        [
            "libadwaita",
            "cosmic",
            "bubblegum",
            "solarized",
            "synthwave",
            "terminal"
        ]
    );
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_folder_scan_lists_only_conf_files_by_name() {
    let d = temp_dir("scan");
    for f in ["b.ron", "a.ron", "notes.txt", "c.ron.bak"] {
        std::fs::write(d.join(f), "").unwrap();
    }
    std::fs::create_dir(d.join("sub.ron")).unwrap();
    symlink(d.join("nowhere.ron"), d.join("broken.ron")).unwrap();
    let names: Vec<_> = scan_folder(&d)
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["a.ron", "b.ron"]);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn switching_with_unsaved_changes_needs_a_second_click() {
    let d = temp_dir("switch");
    let (a, b) = (d.join("a.ron"), d.join("b.ron"));
    std::fs::write(&a, r##"{
    "metrics": {
        "radius-m": 1,
    },
}
"##).unwrap();
    std::fs::write(&b, r##"{
    "metrics": {
        "radius-m": 2,
    },
}
"##).unwrap();
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
    assert_eq!(tok(&e.generate(), "radius-m").as_deref(), Some("9"));
    e.pick(Target::File(a.clone()));
    assert_eq!(e.path, Some(a.clone()));
    assert_eq!(tok(&e.generate(), "radius-m").as_deref(), Some("1"));
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
        last_file: Some("/a/one.ron".into()),
        folders: vec!["/x".into(), "/y".into()],
        recent: vec!["/a/one.ron".into(), "/b/two.ron".into()],
        section: Some("metrics".into()),
        variant: Some(Variant::Dark),
    };
    assert_eq!(State::parse(&state.to_text()), state);
    assert_eq!(State::parse(""), State::default());
    // It is plain RON a person can read and edit.
    assert!(state.to_text().contains("last_file: Some(\"/a/one.ron\")"), "{}", state.to_text());
}

#[test]
fn the_recent_list_has_no_duplicates_and_a_limit() {
    let mut s = State::default();
    for i in 0..12 {
        s.remember(std::path::Path::new(&format!("/t/{i}.ron")));
    }
    s.remember(std::path::Path::new("/t/5.ron"));
    assert_eq!(s.recent.len(), RECENT_MAX);
    assert_eq!(s.recent[0], PathBuf::from("/t/5.ron"));
    assert_eq!(
        s.recent
            .iter()
            .filter(|p| **p == PathBuf::from("/t/5.ron"))
            .count(),
        1
    );
}

#[test]
fn the_open_file_section_and_variant_are_remembered_across_runs() {
    let d = temp_dir("state");
    let state = d.join("state.ron");
    let theme = d.join("t.ron");
    std::fs::write(&theme, r##"{
    "theme": {
        "name": "T",
        "key": "t",
    },
}
"##).unwrap();
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
    let other = d.join("o.ron");
    std::fs::write(&other, "").unwrap();
    let (cli, _) = Editor::with(Some(state), Some(other.clone()));
    assert_eq!(cli.path, Some(other));
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn saving_remembers_the_file_as_last_and_recent() {
    let d = temp_dir("remember");
    let state = d.join("state.ron");
    let (mut e, _) = Editor::with(Some(state.clone()), None);
    e.theme_dirs = Vec::new();
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let file = d.join("mine.ron");
    let _ = e.update(Msg::SavedAs(Some(file.clone())));
    assert_eq!(e.state.last_file, Some(file.clone()));
    assert_eq!(e.state.recent.first(), Some(&file));
    assert!(read(&state).contains("mine.ron"));
    assert!(!read(&state).contains("embedded"));
    let _ = std::fs::remove_dir_all(d);
}

// ---------------------------------------------------------- save as

#[test]
fn save_only_updates_the_file_and_registers_nothing() {
    let d = temp_dir("save-only");
    let data = d.join("data");
    let file = d.join("a.ron");
    std::fs::write(&file, "").unwrap();
    let mut e = editor();
    e.theme_dirs = vec![data.clone()];
    e.load(file.clone());
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::Save);
    assert!((tok(&read(&file), "radius-m").as_deref() == Some("7")) && !e.dirty());
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
    assert!((tok(&e.generate(), "radius-m").as_deref() == Some("12")) && tok(&e.generate(), "radius-l").is_none());
    let _ = e.update(Msg::Undo);
    assert!(!e.generate().contains("radius-m"));
    let _ = e.update(Msg::Redo);
    assert!((tok(&e.generate(), "radius-m").as_deref() == Some("12")));
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
    let f = d.join("a.ron");
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
    let mut e = with_text(r##"{
    "metrics": {
        "radius-m": 4,
    },
    "dark:colors": {
        "window-bg": "#101010",
    },
}
"##);
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
        with_text(r##"{
    "metrics": {
        "radius-m": 4,
        "radius-l": 5,
        "space-m": 9,
    },
    "dark:metrics": {
        "radius-m": 2,
    },
}
"##);
    let _ = e.update(Msg::ResetGroup("metrics".into(), "radius".into()));
    let out = e.generate();
    assert!(
        !(tok(&out, "radius-m").as_deref() == Some("4")) && tok(&out, "radius-l").is_none(),
        "{out}"
    );
    assert!(
        (tok(&out, "space-m").as_deref() == Some("9")) && has_section(&out, "dark:metrics"),
        "{out}"
    );
    let _ = e.update(Msg::Undo);
    assert!(
        (tok(&e.generate(), "radius-l").as_deref() == Some("5")),
        "reset is one undo step"
    );
}

#[test]
fn copy_from_base_fills_only_what_the_variant_does_not_set() {
    let mut e = with_text(
        r##"{
    "theme": {
        "name": "X",
    },
    "colors": {
        "window-bg": "#111111",
        "view-bg": "#222222",
    },
    "dark:colors": {
        "view-bg": "#999999",
    },
}
"##,
    );
    let _ = e.update(Msg::Variant(Variant::Base));
    assert_eq!(e.copy_from_base(), 0, "nothing to copy into the base");
    let _ = e.update(Msg::Variant(Variant::Dark));
    let _ = e.update(Msg::CopyFromBase);
    let out = e.generate();
    assert!(has_section(&out, "dark:colors"), "{out}");
    assert_eq!(tok_in(&out, "dark:colors", "window-bg").as_deref(), Some("#111111"), "{out}");
    assert_eq!(tok_in(&out, "dark:colors", "view-bg").as_deref(), Some("#999999"), "{out}");
    assert!(!has_section(&out, "dark:theme"), "the identity is not copied");
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
        tok(&e.generate(), &k).is_some_and(|t| t.starts_with("linear-gradient(90deg")),
        "{}",
        e.generate()
    );
    let _ = e.update(Msg::Gradient(
        s.clone(),
        k.clone(),
        GradEdit::Angle("45".into()),
    ));
    assert!(
        tok(&e.generate(), &k).is_some_and(|t| t.starts_with("linear-gradient(45deg")),
        "{}",
        e.generate()
    );
    let _ = e.update(Msg::Gradient(s.clone(), k.clone(), GradEdit::Add));
    let g = gradient_of(&e.values[&(s.clone(), k.clone())]);
    assert_eq!(g.stops.len(), 3);
    let _ = e.update(Msg::Gradient(
        s.clone(),
        k.clone(),
        GradEdit::Color(0, "#ff0000".into()),
    ));
    let _ = e.update(Msg::Gradient(s.clone(), k.clone(), GradEdit::Remove(1)));
    let g = gradient_of(&e.values[&(s.clone(), k.clone())]);
    assert_eq!(g.stops.len(), 2);
    assert_eq!(g.stops[0].1, iced::Color::from_rgb(1.0, 0.0, 0.0));
    let _ = e.update(Msg::Gradient(s.clone(), k.clone(), GradEdit::Remove(0)));
    assert_eq!(
        gradient_of(&e.values[&(s, k)])
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

// ------------------------------------------------- bundled themes and the global theme

#[test]
fn the_bundled_themes_are_valid_and_match_the_themes_directory() {
    for (key, text) in crate::bundled::BUNDLED {
        let report = Schema::toolkit().validate(text);
        assert!(report.errors.is_empty(), "{key}: {:?}", report.errors);
        assert_eq!(
            read(
                &PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../themes"))
                    .join(format!("{key}.ron"))
            ),
            text
        );
        let parsed = iad::theme_config::Document::parse(text).unwrap();
        assert_eq!(
            parsed.text("theme", "key"),
            Some(key),
            "the file's own key names it"
        );
    }
}

#[test]
fn toy_reference_themes_are_not_bundled() {
    for toy in ["win95", "winxp", "win7", "mate", "gnome2"] {
        assert!(crate::bundled::text(toy).is_none(), "{toy}");
    }
}

#[test]
fn opening_a_bundled_theme_gives_an_unsaved_copy_that_never_writes_the_bundle() {
    let d = temp_dir("bundled-open");
    let mut e = editor();
    e.theme_dirs = vec![d.join("data")];
    let _ = e.update(Msg::PickBundled("terminal"));
    assert!(e.path.is_none() && e.source == Some("terminal") && !e.dirty());
    assert!((tok(&e.generate(), "name").as_deref() == Some("Terminal")));
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "9".into()));
    assert!(e.dirty());
    // Switching to another bundled theme asks first, then discards.
    let _ = e.update(Msg::PickBundled("cosmic"));
    assert_eq!(e.source, Some("terminal"));
    let _ = e.update(Msg::PickBundled("cosmic"));
    assert_eq!(e.source, Some("cosmic"));
    // Save as gives it a file of its own and forgets the bundle.
    assert!(e.save_to(d.join("mine.ron")));
    assert!(e.source.is_none() && e.path == Some(d.join("mine.ron")));
    assert!(tok(&read(&d.join("mine.ron")), "name").is_some_and(|n| n.contains("COSMIC")));
    assert_eq!(
        crate::bundled::text("cosmic").unwrap(),
        include_str!("../../themes/cosmic.ron")
    );
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_global_theme_saves_first_instead_of_refusing() {
    let d = temp_dir("global-dirty");
    let data = d.join("data");
    let f = d.join("t.ron");
    std::fs::write(&f, r##"{
    "theme": {
        "name": "T",
        "key": "t",
    },
}
"##).unwrap();
    let mut e = editor();
    e.theme_dirs = vec![data.clone()];
    // Untitled: Save as opens, and nothing is linked yet.
    assert!(e.update(Msg::SetGlobal).units() > 0, "Save as opens");
    assert_eq!(e.after_save, Some(AfterSave::Global));
    assert!(!data.join("default.ron").exists());
    // Cancelling the dialog forgets the plan.
    let _ = e.update(Msg::SavedAs(None));
    assert_eq!(e.after_save, None);
    // Choosing a file writes it and makes it global.
    let _ = e.update(Msg::SetGlobal);
    let chosen = d.join("mine");
    let _ = e.update(Msg::SavedAs(Some(chosen.clone())));
    let chosen = chosen.with_extension("conf");
    assert!(chosen.is_file() && e.path.as_ref() == Some(&chosen));
    assert!(e.is_global(), "{}", e.status);
    assert_eq!(default_target(&data), Some(canon(&chosen)));
    assert_eq!(e.after_save, None);
    // A file with unsaved changes is saved, then linked.
    e.load(f.clone());
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    assert!(e.dirty());
    let _ = e.update(Msg::SetGlobal);
    assert!(!e.dirty() && e.is_global(), "{}", e.status);
    assert!((tok(&read(&f), "radius-m").as_deref() == Some("7")));
    assert_eq!(default_target(&data), Some(canon(&f)));
    // A bundled theme that was edited has no file either: Save as first.
    let _ = e.update(Msg::PickBundled("terminal"));
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "9".into()));
    assert!(e.update(Msg::SetGlobal).units() > 0);
    assert_eq!(e.after_save, Some(AfterSave::Global));
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_views_build_for_a_bundled_global_and_an_untitled_theme() {
    let d = temp_dir("views-global");
    let mut e = editor();
    e.theme_dirs = vec![d.join("data")];
    e.rescan();
    let _ = e.view();
    let _ = e.update(Msg::PickBundled("libadwaita"));
    let _ = e.view();
    let _ = e.update(Msg::SetGlobal);
    let _ = e.update(Msg::ThemePicker(true));
    let _ = e.view();
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_gallery_starts_with_the_unsaved_values_in_a_temp_file_and_restarts() {
    use std::os::unix::fs::PermissionsExt;
    let d = temp_dir("gallery");
    let out = d.join("seen");
    let exe = d.join("fake.sh");
    std::fs::write(
        &exe,
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$1\" > {0}.tmp\nprintf 'scheme=%s\\n' \"${{GALLERY_SCHEME-unset}}\" >> {0}.tmp\ncat \"$IAD_THEME\" >> {0}.tmp\nmv {0}.tmp {0}\nsleep 30\n",
            out.display()
        ),
    )
    .unwrap();
    std::fs::set_permissions(&exe, std::fs::Permissions::from_mode(0o755)).unwrap();
    let f = d.join("t.ron");
    std::fs::write(&f, r##"{
    "metrics": {
        "radius-m": 1,
    },
}
"##).unwrap();
    let mut e = editor();
    e.exe = Some(exe);
    e.load(f.clone());
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::Gallery);
    assert!(e.gallery.is_some(), "{}", e.status);
    let mut seen = String::new();
    for _ in 0..100 {
        if let Ok(t) = std::fs::read_to_string(&out) {
            seen = t;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(
        seen.starts_with("gallery\n")
            && tok(seen.split_once("scheme=").map_or("", |(_, rest)| rest.split_once('\n').map_or("", |(_, t)| t)), "radius-m").as_deref() == Some("7"),
        "{seen}"
    );
    assert_eq!(
        read(&f),
        r##"{
    "metrics": {
        "radius-m": 1,
    },
}
"##,
        "the open file is untouched"
    );
    // The chosen scheme reaches the new gallery, and only it.
    let _ = e.update(Msg::LaunchMode(w::ThemeMode::Dark));
    let _ = std::fs::remove_file(&out);
    let _ = e.update(Msg::Gallery);
    let mut seen = String::new();
    for _ in 0..100 {
        if let Ok(t) = std::fs::read_to_string(&out) {
            seen = t;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    assert!(seen.contains("scheme=dark\n"), "{seen}");
    // Again: the old one is closed, a new one runs.
    let first = e.gallery.as_ref().unwrap().id();
    let _ = e.update(Msg::Gallery);
    assert_ne!(e.gallery.as_ref().unwrap().id(), first);
    let _ = e.gallery.take().unwrap().kill();
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_missing_program_is_reported_not_started() {
    let mut e = editor();
    e.exe = Some(PathBuf::from("/nonexistent/theme-manager"));
    let _ = e.update(Msg::Gallery);
    assert!(e.gallery.is_none() && e.status.contains("Could not start"));
}

#[test]
fn dirty_is_cached_for_the_values_and_follows_every_change() {
    let mut e = editor();
    assert!(!e.dirty() && !e.dirty());
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    assert!(e.dirty(), "an edit is noticed through the cache");
    assert!(e.dirty());
    let _ = e.update(Msg::Undo);
    assert!(!e.dirty(), "undo back to the saved text is clean again");
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let d = temp_dir("dirty-cache");
    assert!(e.save_to(d.join("t.ron")));
    assert!(!e.dirty(), "saving makes the new text the clean one");
    let _ = std::fs::remove_dir_all(d);
}

/// Lays `block`'s group out at `width` and returns its height.
fn laid_out_height(
    e: &Editor,
    section: &str,
    title: &str,
    specs: Vec<&iad::theme_config::Spec>,
    width: f32,
) -> f32 {
    use iced::advanced::{layout, renderer::Headless, widget::Tree};
    let renderer = iced::futures::executor::block_on(iced::Renderer::new(
        iced::Font::DEFAULT,
        iced::Pixels(14.0),
        Some("tiny-skia"),
    ))
    .unwrap();
    let mut element = e.group(section, title, specs, false, iad::Metrics::desktop());
    let mut state = Tree::new(&element);
    element
        .as_widget_mut()
        .layout(
            &mut state,
            &renderer,
            &layout::Limits::new(iced::Size::ZERO, iced::Size::new(width, f32::INFINITY)),
        )
        .size()
        .height
}

fn assert_heights_match(e: &Editor, width: f32) {
    for section in e.sections.clone() {
        for (title, specs) in e.groups_of(&section) {
            let estimate = e.group_height(&section, &title, &specs);
            let real = laid_out_height(e, &section, &title, specs, width);
            assert!(
                (estimate - real).abs() < 0.6,
                "[{section}] {title} at {width}px: computed {estimate}, laid out {real}"
            );
        }
    }
}

#[test]
fn block_heights_match_the_real_layout() {
    let mut e = editor();
    // The window is at least 560px wide, and tokens are one line at any width from there.
    for width in [crate::page::MIN_WIDTH, 900.0, 1400.0] {
        assert_heights_match(&e, width);
    }
    // A gradient adds a preview, an angle and a row per stop.
    let grad = e
        .specs
        .iter()
        .find(|s| s.ty.kind() == iad::theme_config::Kind::Gradient)
        .cloned()
        .unwrap();
    for stops in [2usize, 3, 8] {
        let list: Vec<String> = (0..stops)
            .map(|i| format!("#{:02x}0000 {}%", i * 20, i * 100 / (stops - 1)))
            .collect();
        e.values.insert(
            (grad.section.clone(), grad.key.clone()),
            iad::theme_config::Gradient::parse(&format!("linear-gradient(90deg, {})", list.join(", ")))
                .map(|g| iad::theme_config::Value::Gradient(g).to_ron())
                .unwrap(),
        );
        e.refresh();
        assert_heights_match(&e, 760.0);
    }
    // Breakpoint rules add a row per rule.
    let rules: Vec<(String, String)> = e
        .specs
        .iter()
        .filter(|s| s.key == "breakpoint-rules")
        .map(|s| (s.section.clone(), s.key.clone()))
        .collect();
    for raw in [
        "xs:compact",
        "xs:compact,sm:medium",
        "xs:compact,sm:medium,md:compact",
        // All six breakpoints: no more rules to add, so no button.
        "xs:compact,sm:medium,md:compact,lg:compact,xl:compact,xxl:compact",
        // Invalid rules show the same one-line heading.
        "bogus:nothing",
    ] {
        for at in &rules {
            e.values.insert(at.clone(), ron::Value::String(raw.to_string()));
        }
        e.refresh();
        assert_heights_match(&e, 760.0);
    }
    // A folded group is its heading.
    let (section, title) = {
        let s = e.sections[0].clone();
        (s.clone(), e.groups_of(&s)[0].0.clone())
    };
    e.collapsed.insert((section, title));
    assert_heights_match(&e, 760.0);
}

#[test]
fn only_the_blocks_near_the_view_are_built_and_the_page_keeps_its_height() {
    let mut e = editor();
    let _ = e.update(Msg::Section("all".into()));
    let m = iad::Metrics::desktop();
    let gap = m.group_spacing;
    let (count, total, top) = {
        let blocks = e.blocks();
        let total: f32 =
            blocks.iter().map(|b| b.height).sum::<f32>() + gap * (blocks.len() - 1) as f32;
        (blocks.len(), total, e.built(&blocks, gap))
    };
    assert!(total > 5000.0, "the All page is long: {total}");
    assert_eq!(top.start, 0);
    assert!(top.len() < count / 2, "{top:?} of {count}");
    // Scrolled to the end, the last block is built and the first is not.
    let _ = e.update(Msg::TokensScrolled(iad::layouts::virtualize::Viewport {
        offset: total - 800.0,
        height: 800.0,
    }));
    let end = {
        let blocks = e.blocks();
        e.built(&blocks, gap)
    };
    assert_eq!(end.end, count);
    assert!(end.start > 0 && end.len() < count / 2, "{end:?} of {count}");
    let _ = e.page(m);
    // A new page starts at the top again.
    let _ = e.update(Msg::Section("colors".into()));
    assert_eq!(e.tokens_view.offset, 0.0);
    // A short page is built whole.
    let _ = e.update(Msg::Search("accent-fg".into()));
    let blocks = e.blocks();
    assert!(!blocks.is_empty());
    assert_eq!(e.built(&blocks, gap), 0..blocks.len());
}

// ----------------------------------------- every theme is a file; default.ron is a symlink

#[test]
fn setting_the_global_theme_points_default_conf_at_the_file_and_replaces_it_atomically() {
    let d = temp_dir("link-global");
    let (data, other) = (d.join("data"), d.join("else"));
    std::fs::create_dir_all(&other).unwrap();
    let (a, b) = (other.join("a.ron"), other.join("b.ron"));
    std::fs::write(&a, r##"{
    "theme": {
        "name": "A",
    },
}
"##).unwrap();
    std::fs::write(&b, r##"{
    "theme": {
        "name": "B",
    },
}
"##).unwrap();
    assert_eq!(default_target(&data), None);
    let linked = set_default(&data, &a).unwrap();
    assert_eq!(
        linked,
        Linked {
            link: data.join("default.ron"),
            kept: None
        }
    );
    assert!(is_link(&linked.link) && default_target(&data) == Some(canon(&a)));
    set_default(&data, &b).unwrap();
    assert_eq!(default_target(&data), Some(canon(&b)));
    // Both themes are still their own files, and nothing else was left in the directory.
    assert!(a.is_file() && b.is_file());
    assert_eq!(std::fs::read_dir(&data).unwrap().count(), 1);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_regular_default_conf_is_kept_as_a_theme_of_its_own_not_overwritten() {
    let d = temp_dir("keep-default");
    let data = d.join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("default.ron"), r##"{
    "theme": {
        "name": "Old Look",
    },
}
"##).unwrap();
    let a = d.join("a.ron");
    std::fs::write(&a, r##"{
    "theme": {
        "name": "A",
    },
}
"##).unwrap();
    let linked = set_default(&data, &a).unwrap();
    assert!(is_link(&data.join("default.ron")));
    assert_eq!(default_target(&data), Some(canon(&a)));
    let kept = linked.kept.unwrap();
    assert_eq!(kept, data.join("Old_Look.ron"));
    assert_eq!(read(&kept), r##"{
    "theme": {
        "name": "Old Look",
    },
}
"##);
    // Again: another regular file in the way gets another name.
    std::fs::remove_file(data.join("default.ron")).unwrap();
    std::fs::write(data.join("default.ron"), r##"{
    "theme": {
        "name": "Old Look",
    },
}
"##).unwrap();
    assert_eq!(
        set_default(&data, &a).unwrap().kept,
        Some(data.join("Old_Look-2.ron"))
    );
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_open_regular_default_conf_moves_to_its_own_name_and_becomes_the_target() {
    let d = temp_dir("open-default");
    let data = d.join("data");
    std::fs::create_dir_all(&data).unwrap();
    let file = data.join("default.ron");
    std::fs::write(&file, r##"{
    "theme": {
        "name": "Mine",
    },
}
"##).unwrap();
    let mut e = editor();
    e.theme_dirs = vec![data.clone()];
    e.load(file.clone());
    let _ = e.update(Msg::SetGlobal);
    let moved = data.join("Mine.ron");
    assert!(is_link(&file) && default_target(&data) == Some(canon(&moved)));
    assert_eq!(e.path, Some(moved.clone()));
    assert!(e.is_global(), "{}", e.status);
    assert_eq!(read(&moved), r##"{
    "theme": {
        "name": "Mine",
    },
}
"##);
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn removing_the_global_theme_removes_the_symlink_and_never_a_theme_file() {
    let d = temp_dir("clear-global");
    assert!(clear_global(&d).is_err());
    let a = d.join("a.ron");
    std::fs::write(&a, "x").unwrap();
    symlink(&a, d.join("default.ron")).unwrap();
    clear_global(&d).unwrap();
    assert!(!d.join("default.ron").exists() && a.is_file());
    std::fs::write(d.join("default.ron"), "y").unwrap();
    assert!(clear_global(&d).unwrap_err().contains("regular file"));
    assert!(d.join("default.ron").is_file());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn a_bundled_theme_is_copied_as_its_own_file_and_linked() {
    let d = temp_dir("bundled-global");
    let data = d.join("data");
    let mut e = editor();
    e.theme_dirs = vec![data.clone()];
    let _ = e.update(Msg::PickBundled("solarized"));
    assert!(!e.is_global());
    let _ = e.update(Msg::SetGlobal);
    assert!(e.is_global(), "{}", e.status);
    assert_eq!(
        read(&data.join("solarized.ron")),
        crate::bundled::text("solarized").unwrap()
    );
    assert!(is_link(&data.join("default.ron")));
    assert_eq!(
        default_target(&data),
        Some(canon(&data.join("solarized.ron")))
    );
    // Installed unchanged, it is listed once, as the installed file, marked as the global one.
    let list = list_presets(&[data.clone()], &[], &[]);
    assert!(!list.iter().any(|p| p.bundled == Some("solarized")));
    assert!(
        list.iter()
            .any(|p| p.name == "Solarized" && p.is_default && p.group == "Installed")
    );
    // Another theme moves the link; both stay files. Removing leaves the files.
    let _ = e.update(Msg::PickBundled("cosmic"));
    let _ = e.update(Msg::SetGlobal);
    assert_eq!(
        default_target(&data),
        Some(canon(&data.join("cosmic.ron")))
    );
    assert!(data.join("solarized.ron").is_file());
    let _ = e.update(Msg::ClearGlobal);
    assert!(!e.is_global() && !data.join("default.ron").exists());
    assert!(data.join("cosmic.ron").is_file() && data.join("solarized.ron").is_file());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn an_installed_file_with_other_values_is_never_overwritten_by_a_bundled_theme() {
    let d = temp_dir("bundled-keep");
    let data = d.join("data");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::write(data.join("terminal.ron"), r##"{
    "metrics": {
        "radius-m": 1,
    },
}
"##).unwrap();
    let err = set_global_bundled(&data, "terminal").unwrap_err();
    assert!(err.contains("already exists"), "{err}");
    assert_eq!(read(&data.join("terminal.ron")), r##"{
    "metrics": {
        "radius-m": 1,
    },
}
"##);
    assert!(!data.join("default.ron").exists());
    assert!(set_global_bundled(&data, "nope").is_err());
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_list_groups_the_installed_bundled_folder_and_recent_themes() {
    let d = temp_dir("list");
    let data = d.join("data");
    let folder = d.join("folder");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(data.join("own.ron"), r##"{
    "theme": {
        "name": "Own",
    },
}
"##).unwrap();
    std::fs::write(folder.join("mine.ron"), r##"{
    "theme": {
        "name": "Mine",
    },
}
"##).unwrap();
    std::fs::write(folder.join("plain.ron"), "").unwrap();
    let far = temp_dir("recent").join("far.ron");
    std::fs::write(&far, "").unwrap();
    // The global theme lives outside the directory: it is listed as installed, marked.
    set_default(&data, &folder.join("plain.ron")).unwrap();
    let list = file_presets(
        &[data.clone()],
        &[folder.clone()],
        &[far, folder.join("mine.ron"), d.join("gone.ron")],
    );
    let group = |n: &str| list.iter().find(|p| p.name == n).map(|p| p.group);
    assert_eq!(group("Own"), Some("Installed"));
    assert_eq!(group("plain"), Some("Installed"));
    assert!(list.iter().find(|p| p.name == "plain").unwrap().is_default);
    assert_eq!(group("Mine"), Some("Folders"));
    assert_eq!(group("far"), Some("Recent"));
    assert_eq!(list.iter().filter(|p| p.name == "Mine").count(), 1);
    assert!(list.iter().all(|p| p.path.is_file()));
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn there_is_no_registering_in_the_menu_or_on_save_as() {
    let d = temp_dir("no-register");
    let data = d.join("data");
    let mut e = editor();
    e.theme_dirs = vec![data.clone()];
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "7".into()));
    let _ = e.update(Msg::SavedAs(Some(d.join("mine.ron"))));
    assert!(!data.exists(), "Save as writes the file and nothing else");
    let _ = std::fs::remove_dir_all(d);
}

#[test]
fn the_header_offers_set_as_global_theme_and_shows_when_it_is_done() {
    let d = temp_dir("header-global");
    let mut e = editor();
    e.theme_dirs = vec![d.join("data")];
    let _ = e.view();
    let _ = e.update(Msg::PickBundled("terminal"));
    assert!(!e.is_global());
    let _ = e.view();
    let _ = e.update(Msg::SetGlobal);
    assert!(e.is_global());
    let _ = e.view();
    let _ = std::fs::remove_dir_all(d);
}

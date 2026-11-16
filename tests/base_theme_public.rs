//! Base-theme precedence and discovery, without mutating the parent process environment.

use iad::theme_config::{Inheritance, Source, ThemeConfig};
use iced_adaptive_design as iad;
use std::process::Command;

const BASE: &str = r##"{
    "metrics": {
        "radius-window": 15,
        "space-m": 12,
    },
    "colors": {
        "window-bg": "#ffffff",
    },
    "dark:colors": {
        "window-bg": "#222222",
    },
}
"##;

#[test]
fn base_toolkit_and_app_layers_are_validated_in_order() {
    let cfg = ThemeConfig::builder("base-test")
        .files(false)
        .base_theme(BASE)
        .toolkit_text(r##"{
    "metrics": {
        "radius-window": 21,
        "space-m": "invalid",
    },
}
"##)
        .app_text(r##"{
    "metrics": {
        "radius-window": 25,
    },
}
"##)
        .build();
    let theme = iad::Theme::light().with_theme_config(&cfg);
    assert_eq!(theme.metrics.radius_window, 25.0);
    assert_eq!(theme.metrics.space_m, 12.0);
    assert_eq!(cfg.source("metrics", "space-m"), Source::Theme);
    assert_eq!(cfg.source("metrics", "radius-window"), Source::App);
    assert!(!cfg.warnings().is_empty());
}

#[test]
fn explicit_base_survives_disabled_discovery_and_opt_out() {
    for (files, inheritance) in [
        (false, Inheritance::Inherit),
        (true, Inheritance::OptOut),
        (false, Inheritance::OptOut),
    ] {
        let cfg = ThemeConfig::builder("base-test")
            .files(files)
            .app_text("")
            .inheritance(inheritance)
            .base_theme(BASE)
            .build();
        assert_eq!(
            iad::Theme::light()
                .with_theme_config(&cfg)
                .metrics
                .radius_window,
            15.0
        );
        assert_eq!(cfg.inheritance(), inheritance);
    }
}

#[test]
fn external_theme_overrides_are_not_app_overrides() {
    let cfg = ThemeConfig::builder("base-test")
        .files(false)
        .base_theme(BASE)
        .toolkit_text(r##"{
    "metrics": {
        "radius-window": 21,
    },
}
"##)
        .build();
    assert_eq!(cfg.inheritance(), Inheritance::Inherit);
    assert!(cfg.overrides().is_empty());
    let mut desktop = iad::config::Config::default();
    desktop.allow_app_overrides = false;
    let cfg = desktop.with_theme_config(&cfg);
    assert_eq!(iad::Theme::from_config(&cfg).metrics.radius_window, 21.0);
}

#[test]
fn invalid_layers_preserve_the_last_valid_value() {
    let cfg = ThemeConfig::builder("base-test")
        .files(false)
        .base_theme(r##"{
    "metrics": {
        "radius-window": "invalid",
        "space-m": 12,
    },
}
"##)
        .toolkit_text(r##"{
    "metrics": {
        "radius-window": 21,
        "space-m": "invalid",
    },
}
"##)
        .app_text(r##"{
    "metrics": {
        "radius-window": "invalid",
        "space-m": "invalid",
    },
}
"##)
        .build();
    let theme = iad::Theme::light().with_theme_config(&cfg);
    assert_eq!(theme.metrics.radius_window, 21.0);
    assert_eq!(theme.metrics.space_m, 12.0);
    assert_eq!(cfg.warnings().len(), 4);

    let invalid_base = ThemeConfig::builder("base-test")
        .files(false)
        .base_theme(r##"{
    "metrics": {
        "radius-window": "invalid",
    },
}
"##)
        .build();
    assert_eq!(invalid_base.number("metrics", "radius-window"), Some(0.0));
    assert_eq!(
        invalid_base.source("metrics", "radius-window"),
        Source::Default
    );
}

#[test]
fn desktop_policy_rejects_app_values_but_retains_both_theme_layers() {
    let cfg = ThemeConfig::builder("base-test")
        .files(false)
        .base_theme(BASE)
        .toolkit_text(r##"{
    "metrics": {
        "radius-window": 21,
    },
}
"##)
        .app_text(r##"{
    "metrics": {
        "radius-window": 25,
        "space-m": 18,
    },
    "colors": {
        "window-bg": "#ff0000",
    },
}
"##)
        .build();
    let mut desktop = iad::config::Config::default();
    desktop.allow_app_overrides = false;
    let desktop = desktop.with_theme_config(&cfg);
    let theme = iad::Theme::from_config(&desktop);
    assert_eq!(theme.metrics.radius_window, 21.0);
    assert_eq!(theme.metrics.space_m, 12.0);
    assert_eq!(theme.palette.window_bg_color, iced::Color::WHITE);
    let resolved = cfg.for_policy(false);
    assert_eq!(resolved.denied().len(), 3);
    assert_eq!(resolved.source("metrics", "radius-window"), Source::Theme);
    assert_eq!(resolved.source("metrics", "space-m"), Source::Theme);
}

#[test]
fn discovery_matrix() {
    let root = std::env::temp_dir().join(format!("iad-base-theme-{}", std::process::id()));
    std::fs::create_dir_all(root.join("installed/iad/themes")).unwrap();
    let selected = root.join("selected.conf");
    std::fs::write(&selected, r##"{
    "metrics": {
        "radius-window": 21,
        "space-m": "invalid",
    },
}
"##).unwrap();
    std::fs::write(
        root.join("installed/iad/themes/default.ron"),
        r##"{
    "metrics": {
        "radius-window": 23,
    },
}
"##,
    )
    .unwrap();
    for base in [false, true] {
        for case in [
            "unset",
            "none",
            "blank",
            "selected",
            "installed",
            "none-installed",
            "unreadable",
            "unreadable-installed",
        ] {
            let mut command = Command::new(std::env::current_exe().unwrap());
            command
                .args(["--exact", "discovery_child", "--nocapture"])
                .env("IAD_BASE_TEST_CASE", case)
                .env("IAD_BASE_TEST_BUNDLED", if base { "yes" } else { "no" })
                .env("HOME", &root)
                .env("XDG_CONFIG_HOME", &root)
                .env("XDG_CONFIG_DIRS", &root)
                .env(
                    "XDG_DATA_HOME",
                    if case.contains("installed") {
                        root.join("installed")
                    } else {
                        root.clone()
                    },
                )
                .env("XDG_DATA_DIRS", &root)
                .env_remove("ICED_ADAPTIVE_DESIGN_CONFIG")
                .env_remove("IAD_THEME");
            match case {
                "none" | "none-installed" => {
                    command.env("IAD_THEME", " none ");
                }
                "blank" => {
                    command.env("IAD_THEME", " ");
                }
                "selected" => {
                    command.env("IAD_THEME", &selected);
                }
                "unreadable" | "unreadable-installed" => {
                    command.env("IAD_THEME", root.join("missing.conf"));
                }
                _ => {}
            }
            let output = command.output().unwrap();
            assert!(
                output.status.success(),
                "base={base} case={case}\n{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            let stderr = String::from_utf8_lossy(&output.stderr);
            if case.starts_with("none") {
                assert!(stderr.is_empty(), "base={base} case={case}: {stderr}");
            }
            if case.starts_with("unreadable") {
                assert!(stderr.contains("looking further"), "{stderr}");
            }
        }
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn discovery_child() {
    let Ok(case) = std::env::var("IAD_BASE_TEST_CASE") else {
        return;
    };
    let base = std::env::var("IAD_BASE_TEST_BUNDLED").unwrap() == "yes";
    let config = if base {
        iad::config::Config::load_with_base_theme(BASE)
    } else {
        iad::config::Config::load()
    };
    iad::config::install(config);
    let expected = match case.as_str() {
        "selected" => 21.0,
        "installed" | "unreadable-installed" => 23.0,
        _ if base => 15.0,
        _ => 0.0,
    };
    assert_eq!(iad::Metrics::desktop().radius_window, expected);
    for scheme in [iad::Scheme::Light, iad::Scheme::Dark] {
        let mut config = (*iad::config::current()).clone();
        config.color_scheme = iad::config::Pick::Fixed(scheme);
        let theme = iad::Theme::from_config(&config);
        assert_eq!(theme.metrics.radius_window, expected);
        assert_eq!(theme.metrics.space_m, iad::Metrics::desktop().space_m);
        assert_eq!(theme.scheme, scheme);
        let expected_bg = match (base, scheme) {
            (true, iad::Scheme::Dark) => iced::Color::from_rgb8(0x22, 0x22, 0x22),
            (false, iad::Scheme::Dark) => iced::Color::BLACK,
            _ => iced::Color::WHITE,
        };
        assert_eq!(theme.palette.window_bg_color, expected_bg);
        if base && case == "selected" {
            assert_eq!(theme.metrics.space_m, 12.0);
        }
    }
}

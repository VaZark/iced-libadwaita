use super::*;

#[test]
fn saving_breaks_typing_group_so_undo_returns_to_saved_values() {
    let d = temp_dir("save-undo");
    let mut e = editor();
    let set = |e: &mut Editor, value: &str| {
        let _ = e.update(Msg::Input(
            "metrics".into(),
            "radius-m".into(),
            value.into(),
        ));
    };
    set(&mut e, "7");
    assert!(e.save_to(d.join("theme.conf")));
    set(&mut e, "9");
    let _ = e.update(Msg::Undo);
    assert!(!e.dirty());
    assert!(e.generate().contains("radius-m=7"));
    let _ = e.update(Msg::Redo);
    assert!(e.dirty());
    assert!(e.generate().contains("radius-m=9"));
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn save_as_adds_conf_extension_and_subsequent_save_updates_the_new_file() {
    let d = temp_dir("extension");
    let original = d.join("original.conf");
    std::fs::write(&original, "[metrics]\nradius-m=1\n").unwrap();
    let mut e = editor();
    e.theme_dirs = vec![d.join("registry")];
    e.load(original.clone());
    let _ = e.update(Msg::SavedAs(Some(d.join("copy"))));
    assert_eq!(e.path, Some(d.join("copy.conf")));
    assert!(is_registered(&d.join("registry"), &d.join("copy.conf")));
    let _ = e.update(Msg::Input("metrics".into(), "radius-m".into(), "8".into()));
    let _ = e.update(Msg::Save);
    assert!(read(&d.join("copy.conf")).contains("radius-m=8"));
    assert_eq!(read(&original), "[metrics]\nradius-m=1\n");
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn registration_cannot_hijack_default_or_create_a_dangling_link() {
    let d = temp_dir("reserved-default");
    let registry = d.join("registry");
    let existing = d.join("existing.conf");
    std::fs::write(&existing, "existing").unwrap();
    set_default(&registry, &existing).unwrap();
    let reserved = d.join("default.conf");
    std::fs::write(&reserved, "new").unwrap();
    assert!(
        register(&registry, &reserved)
            .unwrap_err()
            .contains("reserved")
    );
    assert_eq!(default_target(&registry), Some(canon(&existing)));
    assert!(register(&registry, &d.join("missing.conf")).is_err());
    assert!(!registry.join("missing.conf").exists());
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn toy_ids_preserve_hyphens_and_each_fallback_toy_receives_its_own_id() {
    assert_eq!(
        toys::parse_list("retro-desktop: Retro Desktop")[0].name,
        "retro-desktop"
    );
    let d = temp_dir("every-toy");
    let bin = script(&d, "capture", "printf '%s' \"$1\" > \"$IAD_THEME.arg\"\n");
    let bins = toys::Bins {
        gallery: None,
        toys: Some(bin),
    };
    let mut runner = toys::Runner::default();
    for toy in toys::fallback() {
        let path = runner
            .launch(&toy.name, "[metrics]\nradius-m=7\n", &bins)
            .unwrap();
        let arg = PathBuf::from(format!("{}.arg", path.display()));
        wait_for(&arg);
        assert_eq!(read(&arg), toy.name);
        assert!(read(&path).contains("radius-m=7"));
        std::fs::remove_file(arg).unwrap();
        std::fs::remove_file(path).unwrap();
    }
    drop(runner);
    std::fs::remove_dir_all(d).unwrap();
}

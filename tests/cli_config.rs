mod common;

use tempfile::tempdir;

#[test]
fn config_init_creates_local_file_via_real_dispatch() {
    let dir = tempdir().unwrap();

    let output = common::tk(&["config", "init"], dir.path(), None, None);

    assert!(output.status.success());
    assert_eq!(common::stdout(&output), "Created ./.tick.toml\n");
    assert!(dir.path().join(".tick.toml").is_file());
}

#[test]
fn config_init_refuses_when_local_file_already_exists() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join(".tick.toml"), "custom content").unwrap();

    let output = common::tk(&["config", "init"], dir.path(), None, None);

    assert!(!output.status.success());
    assert!(common::stderr(&output).contains("already exists"));
    assert_eq!(
        std::fs::read_to_string(dir.path().join(".tick.toml")).unwrap(),
        "custom content"
    );
}

#[test]
fn config_init_global_succeeds_when_local_exists_and_leaves_it_untouched() {
    let cwd = tempdir().unwrap();
    let home = tempdir().unwrap();
    std::fs::write(cwd.path().join(".tick.toml"), "custom content").unwrap();

    let output = common::tk_with_home(&["config", "init", "-g"], cwd.path(), home.path());

    assert!(output.status.success());
    assert_eq!(common::stdout(&output), "Created ~/.tick.toml\n");
    assert!(home.path().join(".tick.toml").is_file());
    assert_eq!(
        std::fs::read_to_string(cwd.path().join(".tick.toml")).unwrap(),
        "custom content"
    );
}

#[test]
fn config_init_global_refuses_when_global_file_already_exists() {
    let cwd = tempdir().unwrap();
    let home = tempdir().unwrap();
    std::fs::write(home.path().join(".tick.toml"), "custom content").unwrap();

    let output = common::tk_with_home(&["config", "init", "-g"], cwd.path(), home.path());

    assert!(!output.status.success());
    assert!(common::stderr(&output).contains("already exists"));
    assert_eq!(
        std::fs::read_to_string(home.path().join(".tick.toml")).unwrap(),
        "custom content"
    );
}

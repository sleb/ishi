mod common;

use ishi::config::Config;
use tempfile::tempdir;

#[test]
fn init_bare_creates_workspace_via_real_dispatch() {
    let dir = tempdir().unwrap();

    let output = common::ishi(&["init"], dir.path(), None, None);

    assert!(output.status.success());
    assert_eq!(
        common::stdout(&output),
        "Created PARA system in .\nNext: ishi new to capture your first note.\n"
    );
    for name in Config::default().category_dirs {
        assert!(dir.path().join(name).is_dir());
    }
}

#[test]
fn init_named_creates_subdirectory_via_real_dispatch() {
    let dir = tempdir().unwrap();

    let output = common::ishi(&["init", "my-para"], dir.path(), None, None);

    assert!(output.status.success());
    assert_eq!(
        common::stdout(&output),
        "Created PARA system in ./my-para\nNext: cd my-para && ishi new to capture your first note.\n"
    );
    for name in Config::default().category_dirs {
        assert!(dir.path().join("my-para").join(name).is_dir());
    }
}

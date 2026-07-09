mod common;

use tempfile::tempdir;

#[test]
fn move_relocates_flat_file_into_project_via_real_dispatch() {
    let dir = tempdir().unwrap();
    common::init_workspace(dir.path());
    tick::items::create(
        &tick::workspace::Workspace {
            root: dir.path().to_path_buf(),
            config: tick::config::Config::default(),
        },
        tick::category::Category::Inbox,
        "my-file",
        "hello",
    )
    .unwrap();

    let output = common::tk(&["move", "my-file", "project"], dir.path(), None, None);

    assert!(output.status.success());
    let root = dir.path().canonicalize().unwrap();
    let source_path = root.join("0-Inbox/my-file.md");
    let dest_path = root.join("1-Projects/my-file/index.md");
    assert_eq!(
        common::stdout(&output),
        format!(
            "Moved {} to {}\n",
            source_path.display(),
            dest_path.display()
        )
    );
    assert!(dest_path.is_file());
    assert!(!source_path.exists());
}

#[test]
fn mv_alias_behaves_identically_to_move() {
    let dir = tempdir().unwrap();
    common::init_workspace(dir.path());
    tick::items::create(
        &tick::workspace::Workspace {
            root: dir.path().to_path_buf(),
            config: tick::config::Config::default(),
        },
        tick::category::Category::Inbox,
        "my-file",
        "hello",
    )
    .unwrap();

    let output = common::tk(&["mv", "my-file", "project"], dir.path(), None, None);

    assert!(output.status.success());
    let root = dir.path().canonicalize().unwrap();
    let dest_path = root.join("1-Projects/my-file/index.md");
    assert!(dest_path.is_file());
}

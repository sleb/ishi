mod common;

use tempfile::tempdir;

#[test]
fn review_walks_projects_then_areas_via_real_dispatch() {
    let dir = tempdir().unwrap();
    common::init_workspace(dir.path());
    tick::items::create(
        &tick::workspace::Workspace {
            root: dir.path().to_path_buf(),
            config: tick::config::Config::default(),
        },
        tick::category::Category::Project,
        "website-redesign",
        "# Website Redesign\n",
    )
    .unwrap();
    tick::items::create(
        &tick::workspace::Workspace {
            root: dir.path().to_path_buf(),
            config: tick::config::Config::default(),
        },
        tick::category::Category::Area,
        "health",
        "# Health\n",
    )
    .unwrap();

    let output = common::tk(&["review"], dir.path(), None, Some("k\nk\n"));

    assert!(output.status.success());
    let stdout = common::stdout(&output);
    assert!(stdout.contains("Project: website-redesign (last updated today)"));
    assert!(stdout.contains("Area: health (last updated today)"));
    assert!(stdout.contains("[k]eep  [a]rchive  [s]kip?"));
}

#[test]
fn review_reports_nothing_to_review_on_empty_workspace() {
    let dir = tempdir().unwrap();
    common::init_workspace(dir.path());

    let output = common::tk(&["review"], dir.path(), None, None);

    assert!(output.status.success());
    assert_eq!(common::stdout(&output), "Nothing to review.\n");
}

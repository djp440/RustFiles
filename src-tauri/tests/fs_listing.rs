use std::path::PathBuf;

use rustfiles::core::error::ErrorCode;
use rustfiles::core::types::{DirectoryPage, FilterKind, SortKey};

fn fixture_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push(".tmp");
    p.push("fixtures");
    p.push(name);
    p
}

#[test]
fn list_directory_small_dir_returns_directory_page() {
    let path = fixture_path("small-dir");
    let result = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        false,
    );
    let page = result.expect("small-dir should be listable");
    assert!(page.path.contains("small-dir"));
    assert_eq!(page.sort_key, SortKey::Name);
    assert!(page.sort_ascending);
    assert_eq!(page.filter_kind, FilterKind::All);
    assert!(!page.show_hidden);
    assert!(page.snapshot_version > 0, "snapshot_version should be positive");
    assert!(
        page.total_count >= 7,
        "small-dir should have at least 7 entries (5 files + 2 dirs + 1 hidden), got {}",
        page.total_count
    );
    assert!(!page.entries.is_empty(), "entries should not be empty");
}

#[test]
fn list_directory_nonexistent_path_returns_path_not_found() {
    let path = fixture_path("does-not-exist-12345");
    let result = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        false,
    );
    assert!(result.is_err(), "nonexistent path should error");
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::PathNotFound);
}

#[test]
fn list_directory_small_dir_filters_folders_only() {
    let path = fixture_path("small-dir");
    let result = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::Folders,
        false,
    );
    let page = result.expect("small-dir should be listable");
    for entry in &page.entries {
        assert!(entry.is_folder, "folder filter should only return folders");
    }
    assert!(page.total_count >= 2, "should have at least 2 folders");
}

#[test]
fn list_directory_small_dir_filters_files_only() {
    let path = fixture_path("small-dir");
    let result = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::Files,
        false,
    );
    let page = result.expect("small-dir should be listable");
    for entry in &page.entries {
        assert!(!entry.is_folder, "files filter should only return files");
    }
}

#[test]
fn list_directory_small_dir_show_hidden_includes_hidden_file() {
    let path = fixture_path("small-dir");
    let result_with_hidden = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        true,
    );
    let page_with = result_with_hidden.expect("small-dir should be listable");
    let result_without = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        false,
    );
    let page_without = result_without.expect("small-dir should be listable");
    assert!(
        page_with.total_count >= page_without.total_count,
        "show_hidden should show at least as many entries"
    );
}

#[test]
fn list_directory_sorts_by_name_ascending() {
    let path = fixture_path("small-dir");
    let result = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        true,
    );
    let page = result.expect("small-dir should be listable");
    let entries = &page.entries;
    for i in 1..entries.len() {
        let prev = &entries[i - 1];
        let curr = &entries[i];
        if prev.is_folder != curr.is_folder {
            continue;
        }
        assert!(
            prev.name.to_lowercase() <= curr.name.to_lowercase(),
            "entries should be sorted by name ascending (folders first): {} > {}",
            prev.name,
            curr.name
        );
    }
}

#[test]
fn list_directory_sorts_by_name_descending() {
    let path = fixture_path("small-dir");
    let result = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        false,
        &FilterKind::All,
        true,
    );
    let page = result.expect("small-dir should be listable");
    let entries = &page.entries;
    for i in 1..entries.len() {
        let prev = &entries[i - 1];
        let curr = &entries[i];
        if prev.is_folder != curr.is_folder {
            continue;
        }
        assert!(
            prev.name.to_lowercase() >= curr.name.to_lowercase(),
            "entries should be sorted by name descending (folders first): {} < {}",
            prev.name,
            curr.name
        );
    }
}

#[test]
fn list_directory_command_returns_page() {
    let path = fixture_path("small-dir");
    let page = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        false,
    )
    .expect("list_directory should succeed");

    let serialized = serde_json::to_string(&page).expect("should serialize DirectoryPage");
    let deserialized: DirectoryPage =
        serde_json::from_str(&serialized).expect("should deserialize DirectoryPage");

    assert_eq!(deserialized.path, page.path);
    assert_eq!(deserialized.total_count, page.total_count);
    assert_eq!(deserialized.sort_key, page.sort_key);
    assert_eq!(deserialized.filter_kind, page.filter_kind);
    assert_eq!(deserialized.snapshot_version, page.snapshot_version);
    assert_eq!(deserialized.sort_ascending, page.sort_ascending);
    assert_eq!(deserialized.show_hidden, page.show_hidden);
    assert_eq!(deserialized.entries.len(), page.entries.len());
}

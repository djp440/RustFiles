use std::path::PathBuf;

use rustfiles::core::types::{FilterKind, SortKey};

fn fixture_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push(".tmp");
    p.push("fixtures");
    p.push(name);
    p
}

fn ensure_fixtures_exist() {
    let large_dir = fixture_path("large-10k-dir");
    if !large_dir.exists() {
        panic!(
            "Fixtures not found at {}. Run: powershell -ExecutionPolicy Bypass -File scripts/create-fixtures.ps1 -Root .\\.tmp\\fixtures",
            large_dir.display()
        );
    }
}

#[test]
fn list_directory_large_10k_returns_total_count() {
    ensure_fixtures_exist();
    let path = fixture_path("large-10k-dir");
    let result = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        false,
    );
    let page = result.expect("large-10k-dir should be listable");
    assert_eq!(page.total_count, 10000, "large-10k-dir should have 10000 entries");
}

#[test]
fn list_directory_large_10k_paginated_returns_window() {
    ensure_fixtures_exist();
    let path = fixture_path("large-10k-dir");
    let result = rustfiles::core::fs::list_directory_paginated(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        false,
        0,
        100,
    );
    let page = result.expect("large-10k-dir should be listable");
    assert_eq!(page.total_count, 10000, "total_count should be 10000");
    assert_eq!(page.offset, 0, "offset should be 0");
    assert_eq!(page.limit, 100, "limit should be 100");
    assert!(page.entries.len() <= 100, "entries should be <= 100, got {}", page.entries.len());
    assert!(page.snapshot_version > 0, "snapshot_version should be positive");
}

#[test]
fn list_directory_large_10k_paginated_second_page() {
    ensure_fixtures_exist();
    let path = fixture_path("large-10k-dir");
    let result = rustfiles::core::fs::list_directory_paginated(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        false,
        100,
        100,
    );
    let page = result.expect("large-10k-dir should be listable");
    assert_eq!(page.total_count, 10000, "total_count should be 10000");
    assert_eq!(page.offset, 100, "offset should be 100");
    assert!(page.entries.len() <= 100, "entries should be <= 100, got {}", page.entries.len());
    assert!(page.entries.len() > 0, "second page should have entries");
}

#[test]
fn list_directory_large_10k_paginated_last_page_partial() {
    ensure_fixtures_exist();
    let path = fixture_path("large-10k-dir");
    let result = rustfiles::core::fs::list_directory_paginated(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        false,
        9950,
        100,
    );
    let page = result.expect("large-10k-dir should be listable");
    assert_eq!(page.total_count, 10000, "total_count should be 10000");
    assert_eq!(page.offset, 9950, "offset should be 9950");
    assert_eq!(page.entries.len(), 50, "last page should have 50 entries");
}

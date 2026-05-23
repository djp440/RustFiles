use std::path::PathBuf;

use rustfiles::core::error::ErrorCode;
use rustfiles::core::system;

fn fixture_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push(".tmp");
    p.push("fixtures");
    p.push(name);
    p
}

// ============================================================
// 系统集成 — 回收站
// ============================================================

#[test]
fn recycle_bin_nonexistent_path_returns_path_not_found() {
    let bad = fixture_path("small-dir").join("__no_such_recycle_test__");
    let result = system::delete_to_recycle_bin(&bad.to_string_lossy());
    assert!(result.is_err(), "non-existent path must return error");
    assert_eq!(result.unwrap_err().code, ErrorCode::PathNotFound);
}

// ============================================================
// 系统集成 — 默认应用打开
// ============================================================

#[test]
fn default_app_open_nonexistent_path_returns_path_not_found() {
    let bad = fixture_path("small-dir").join("__no_such_app_open_test__");
    let result = system::open_with_default_app(&bad.to_string_lossy());
    assert!(result.is_err(), "non-existent path must return error");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, ErrorCode::PathNotFound,
        "expected PathNotFound for missing file, got {:?}",
        err.code
    );
}

// ============================================================
// 系统集成 — 终端打开
// ============================================================

#[test]
fn terminal_open_nonexistent_path_returns_path_not_found() {
    let bad = fixture_path("small-dir").join("__no_such_terminal_test__");
    let result = system::open_terminal(&bad.to_string_lossy());
    assert!(result.is_err(), "non-existent path must return error");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, ErrorCode::PathNotFound,
        "expected PathNotFound for missing path, got {:?}",
        err.code
    );
}

// ============================================================
// 系统集成 — 属性打开
// ============================================================

#[test]
fn properties_open_nonexistent_path_returns_path_not_found() {
    let bad = fixture_path("small-dir").join("__no_such_properties_test__");
    let result = system::show_properties(&bad.to_string_lossy());
    assert!(result.is_err(), "non-existent path must return error");
    let err = result.unwrap_err();
    assert_eq!(
        err.code, ErrorCode::PathNotFound,
        "expected PathNotFound for missing path, got {:?}",
        err.code
    );
}

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rustfiles::core::error::ErrorCode;
use rustfiles::core::system;
use rustfiles::core::tasks;
use rustfiles::core::types::TaskStatus;

static TEST_COUNTER: AtomicU64 = AtomicU64::new(1);

fn op_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push(".tmp");
    p.push("fixtures");
    p.push("small-dir");
    p
}

fn unique_name(prefix: &str) -> String {
    let seq = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}_{}", prefix, seq)
}

// ============================================================
// create_folder
// ============================================================

#[test]
fn create_folder_returns_task_id_and_creates_dir() {
    let parent = op_dir();
    let name = unique_name("__test_new_folder__");
    let target = parent.join(&name);
    let _ = std::fs::remove_dir(&target);

    let result = tasks::execute_create_folder(&parent.to_string_lossy(), &name, None);
    assert!(result.is_ok(), "create_folder should succeed: {:?}", result.err());
    let task_id = result.unwrap();
    assert!(!task_id.as_str().is_empty(), "task_id should not be empty");

    assert!(target.exists(), "folder should exist on disk");
    assert!(target.is_dir(), "should be a directory");

    let task = tasks::get_file_op_task_status(task_id.as_str());
    assert!(task.is_some(), "task should exist in registry");
    assert_eq!(task.unwrap().status, TaskStatus::Completed);

    let _ = std::fs::remove_dir(&target);
}

#[test]
fn create_folder_rejects_invalid_name() {
    let parent = op_dir();
    let result = tasks::execute_create_folder(&parent.to_string_lossy(), "CON", None);
    assert!(result.is_err(), "CON is a reserved name");
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::InternalError);
}

// ============================================================
// rename_item
// ============================================================

#[test]
fn rename_item_returns_task_id_and_renames() {
    let dir = op_dir();
    let old_name = unique_name("__test_rename_src__");
    let new_name = unique_name("__test_rename_dst__");
    let old_path = dir.join(&old_name);
    let new_path = dir.join(&new_name);

    std::fs::write(&old_path, "rename test content").expect("write temp file");
    let _ = std::fs::remove_file(&new_path);
    assert!(old_path.exists(), "source must exist");

    let result = tasks::execute_rename_item(&old_path.to_string_lossy(), &new_name, None);
    assert!(result.is_ok(), "rename should succeed: {:?}", result.err());
    let task_id = result.unwrap();
    assert!(!task_id.as_str().is_empty());

    assert!(new_path.exists(), "target should exist after rename");
    assert!(!old_path.exists(), "source should not exist after rename");

    let task = tasks::get_file_op_task_status(task_id.as_str());
    assert!(task.is_some());
    assert_eq!(task.unwrap().status, TaskStatus::Completed);

    let _ = std::fs::remove_file(&new_path);
}

#[test]
fn rename_item_rejects_invalid_name() {
    let dir = op_dir();
    let name = unique_name("__test_rename_invalid_src__");
    let path = dir.join(&name);
    std::fs::write(&path, "content").expect("write temp file");

    let result = tasks::execute_rename_item(&path.to_string_lossy(), "n<ul>", None);
    assert!(result.is_err(), "illegal chars should be rejected");
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::InternalError);

    let _ = std::fs::remove_file(&path);
}

// ============================================================
// delete_to_recycle_bin
// ============================================================

#[test]
fn delete_to_recycle_bin_returns_task_id() {
    let dir = op_dir();
    let name = unique_name("__test_recycle__");
    let path = dir.join(&name);
    std::fs::write(&path, "recycle me").expect("write temp file");
    assert!(path.exists());

    let result = tasks::execute_delete_to_recycle_bin(&path.to_string_lossy(), None);
    assert!(result.is_ok(), "delete_to_recycle_bin should succeed: {:?}", result.err());
    let task_id = result.unwrap();
    assert!(!task_id.as_str().is_empty());

    let task = tasks::get_file_op_task_status(task_id.as_str());
    assert!(task.is_some());
    assert_eq!(task.unwrap().status, TaskStatus::Completed);
}

#[test]
fn delete_to_recycle_bin_nonexistent_path_fails() {
    let dir = op_dir();
    let bad = dir.join("__no_such_file_ever__");
    let result = tasks::execute_delete_to_recycle_bin(&bad.to_string_lossy(), None);
    assert!(result.is_err(), "non-existent path should fail");
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::PathNotFound);
}

// ============================================================
// delete_permanently
// ============================================================

#[test]
fn delete_permanently_removes_file() {
    let dir = op_dir();
    let name = unique_name("__test_perm_delete__");
    let path = dir.join(&name);
    std::fs::write(&path, "permanent delete me").expect("write temp file");
    assert!(path.exists());

    let result = tasks::execute_delete_permanently(&path.to_string_lossy(), None);
    assert!(result.is_ok(), "permanent delete should succeed: {:?}", result.err());

    assert!(!path.exists(), "file should be gone after permanent delete");

    let task = tasks::get_file_op_task_status(result.unwrap().as_str());
    assert!(task.is_some());
    assert_eq!(task.unwrap().status, TaskStatus::Completed);
}

#[test]
fn delete_permanently_nonexistent_path_fails() {
    let dir = op_dir();
    let bad = dir.join("__no_such_file_perm__");
    let result = tasks::execute_delete_permanently(&bad.to_string_lossy(), None);
    assert!(result.is_err(), "non-existent path should fail");
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::PathNotFound);
}

// ============================================================
// open_with_default_app (system integration)
// ============================================================

#[test]
fn open_with_default_app_existing_file_succeeds() {
    let dir = op_dir();
    let name = unique_name("__test_open_app__");
    let path = dir.join(&name);
    std::fs::write(&path, "open me").expect("write temp file");

    let result = system::open_with_default_app(&path.to_string_lossy());
    assert!(result.is_ok(), "open_with_default_app should succeed: {:?}", result.err());

    let _ = std::fs::remove_file(&path);
}

#[test]
fn open_with_default_app_nonexistent_path_fails() {
    let dir = op_dir();
    let bad = dir.join("__no_such_open__");
    let result = system::open_with_default_app(&bad.to_string_lossy());
    assert!(result.is_err(), "non-existent path should fail");
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::PathNotFound);
}

// ============================================================
// open_terminal (system integration)
// ============================================================

#[test]
fn open_terminal_existing_dir_succeeds() {
    let dir = op_dir();
    let result = system::open_terminal(&dir.to_string_lossy());
    assert!(result.is_ok(), "open_terminal on existing dir should succeed: {:?}", result.err());
}

#[test]
fn open_terminal_nonexistent_path_fails() {
    let dir = op_dir();
    let bad = dir.join("__no_such_dir_for_term__");
    let result = system::open_terminal(&bad.to_string_lossy());
    assert!(result.is_err(), "non-existent path should fail");
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::PathNotFound);
}

// ============================================================
// show_properties (system integration)
// ============================================================

#[test]
fn show_properties_existing_path_succeeds() {
    let dir = op_dir();
    let name = unique_name("__test_props__");
    let path = dir.join(&name);
    std::fs::write(&path, "props").expect("write temp file");

    let result = system::show_properties(&path.to_string_lossy());
    // Properties dialog may not open in all environments (CI, headless, etc.)
    // The contract is: if it fails, it should be PropertiesOpenFailed
    match result {
        Ok(_) => {},
        Err(e) => assert_eq!(e.code, ErrorCode::PropertiesOpenFailed,
            "expected PropertiesOpenFailed on failure, got {:?}", e.code),
    }

    let _ = std::fs::remove_file(&path);
}

#[test]
fn show_properties_nonexistent_path_fails() {
    let dir = op_dir();
    let bad = dir.join("__no_such_props__");
    let result = system::show_properties(&bad.to_string_lossy());
    assert!(result.is_err(), "non-existent path should fail");
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::PathNotFound);
}

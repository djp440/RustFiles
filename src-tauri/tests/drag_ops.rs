use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rustfiles::core::drag;

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

fn create_test_file(dir: &PathBuf, name: &str) -> String {
    let path = dir.join(name);
    let _ = std::fs::remove_file(&path);
    std::fs::write(&path, "test content").expect("write test file");
    path.to_string_lossy().to_string()
}

fn create_test_dir(dir: &PathBuf, name: &str) -> String {
    let path = dir.join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir(&path).expect("create test dir");
    let subfile = path.join("subfile.txt");
    std::fs::write(&subfile, "sub content").expect("write subfile");
    path.to_string_lossy().to_string()
}

// ============================================================
// create_operation
// ============================================================

#[test]
fn create_drag_operation_saves_paths_and_type() {
    let dir = op_dir();
    let src1 = create_test_file(&dir, &unique_name("__test_drag_src1__"));
    let src2 = create_test_file(&dir, &unique_name("__test_drag_src2__"));
    let sources = vec![src1.clone(), src2.clone()];

    let op_id = drag::create_operation(sources.clone(), "move", "tab-1");
    assert!(!op_id.is_empty(), "operation_id should not be empty");

    let op = drag::get_operation(&op_id);
    assert!(op.is_some(), "operation should exist");
    let op = op.unwrap();
    assert_eq!(op.source_paths, sources);
    assert_eq!(op.drag_type, drag::DragOpType::Move);
    assert_eq!(op.source_tab_id, "tab-1");
    assert!(op.created_at > 0, "created_at should be set");
    assert_eq!(op.status, drag::DragOpStatus::Active);
}

#[test]
fn create_drag_operation_with_copy_type() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_drag_copy__"));
    let sources = vec![src];

    let op_id = drag::create_operation(sources, "copy", "tab-2");
    let op = drag::get_operation(&op_id).expect("operation should exist");
    assert_eq!(op.drag_type, drag::DragOpType::Copy);
}

#[test]
fn create_drag_operation_defaults_to_move() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_drag_default__"));
    let sources = vec![src];

    let op_id = drag::create_operation(sources, "unknown", "tab-1");
    let op = drag::get_operation(&op_id).expect("operation should exist");
    assert_eq!(op.drag_type, drag::DragOpType::Move);
}

// ============================================================
// get_operation
// ============================================================

#[test]
fn get_operation_nonexistent_returns_none() {
    let op = drag::get_operation("nonexistent-op-id");
    assert!(op.is_none(), "nonexistent operation should return None");
}

// ============================================================
// delete_operation
// ============================================================

#[test]
fn delete_operation_removes_from_registry() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_drag_del__"));
    let op_id = drag::create_operation(vec![src], "move", "tab-1");

    assert!(drag::get_operation(&op_id).is_some());
    drag::delete_operation(&op_id);
    assert!(drag::get_operation(&op_id).is_none());
}

// ============================================================
// mark_dropped
// ============================================================

#[test]
fn mark_dropped_changes_status() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_drag_dropped__"));
    let op_id = drag::create_operation(vec![src], "move", "tab-1");

    drag::mark_dropped(&op_id);
    let op = drag::get_operation(&op_id).expect("operation should exist");
    assert_eq!(op.status, drag::DragOpStatus::Dropped);
}

// ============================================================
// Status migration
// ============================================================

#[test]
fn drag_op_status_migration_active_to_dropped() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_drag_migrate__"));
    let op_id = drag::create_operation(vec![src], "copy", "tab-1");

    let op = drag::get_operation(&op_id).expect("operation should exist");
    assert_eq!(op.status, drag::DragOpStatus::Active);

    drag::mark_dropped(&op_id);
    let op = drag::get_operation(&op_id).expect("operation should exist");
    assert_eq!(op.status, drag::DragOpStatus::Dropped);
}

#[test]
fn drag_op_dropped_twice_is_idempotent() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_drag_double__"));
    let op_id = drag::create_operation(vec![src], "move", "tab-1");

    drag::mark_dropped(&op_id);
    drag::mark_dropped(&op_id);
    let op = drag::get_operation(&op_id).expect("operation should exist");
    assert_eq!(op.status, drag::DragOpStatus::Dropped);
}

// ============================================================
// delete after dropped
// ============================================================

#[test]
fn delete_operation_after_dropped_succeeds() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_drag_del_after__"));
    let op_id = drag::create_operation(vec![src], "move", "tab-1");

    drag::mark_dropped(&op_id);
    assert!(drag::get_operation(&op_id).is_some());

    drag::delete_operation(&op_id);
    assert!(drag::get_operation(&op_id).is_none());
}

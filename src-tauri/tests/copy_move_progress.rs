use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

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

fn create_test_file(dir: &PathBuf, name: &str) -> String {
    let path = dir.join(name);
    let _ = std::fs::remove_file(&path);
    std::fs::write(&path, "test content").expect("write test file");
    path.to_string_lossy().to_string()
}

// ============================================================
// progress_current / progress_total
// ============================================================

#[test]
fn copy_items_progress_reflects_completed_total() {
    let dir = op_dir();
    let src1 = create_test_file(&dir, &unique_name("__test_prog_src1__"));
    let src2 = create_test_file(&dir, &unique_name("__test_prog_src2__"));
    let target_dir_name = unique_name("__test_prog_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_copy_items(&[src1, src2], &target.to_string_lossy(), None);
    assert!(result.is_ok(), "copy should succeed: {:?}", result.err());
    let task_id = result.unwrap();

    let task = tasks::get_file_op_task_status(task_id.as_str());
    assert!(task.is_some(), "task should exist in registry");
    let task = task.unwrap();

    assert_eq!(task.progress_total, 2, "total should be 2");
    assert_eq!(task.progress_current, 2, "current should be 2 (all completed)");
    assert_eq!(task.status, TaskStatus::Completed);

    let _ = std::fs::remove_dir_all(&target);
}

#[test]
fn copy_items_single_file_progress_one() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_prog_single__"));
    let target_dir_name = unique_name("__test_prog_single_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_copy_items(&[src], &target.to_string_lossy(), None);
    assert!(result.is_ok());
    let task = tasks::get_file_op_task_status(result.unwrap().as_str()).expect("task should exist");

    assert_eq!(task.progress_total, 1);
    assert_eq!(task.progress_current, 1);

    let _ = std::fs::remove_dir_all(&target);
}

// ============================================================
// PartiallyCompleted items
// ============================================================

#[test]
fn copy_items_partially_completed_has_correct_items() {
    let dir = op_dir();
    let src1 = create_test_file(&dir, &unique_name("__test_part_prog1__"));
    let bad = dir.join("__no_such_part_prog__").to_string_lossy().to_string();
    let target_dir_name = unique_name("__test_part_prog_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_copy_items(&[src1.clone(), bad.clone()], &target.to_string_lossy(), None);
    assert!(result.is_ok(), "partial copy should return ok");
    let task_id = result.unwrap();

    let task = tasks::get_file_op_task_status(task_id.as_str());
    assert!(task.is_some(), "task should exist");
    let task = task.unwrap();

    assert_eq!(task.status, TaskStatus::PartiallyCompleted);
    assert_eq!(task.progress_total, 2);
    assert_eq!(task.progress_current, 1);

    // Verify the task file op registry entry
    let file_task = tasks::get_file_op_task_status(task_id.as_str());
    assert!(file_task.is_some());
    let file_task = file_task.unwrap();
    assert_eq!(file_task.progress_current, 1);
    assert_eq!(file_task.progress_total, 2);
    assert_eq!(file_task.status, TaskStatus::PartiallyCompleted);

    let _ = std::fs::remove_dir_all(&target);
}

#[test]
fn copy_items_partial_failure_sets_progress() {
    let dir = op_dir();
    let src1 = create_test_file(&dir, &unique_name("__test_prog_part1__"));
    let bad = dir.join("__no_such_prog__").to_string_lossy().to_string();
    let target_dir_name = unique_name("__test_prog_part_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_copy_items(&[src1, bad], &target.to_string_lossy(), None);
    assert!(result.is_ok());
    let task = tasks::get_file_op_task_status(result.unwrap().as_str()).expect("task should exist");

    assert_eq!(task.status, TaskStatus::PartiallyCompleted);
    assert_eq!(task.progress_current, 1);
    assert_eq!(task.progress_total, 2);

    let _ = std::fs::remove_dir_all(&target);
}

// ============================================================
// move items progress
// ============================================================

#[test]
fn move_items_progress_reflects_copy_progress() {
    let dir = op_dir();
    let src1 = create_test_file(&dir, &unique_name("__test_move_prog1__"));
    let src2 = create_test_file(&dir, &unique_name("__test_move_prog2__"));
    let target_dir_name = unique_name("__test_move_prog_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_move_items(&[src1, src2], &target.to_string_lossy(), None);
    assert!(result.is_ok(), "move should succeed: {:?}", result.err());
    let task = tasks::get_file_op_task_status(result.unwrap().as_str()).expect("task should exist");

    assert_eq!(task.progress_total, 2);
    assert_eq!(task.progress_current, 2);
    assert_eq!(task.status, TaskStatus::Completed);

    let _ = std::fs::remove_dir_all(&target);
}

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use rustfiles::core::clipboard;
use rustfiles::core::error::ErrorCode;
use rustfiles::core::tasks;

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
// create_clipboard_operation
// ============================================================

#[test]
fn create_clipboard_operation_saves_paths_and_type() {
    let dir = op_dir();
    let src1 = create_test_file(&dir, &unique_name("__test_cb_src1__"));
    let src2 = create_test_file(&dir, &unique_name("__test_cb_src2__"));
    let sources = vec![src1.clone(), src2.clone()];

    let op_id = clipboard::create_operation(sources.clone(), "copy", "tab-1");
    assert!(!op_id.is_empty(), "operation_id should not be empty");

    let op = clipboard::get_operation(&op_id);
    assert!(op.is_some(), "operation should exist");
    let op = op.unwrap();
    assert_eq!(op.source_paths, sources);
    assert_eq!(op.op_type, clipboard::ClipOpType::Copy);
    assert_eq!(op.source_tab_id, "tab-1");
    assert!(op.created_at > 0, "created_at should be set");
    assert_eq!(op.status, clipboard::ClipOpStatus::Active);
}

#[test]
fn create_clipboard_operation_with_cut_type() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_cb_src_cut__"));
    let sources = vec![src];

    let op_id = clipboard::create_operation(sources.clone(), "cut", "tab-2");
    let op = clipboard::get_operation(&op_id).expect("operation should exist");
    assert_eq!(op.op_type, clipboard::ClipOpType::Cut);
}

// ============================================================
// get_operation
// ============================================================

#[test]
fn get_operation_nonexistent_returns_none() {
    let op = clipboard::get_operation("nonexistent-op-id");
    assert!(op.is_none(), "nonexistent operation should return None");
}

// ============================================================
// delete_operation
// ============================================================

#[test]
fn delete_operation_removes_from_registry() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_cb_del__"));
    let op_id = clipboard::create_operation(vec![src], "copy", "tab-1");

    assert!(clipboard::get_operation(&op_id).is_some());
    clipboard::delete_operation(&op_id);
    assert!(clipboard::get_operation(&op_id).is_none());
}

// ============================================================
// mark_pasted
// ============================================================

#[test]
fn mark_pasted_changes_status() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_cb_pasted__"));
    let op_id = clipboard::create_operation(vec![src], "copy", "tab-1");

    clipboard::mark_pasted(&op_id);
    let op = clipboard::get_operation(&op_id).expect("operation should exist");
    assert_eq!(op.status, clipboard::ClipOpStatus::Pasted);
}

// ============================================================
// execute_copy_items
// ============================================================

#[test]
fn execute_copy_items_single_file() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_copy_file__"));
    let target_dir = unique_name("__test_copy_target__");
    let target = dir.join(&target_dir);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_copy_items(&[src.clone()], &target.to_string_lossy(), None);
    assert!(result.is_ok(), "copy should succeed: {:?}", result.err());

    // 检查文件是否已复制
    let src_buf = PathBuf::from(&src);
    let src_name = src_buf.file_name().unwrap();
    let copied = target.join(src_name);
    assert!(copied.exists(), "copied file should exist");
    assert!(copied.is_file(), "should be a file");

    let task = tasks::get_file_op_task_status(result.unwrap().as_str());
    assert!(task.is_some());
    assert_eq!(task.unwrap().status, rustfiles::core::types::TaskStatus::Completed);

    let _ = std::fs::remove_dir_all(&target);
}

#[test]
fn execute_copy_items_directory() {
    let dir = op_dir();
    let dir_name = unique_name("__test_copy_dir__");
    let src_dir = create_test_dir(&dir, &dir_name);
    let target_dir_name = unique_name("__test_copy_dir_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_copy_items(&[src_dir.clone()], &target.to_string_lossy(), None);
    assert!(result.is_ok(), "copy dir should succeed: {:?}", result.err());

    let src_dir_buf = PathBuf::from(&src_dir);
    let dir_name_only = src_dir_buf.file_name().unwrap();
    let copied_dir = target.join(dir_name_only);
    assert!(copied_dir.exists(), "copied dir should exist");
    assert!(copied_dir.is_dir(), "should be a dir");

    let subfile = copied_dir.join("subfile.txt");
    assert!(subfile.exists(), "subfile should be copied");

    let _ = std::fs::remove_dir_all(&target);
}

#[test]
fn execute_copy_items_source_not_found() {
    let dir = op_dir();
    let bad_path = dir.join("__no_such_source__");
    let target = dir.join("__copy_target__");
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_copy_items(&[bad_path.to_string_lossy().to_string()], &target.to_string_lossy(), None);
    assert!(result.is_err(), "nonexistent source should fail");
    let err = result.unwrap_err();
    assert_eq!(err.code, ErrorCode::PathNotFound);

    let _ = std::fs::remove_dir_all(&target);
}

#[test]
fn execute_copy_items_target_already_exists() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_copy_exists_src__"));
    let target_dir_name = unique_name("__test_copy_exists_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    // 先复制一次
    let _ = tasks::execute_copy_items(&[src.clone()], &target.to_string_lossy(), None);

    // 第二次复制应返回冲突错误
    let result = tasks::execute_copy_items(&[src.clone()], &target.to_string_lossy(), None);
    // 我们期望返回 TargetAlreadyExists 错误
    assert!(result.is_err(), "duplicate copy should fail");
    assert_eq!(result.unwrap_err().code, ErrorCode::TargetAlreadyExists);

    let _ = std::fs::remove_dir_all(&target);
}

#[test]
fn execute_copy_items_partial_failure() {
    let dir = op_dir();
    let src1 = create_test_file(&dir, &unique_name("__test_partial_src1__"));
    let bad = dir.join("__no_such_partial__").to_string_lossy().to_string();
    let target_dir_name = unique_name("__test_partial_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_copy_items(&[src1.clone(), bad], &target.to_string_lossy(), None);
    assert!(result.is_ok(), "partial copy should return ok with PartiallyCompleted");

    let task = tasks::get_file_op_task_status(result.unwrap().as_str());
    assert!(task.is_some());
    assert_eq!(task.unwrap().status, rustfiles::core::types::TaskStatus::PartiallyCompleted);

    let _ = std::fs::remove_dir_all(&target);
}

#[test]
fn execute_copy_items_all_fail() {
    let dir = op_dir();
    let bad1 = dir.join("__no_such_fail1__").to_string_lossy().to_string();
    let bad2 = dir.join("__no_such_fail2__").to_string_lossy().to_string();
    let target = dir.join("__fail_target__");
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_copy_items(&[bad1, bad2], &target.to_string_lossy(), None);
    assert!(result.is_err(), "all fail should return error");
    assert_eq!(result.unwrap_err().code, ErrorCode::PathNotFound);

    let _ = std::fs::remove_dir_all(&target);
}

// ============================================================
// execute_move_items
// ============================================================

#[test]
fn execute_move_items_single_file() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_move_file__"));
    let target_dir_name = unique_name("__test_move_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_move_items(&[src.clone()], &target.to_string_lossy(), None);
    assert!(result.is_ok(), "move should succeed: {:?}", result.err());

    // 源文件应该被删除
    assert!(!PathBuf::from(&src).exists(), "source should be deleted after move");

    // 目标文件应该存在
    let src_buf = PathBuf::from(&src);
    let src_name = src_buf.file_name().unwrap();
    let moved = target.join(src_name);
    assert!(moved.exists(), "moved file should exist at target");

    let task = tasks::get_file_op_task_status(result.unwrap().as_str());
    assert!(task.is_some());
    assert_eq!(task.unwrap().status, rustfiles::core::types::TaskStatus::Completed);

    let _ = std::fs::remove_dir_all(&target);
}

#[test]
fn execute_move_items_copy_succeeds_source_retained_on_partial_failure() {
    let dir = op_dir();
    let src1 = create_test_file(&dir, &unique_name("__test_move_partial1__"));
    let bad = dir.join("__no_such_move_partial__").to_string_lossy().to_string();
    let target_dir_name = unique_name("__test_move_partial_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let result = tasks::execute_move_items(&[src1.clone(), bad], &target.to_string_lossy(), None);
    // 部分成功但移动失败——不删除成功复制的源文件
    assert!(result.is_ok(), "partial move should still return ok");

    // 成功复制的源应该保留（因为移动操作失败时不删除源）
    assert!(PathBuf::from(&src1).exists(), "successfully copied source should be retained on partial move failure");

    let task = tasks::get_file_op_task_status(result.unwrap().as_str());
    assert!(task.is_some());
    assert_eq!(task.unwrap().status, rustfiles::core::types::TaskStatus::PartiallyCompleted);

    let _ = std::fs::remove_dir_all(&target);
}

// ============================================================
// 完整 clipboard 操作流程
// ============================================================

#[test]
fn clipboard_copy_then_paste_retains_source() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_flow_src__"));
    let target_dir_name = unique_name("__test_flow_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let op_id = clipboard::create_operation(vec![src.clone()], "copy", "tab-1");
    let op = clipboard::get_operation(&op_id).expect("operation should exist");

    let result = tasks::execute_copy_items(&op.source_paths, &target.to_string_lossy(), None);
    assert!(result.is_ok(), "paste copy should succeed");

    // 源应该保留
    assert!(PathBuf::from(&src).exists(), "source should be retained after copy paste");
    clipboard::mark_pasted(&op_id);

    let _ = std::fs::remove_dir_all(&target);
}

#[test]
fn clipboard_cut_then_paste_deletes_source() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_flow_cut_src__"));
    let target_dir_name = unique_name("__test_flow_cut_target__");
    let target = dir.join(&target_dir_name);
    let _ = std::fs::remove_dir_all(&target);
    std::fs::create_dir_all(&target).expect("create target dir");

    let op_id = clipboard::create_operation(vec![src.clone()], "cut", "tab-1");
    let op = clipboard::get_operation(&op_id).expect("operation should exist");

    let result = tasks::execute_move_items(&op.source_paths, &target.to_string_lossy(), None);
    assert!(result.is_ok(), "paste cut should succeed");

    // 源应该被删除
    assert!(!PathBuf::from(&src).exists(), "source should be deleted after cut paste");
    clipboard::mark_pasted(&op_id);

    let _ = std::fs::remove_dir_all(&target);
}

#[test]
fn paste_nonexistent_operation_id_fails() {
    let dir = op_dir();
    let target = dir.join("__no_target_dir__");
    std::fs::create_dir_all(&target).expect("create target dir");

    // 尝试获取不存在的操作
    let op = clipboard::get_operation("nonexistent");
    assert!(op.is_none(), "nonexistent operation should not be found");

    let _ = std::fs::remove_dir_all(&target);
}

#[test]
fn paste_twice_is_rejected() {
    let dir = op_dir();
    let src = create_test_file(&dir, &unique_name("__test_double_paste__"));
    let target1_name = unique_name("__test_dp_target1__");
    let target2_name = unique_name("__test_dp_target2__");
    let target1 = dir.join(&target1_name);
    let target2 = dir.join(&target2_name);
    std::fs::create_dir(&target1).expect("create target1");
    std::fs::create_dir(&target2).expect("create target2");

    let op_id = clipboard::create_operation(vec![src.clone()], "copy", "tab-1");

    // 第一次粘贴成功
    let op = clipboard::get_operation(&op_id).expect("operation should exist");
    let result1 = tasks::execute_copy_items(&op.source_paths, &target1.to_string_lossy(), None);
    assert!(result1.is_ok(), "first paste should succeed");
    clipboard::mark_pasted(&op_id);

    // 检查操作已被标记为 Pasted
    let op_after = clipboard::get_operation(&op_id).expect("operation should still exist");
    assert_eq!(op_after.status, clipboard::ClipOpStatus::Pasted, "operation should be Pasted after first paste");

    let _ = std::fs::remove_dir_all(&target1);
    let _ = std::fs::remove_dir_all(&target2);
}

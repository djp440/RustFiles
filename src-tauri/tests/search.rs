use std::path::PathBuf;
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use rustfiles::core::error::ErrorCode;
use rustfiles::core::search::{self, SearchRequest, TaskId};
use rustfiles::core::types::TaskStatus;

static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
static FIXTURE_ROOT: OnceLock<PathBuf> = OnceLock::new();

fn test_lock() -> &'static Mutex<()> {
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

fn acquire_test_guard() -> std::sync::MutexGuard<'static, ()> {
    test_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri should have project root parent")
        .to_path_buf()
}

fn fixture_root() -> &'static PathBuf {
    FIXTURE_ROOT.get_or_init(|| {
        let root = project_root().join(".tmp").join("search_fixtures");
        let script = project_root().join("scripts").join("create-fixtures.ps1");
        let script_str = script.to_string_lossy().to_string();
        let root_str = root.to_string_lossy().to_string();
        let output = Command::new("powershell")
            .args([
                "-ExecutionPolicy",
                "Bypass",
                "-File",
                &script_str,
                "-Root",
                &root_str,
            ])
            .output()
            .expect("failed to execute create-fixtures.ps1");
        assert!(
            output.status.success(),
            "fixture generation failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        root
    })
}

fn fixture_path(name: &str) -> PathBuf {
    fixture_root().join(name)
}

fn wait_for_snapshot<F>(
    task_id: &TaskId,
    timeout: Duration,
    predicate: F,
) -> search::SearchTaskSnapshot
where
    F: Fn(&TaskStatus) -> bool,
{
    let start = Instant::now();
    loop {
        if let Some(snapshot) = search::get_task_snapshot(task_id) {
            if predicate(&snapshot.status) {
                return snapshot;
            }
        }

        if start.elapsed() > timeout {
            return search::get_task_snapshot(task_id)
                .unwrap_or_else(|| panic!("task snapshot missing after {:?}", timeout));
        }

        std::thread::sleep(Duration::from_millis(20));
    }
}

fn wait_for_terminal_snapshot(task_id: &TaskId, timeout: Duration) -> search::SearchTaskSnapshot {
    wait_for_snapshot(task_id, timeout, |status| {
        matches!(
            status,
            TaskStatus::Cancelled
                | TaskStatus::Completed
                | TaskStatus::Failed
                | TaskStatus::PartiallyCompleted
        )
    })
}

fn search_paths(snapshot: &search::SearchTaskSnapshot) -> Vec<String> {
    snapshot
        .batches
        .iter()
        .flat_map(|batch| batch.matches.iter().map(|entry| entry.path.clone()))
        .collect()
}

struct PermissionAclGuard {
    path: PathBuf,
    identity: String,
}

impl PermissionAclGuard {
    fn deny_read(path: PathBuf) -> Self {
        let identity = current_identity();
        let deny_rule = format!("{}:(RX)", identity);
        let output = Command::new("icacls")
            .arg(&path)
            .args(["/deny", &deny_rule])
            .output()
            .expect("failed to execute icacls /deny");
        assert!(
            output.status.success(),
            "icacls deny failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Self { path, identity }
    }
}

impl Drop for PermissionAclGuard {
    fn drop(&mut self) {
        let remove_rule = self.identity.clone();
        let _ = Command::new("icacls")
            .arg(&self.path)
            .args(["/remove:d", &remove_rule, "/T", "/C", "/Q"])
            .output();
    }
}

fn current_identity() -> String {
    let output = Command::new("whoami")
        .output()
        .expect("failed to execute whoami");
    assert!(
        output.status.success(),
        "whoami failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn non_recursive_search_uses_current_directory_snapshot() {
    let _guard = acquire_test_guard();
    let root = fixture_path("small-dir");
    let task_id = search::start_search(SearchRequest {
        root_path: root.to_string_lossy().to_string(),
        query: "file_1.txt".to_string(),
        recursive: false,
        snapshot_version: Some(123),
    })
    .expect("start_search should succeed");

    let snapshot = wait_for_terminal_snapshot(&task_id, Duration::from_secs(5));
    assert_eq!(snapshot.status, TaskStatus::Completed);
    assert_eq!(snapshot.batches.len(), 1, "non-recursive search should emit one batch");

    let batch = &snapshot.batches[0];
    assert_eq!(batch.task_id, task_id);
    assert_eq!(batch.root_path, root.to_string_lossy().to_string());
    assert_eq!(batch.query, "file_1.txt");
    assert!(!batch.recursive);
    assert_eq!(batch.snapshot_version, Some(123));
    assert!(batch.error_summaries.is_empty());

    let paths = search_paths(&snapshot);
    assert_eq!(paths.len(), 1);
    assert!(paths[0].ends_with("file_1.txt"));
    assert!(paths[0].contains("small-dir"));
}

#[test]
fn recursive_search_walks_deep_tree_and_batches_results() {
    let _guard = acquire_test_guard();
    let root = fixture_path("deep-tree");
    let task_id = search::start_search(SearchRequest {
        root_path: root.to_string_lossy().to_string(),
        query: "file".to_string(),
        recursive: true,
        snapshot_version: Some(456),
    })
    .expect("start_search should succeed");

    let snapshot = wait_for_terminal_snapshot(&task_id, Duration::from_secs(10));
    assert_eq!(snapshot.status, TaskStatus::Completed);
    assert!(
        snapshot.batches.len() >= 2,
        "recursive search should batch large result sets"
    );

    let paths = search_paths(&snapshot);
    assert!(
        paths.len() >= 20,
        "recursive search should find nested files, got {}",
        paths.len()
    );
    assert!(
        paths.iter().any(|path| path.contains("leaf_bundle")),
        "recursive search should include deep nested results"
    );
    assert_eq!(snapshot.batches[0].snapshot_version, Some(456));
}

#[test]
fn cancel_recursive_search_stops_background_task() {
    let _guard = acquire_test_guard();
    let root = fixture_path("large-10k-dir");
    let task_id = search::start_search(SearchRequest {
        root_path: root.to_string_lossy().to_string(),
        query: "file_".to_string(),
        recursive: true,
        snapshot_version: None,
    })
    .expect("start_search should succeed");

    let cancel_status = search::cancel_task(&task_id).expect("cancel_task should succeed");
    assert!(
        matches!(cancel_status, TaskStatus::Cancelling | TaskStatus::Cancelled | TaskStatus::PartiallyCompleted),
        "cancel_task should return a cancellable or terminal status, got {:?}",
        cancel_status
    );

    let final_snapshot = wait_for_terminal_snapshot(&task_id, Duration::from_secs(10));
    assert!(
        matches!(
            final_snapshot.status,
            TaskStatus::Cancelled | TaskStatus::PartiallyCompleted
        ),
        "cancelled search should finish as cancelled or partially completed, got {:?}",
        final_snapshot.status
    );

    let paths = search_paths(&final_snapshot);
    assert!(
        paths.len() < 10_000,
        "cancel should stop the search before all matches are produced"
    );
}

#[test]
fn recursive_search_skips_permission_denied_directories_and_reports_error_summary() {
    let _guard = acquire_test_guard();
    let root = fixture_path("permission-cases");
    let blocked = root.join("no-access-dir");
    let _acl_guard = PermissionAclGuard::deny_read(blocked.clone());

    let task_id = search::start_search(SearchRequest {
        root_path: root.to_string_lossy().to_string(),
        query: "sample".to_string(),
        recursive: true,
        snapshot_version: None,
    })
    .expect("start_search should succeed");

    let snapshot = wait_for_terminal_snapshot(&task_id, Duration::from_secs(5));
    assert_eq!(snapshot.status, TaskStatus::Completed);

    let error_summaries: Vec<_> = snapshot
        .batches
        .iter()
        .flat_map(|batch| batch.error_summaries.iter())
        .collect();
    assert!(
        !error_summaries.is_empty(),
        "permission denied directory should produce an error summary"
    );
    assert!(
        error_summaries
            .iter()
            .any(|summary| summary.path.ends_with("no-access-dir")),
        "error summary should point to the skipped directory"
    );
    assert!(
        error_summaries
            .iter()
            .any(|summary| summary.error.code == ErrorCode::PermissionDenied),
        "error summary should use permission_denied"
    );

    let paths = search_paths(&snapshot);
    assert!(
        paths.iter().any(|path| path.ends_with("sample-readonly.txt")),
        "accessible files should still be returned"
    );
    assert!(
        paths.iter().any(|path| path.ends_with("sample-normal.txt")),
        "accessible files should still be returned"
    );
    assert!(
        !paths.iter().any(|path| path.ends_with("sample-blocked.txt")),
        "blocked directory contents should be skipped"
    );
}

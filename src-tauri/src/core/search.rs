use serde::{Deserialize, Serialize};
#[cfg(windows)]
use std::os::windows::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use crate::core::error::{AppError, ErrorCode};
use crate::core::fs;
use crate::core::tasks;
use crate::core::types::{FileEntry, TaskStatus};

const SEARCH_BATCH_SIZE: usize = 16;
const SEARCH_YIELD_EVERY: usize = 64;
const SEARCH_PAUSE_EVERY: usize = 64;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
#[serde(transparent)]
pub struct TaskId(pub String);

impl TaskId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SearchRequest {
    pub root_path: String,
    pub query: String,
    pub recursive: bool,
    pub snapshot_version: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchErrorSummary {
    pub path: String,
    pub error: AppError,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchResultBatch {
    pub task_id: TaskId,
    pub root_path: String,
    pub query: String,
    pub recursive: bool,
    pub snapshot_version: Option<u64>,
    pub matches: Vec<FileEntry>,
    pub error_summaries: Vec<SearchErrorSummary>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SearchTaskSnapshot {
    pub task_id: TaskId,
    pub request: SearchRequest,
    pub status: TaskStatus,
    pub batches: Vec<SearchResultBatch>,
}

#[derive(Clone)]
struct BatchAccumulator {
    matches: Vec<FileEntry>,
    error_summaries: Vec<SearchErrorSummary>,
}

impl BatchAccumulator {
    fn new() -> Self {
        Self {
            matches: Vec::new(),
            error_summaries: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.matches.is_empty() && self.error_summaries.is_empty()
    }

    fn push_match(&mut self, entry: FileEntry) -> bool {
        self.matches.push(entry);
        self.should_flush()
    }

    fn push_error(&mut self, error: SearchErrorSummary) -> bool {
        self.error_summaries.push(error);
        self.should_flush()
    }

    fn should_flush(&self) -> bool {
        self.matches.len() + self.error_summaries.len() >= SEARCH_BATCH_SIZE
    }

    fn drain_into_batch(
        &mut self,
        task_id: &TaskId,
        root_path: &str,
        query: &str,
        recursive: bool,
        snapshot_version: Option<u64>,
    ) -> Option<SearchResultBatch> {
        if self.is_empty() {
            return None;
        }

        Some(SearchResultBatch {
            task_id: task_id.clone(),
            root_path: root_path.to_string(),
            query: query.to_string(),
            recursive,
            snapshot_version,
            matches: std::mem::take(&mut self.matches),
            error_summaries: std::mem::take(&mut self.error_summaries),
        })
    }
}

pub fn start_search(request: SearchRequest) -> Result<TaskId, AppError> {
    validate_root(&request.root_path)?;

    let task_id = tasks::create_search_task(request.clone());
    let worker_task_id = task_id.clone();
    std::thread::spawn(move || run_search(worker_task_id, request));
    Ok(task_id)
}

pub fn cancel_task(task_id: &TaskId) -> Result<TaskStatus, AppError> {
    tasks::cancel_search_task(task_id)
}

pub fn get_task_snapshot(task_id: &TaskId) -> Option<SearchTaskSnapshot> {
    tasks::get_search_task_snapshot(task_id)
}

pub fn wait_for_task_status(task_id: &TaskId, timeout: Duration, expected: &[TaskStatus]) -> Option<SearchTaskSnapshot> {
    let started = std::time::Instant::now();
    while started.elapsed() < timeout {
        if let Some(snapshot) = get_task_snapshot(task_id) {
            if expected.iter().any(|status| status == &snapshot.status) {
                return Some(snapshot);
            }
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    get_task_snapshot(task_id)
}

pub fn wait_for_terminal_snapshot(task_id: &TaskId, timeout: Duration) -> Option<SearchTaskSnapshot> {
    wait_for_task_status(
        task_id,
        timeout,
        &[
            TaskStatus::Cancelled,
            TaskStatus::Completed,
            TaskStatus::Failed,
            TaskStatus::PartiallyCompleted,
        ],
    )
}

fn run_search(task_id: TaskId, request: SearchRequest) {
    if tasks::is_search_cancel_requested(&task_id) {
        tasks::set_search_task_status(&task_id, TaskStatus::Cancelled);
        return;
    }
    tasks::set_search_task_status(&task_id, TaskStatus::Running);

    let search_result = if request.recursive {
        run_recursive_search(&task_id, &request)
    } else {
        run_current_directory_search(&task_id, &request)
    };

    match search_result {
        Ok(SearchRunOutcome::Completed {
            had_matches_or_errors,
        }) => {
            let final_status = if tasks::is_search_cancel_requested(&task_id) {
                if had_matches_or_errors {
                    TaskStatus::PartiallyCompleted
                } else {
                    TaskStatus::Cancelled
                }
            } else {
                TaskStatus::Completed
            };
            tasks::set_search_task_status(&task_id, final_status);
        }
        Ok(SearchRunOutcome::Cancelled {
            had_matches_or_errors,
        }) => {
            let final_status = if had_matches_or_errors {
                TaskStatus::PartiallyCompleted
            } else {
                TaskStatus::Cancelled
            };
            tasks::set_search_task_status(&task_id, final_status);
        }
        Err(error) => {
            tasks::set_search_task_failed(&task_id, error);
        }
    }
}

enum SearchRunOutcome {
    Completed {
        had_matches_or_errors: bool,
    },
    Cancelled {
        had_matches_or_errors: bool,
    },
}

fn run_current_directory_search(
    task_id: &TaskId,
    request: &SearchRequest,
) -> Result<SearchRunOutcome, AppError> {
    let page = fs::list_directory(
        &request.root_path,
        &crate::core::types::SortKey::Name,
        true,
        &crate::core::types::FilterKind::All,
        true,
    )?;
    let effective_snapshot_version = request.snapshot_version.or(Some(page.snapshot_version));
    let query = normalized_query(&request.query);
    let mut accumulator = BatchAccumulator::new();
    let mut had_matches_or_errors = false;
    let mut processed = 0usize;

    for entry in page.entries {
        if tasks::is_search_cancel_requested(task_id) {
            flush_accumulator(task_id, request, effective_snapshot_version, &mut accumulator);
            return Ok(SearchRunOutcome::Cancelled {
                had_matches_or_errors,
            });
        }

        if query_matches(&entry, &query) {
            had_matches_or_errors = true;
            if accumulator.push_match(entry) {
                flush_accumulator(task_id, request, effective_snapshot_version, &mut accumulator);
            }
        }

        processed += 1;
        if processed % SEARCH_PAUSE_EVERY == 0 {
            std::thread::yield_now();
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    flush_accumulator(task_id, request, effective_snapshot_version, &mut accumulator);
    Ok(SearchRunOutcome::Completed {
        had_matches_or_errors,
    })
}

fn run_recursive_search(
    task_id: &TaskId,
    request: &SearchRequest,
) -> Result<SearchRunOutcome, AppError> {
    let page = fs::list_directory(
        &request.root_path,
        &crate::core::types::SortKey::Name,
        true,
        &crate::core::types::FilterKind::All,
        true,
    )?;
    let effective_snapshot_version = request.snapshot_version.or(Some(page.snapshot_version));
    let query = normalized_query(&request.query);
    let mut accumulator = BatchAccumulator::new();
    let mut had_matches_or_errors = false;
    let mut processed = 0usize;
    let mut stack: Vec<PathBuf> = Vec::new();

    for entry in page.entries {
        if query_matches(&entry, &query) {
            had_matches_or_errors = true;
            if accumulator.push_match(entry.clone()) {
                flush_accumulator(task_id, request, effective_snapshot_version, &mut accumulator);
            }
        }
        if entry.is_folder {
            stack.push(PathBuf::from(&entry.path));
        }
    }

    while let Some(dir) = stack.pop() {
        if tasks::is_search_cancel_requested(task_id) {
            flush_accumulator(task_id, request, effective_snapshot_version, &mut accumulator);
            return Ok(SearchRunOutcome::Cancelled {
                had_matches_or_errors,
            });
        }

        let (children, errors) = read_directory_entries_sorted(&dir);
        for error in errors {
            had_matches_or_errors = true;
            if accumulator.push_error(error) {
                flush_accumulator(task_id, request, effective_snapshot_version, &mut accumulator);
            }
        }

        for child in children {
            if tasks::is_search_cancel_requested(task_id) {
                flush_accumulator(task_id, request, effective_snapshot_version, &mut accumulator);
                return Ok(SearchRunOutcome::Cancelled {
                    had_matches_or_errors,
                });
            }

            let metadata = match std::fs::symlink_metadata(&child) {
                Ok(metadata) => metadata,
                Err(error) => {
                    had_matches_or_errors = true;
                    let summary = error_summary(
                        child.to_string_lossy().to_string(),
                        error.kind(),
                        format!("读取文件元数据失败: {}", error),
                    );
                    if accumulator.push_error(summary) {
                        flush_accumulator(task_id, request, effective_snapshot_version, &mut accumulator);
                    }
                    continue;
                }
            };

            let file_entry = build_file_entry(&child, metadata);
            if query_matches(&file_entry, &query) {
                had_matches_or_errors = true;
                if accumulator.push_match(file_entry.clone()) {
                    flush_accumulator(task_id, request, effective_snapshot_version, &mut accumulator);
                }
            }

            if file_entry.is_folder {
                stack.push(PathBuf::from(&file_entry.path));
            }

            processed += 1;
            if processed % SEARCH_YIELD_EVERY == 0 {
                std::thread::yield_now();
                std::thread::sleep(Duration::from_millis(5));
            }
        }
    }

    flush_accumulator(task_id, request, effective_snapshot_version, &mut accumulator);
    Ok(SearchRunOutcome::Completed {
        had_matches_or_errors,
    })
}

fn flush_accumulator(
    task_id: &TaskId,
    request: &SearchRequest,
    snapshot_version: Option<u64>,
    accumulator: &mut BatchAccumulator,
) {
    if let Some(batch) = accumulator.drain_into_batch(
        task_id,
        &request.root_path,
        &request.query,
        request.recursive,
        snapshot_version,
    ) {
        tasks::append_search_batch(task_id, batch);
    }
}

fn read_directory_entries_sorted(dir: &Path) -> (Vec<PathBuf>, Vec<SearchErrorSummary>) {
    let mut paths = Vec::new();
    let mut errors = Vec::new();

    let read_dir = match std::fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        Err(error) => {
            errors.push(error_summary(
                dir.to_string_lossy().to_string(),
                error.kind(),
                format!("读取目录失败: {}", error),
            ));
            return (paths, errors);
        }
    };

    for item in read_dir {
        match item {
            Ok(entry) => paths.push(entry.path()),
            Err(error) => {
                errors.push(error_summary(
                    dir.to_string_lossy().to_string(),
                    error.kind(),
                    format!("读取目录项失败: {}", error),
                ));
            }
        }
    }

    paths.sort_by(|a, b| {
        let a_name = a
            .file_name()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let b_name = b
            .file_name()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        a_name.cmp(&b_name)
    });

    (paths, errors)
}

fn build_file_entry(path: &Path, metadata: std::fs::Metadata) -> FileEntry {
    let name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default();
    let is_folder = metadata.is_dir();
    let size = if is_folder { 0 } else { metadata.len() };
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0);
    let is_hidden = (metadata.file_attributes() & 0x2) != 0;

    FileEntry {
        path: path.to_string_lossy().to_string(),
        name,
        size,
        modified,
        is_hidden,
        is_folder,
    }
}

fn normalized_query(query: &str) -> String {
    query.trim().to_lowercase()
}

fn query_matches(entry: &FileEntry, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    entry.name.to_lowercase().contains(query)
}

fn error_summary(path: String, kind: std::io::ErrorKind, message: String) -> SearchErrorSummary {
    let code = match kind {
        std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
        std::io::ErrorKind::NotFound => ErrorCode::PathNotFound,
        _ => ErrorCode::InternalError,
    };
    let error = AppError::new(code, message).with_refresh("搜索过程中已跳过不可访问目录");
    SearchErrorSummary { path, error }
}

fn validate_root(root_path: &str) -> Result<(), AppError> {
    let root = Path::new(root_path);
    let metadata = std::fs::metadata(root).map_err(|error| match error.kind() {
        std::io::ErrorKind::PermissionDenied => AppError::new(
            ErrorCode::PermissionDenied,
            format!("权限不足: {}", root_path),
        ),
        std::io::ErrorKind::NotFound => AppError::new(
            ErrorCode::PathNotFound,
            format!("路径不存在: {}", root_path),
        ),
        _ => AppError::new(
            ErrorCode::InternalError,
            format!("读取搜索根失败: {}", error),
        ),
    })?;

    if !metadata.is_dir() {
        return Err(AppError::new(
            ErrorCode::PathNotFound,
            format!("不是目录: {}", root_path),
        ));
    }

    Ok(())
}

pub(crate) fn create_task_id() -> TaskId {
    static NEXT_SEARCH_ID: AtomicU64 = AtomicU64::new(1);
    let sequence = NEXT_SEARCH_ID.fetch_add(1, Ordering::Relaxed);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0);
    TaskId(format!("search-{}-{}", timestamp, sequence))
}

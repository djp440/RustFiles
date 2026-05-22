use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::core::error::{AppError, ErrorCode};
use crate::core::search::{create_task_id, SearchRequest, SearchResultBatch, SearchTaskSnapshot, TaskId};
use crate::core::types::TaskStatus;

struct SearchTaskHandle {
    request: SearchRequest,
    status: Mutex<TaskStatus>,
    cancel_requested: AtomicBool,
    batches: Mutex<Vec<SearchResultBatch>>,
}

impl SearchTaskHandle {
    fn new(request: SearchRequest) -> Self {
        Self {
            request,
            status: Mutex::new(TaskStatus::Queued),
            cancel_requested: AtomicBool::new(false),
            batches: Mutex::new(Vec::new()),
        }
    }
}

static SEARCH_TASKS: OnceLock<Mutex<HashMap<String, Arc<SearchTaskHandle>>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, Arc<SearchTaskHandle>>> {
    SEARCH_TASKS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn terminal_status(status: &TaskStatus) -> bool {
    matches!(
        status,
        TaskStatus::Cancelled
            | TaskStatus::Completed
            | TaskStatus::Failed
            | TaskStatus::PartiallyCompleted
    )
}

pub fn create_search_task(request: SearchRequest) -> TaskId {
    let task_id = create_task_id();
    let handle = Arc::new(SearchTaskHandle::new(request));
    registry()
        .lock()
        .expect("search registry mutex poisoned")
        .insert(task_id.0.clone(), handle);
    task_id
}

pub fn get_search_task_snapshot(task_id: &TaskId) -> Option<SearchTaskSnapshot> {
    let handle = {
        let registry = registry().lock().ok()?;
        registry.get(task_id.as_str()).cloned()
    }?;

    let status = handle.status.lock().ok()?.clone();
    let batches = handle.batches.lock().ok()?.clone();
    Some(SearchTaskSnapshot {
        task_id: task_id.clone(),
        request: handle.request.clone(),
        status,
        batches,
    })
}

pub fn get_search_task_status(task_id: &TaskId) -> Option<TaskStatus> {
    let handle = get_handle(task_id)?;
    handle.status.lock().ok().map(|status| status.clone())
}

pub fn set_search_task_status(task_id: &TaskId, status: TaskStatus) {
    if let Some(handle) = get_handle(task_id) {
        if let Ok(mut current) = handle.status.lock() {
            *current = status;
        }
    }
}

pub fn set_search_task_failed(task_id: &TaskId, error: AppError) {
    if let Some(handle) = get_handle(task_id) {
        if let Ok(mut current) = handle.status.lock() {
            *current = TaskStatus::Failed;
        }
        let mut batches = handle
            .batches
            .lock()
            .expect("search batch mutex poisoned");
        batches.push(SearchResultBatch {
            task_id: task_id.clone(),
            root_path: handle.request.root_path.clone(),
            query: handle.request.query.clone(),
            recursive: handle.request.recursive,
            snapshot_version: handle.request.snapshot_version,
            matches: Vec::new(),
            error_summaries: vec![crate::core::search::SearchErrorSummary {
                path: handle.request.root_path.clone(),
                error,
            }],
        });
    }
}

pub fn append_search_batch(task_id: &TaskId, batch: SearchResultBatch) {
    if let Some(handle) = get_handle(task_id) {
        if let Ok(mut batches) = handle.batches.lock() {
            batches.push(batch);
        }
    }
}

pub fn cancel_search_task(task_id: &TaskId) -> Result<TaskStatus, AppError> {
    let handle = get_handle(task_id).ok_or_else(|| {
        AppError::new(
            ErrorCode::InternalError,
            format!("找不到搜索任务: {}", task_id.as_str()),
        )
    })?;

    let current = handle
        .status
        .lock()
        .map_err(|_| AppError::new(ErrorCode::InternalError, "搜索任务状态锁已损坏"))?
        .clone();

    if terminal_status(&current) {
        return Ok(current);
    }

    handle.cancel_requested.store(true, Ordering::SeqCst);
    if let Ok(mut status) = handle.status.lock() {
        if !terminal_status(&status) {
            *status = TaskStatus::Cancelling;
        }
        return Ok(status.clone());
    }

    Ok(TaskStatus::Cancelling)
}

pub fn is_search_cancel_requested(task_id: &TaskId) -> bool {
    get_handle(task_id)
        .map(|handle| handle.cancel_requested.load(Ordering::SeqCst))
        .unwrap_or(true)
}

fn get_handle(task_id: &TaskId) -> Option<Arc<SearchTaskHandle>> {
    let registry = registry().lock().ok()?;
    registry.get(task_id.as_str()).cloned()
}

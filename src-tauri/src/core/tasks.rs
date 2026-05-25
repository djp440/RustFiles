use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::core::error::{AppError, ErrorCode};
use crate::core::path_safety;
use crate::core::search::{create_task_id, SearchRequest, SearchResultBatch, SearchTaskSnapshot, TaskId};
use crate::core::system;
use crate::core::types::{FileTask, TaskStatus};

#[derive(Clone, Debug, PartialEq)]
pub struct TaskSummary {
    pub id: String,
    pub kind: String,
    pub status: TaskStatus,
    pub message: Option<String>,
    pub completed_items: Vec<String>,
    pub incomplete_items: Vec<String>,
    pub unknown_items: Vec<String>,
    pub can_cancel: bool,
}

fn is_terminal(status: &TaskStatus) -> bool {
    matches!(
        status,
        TaskStatus::Cancelled
            | TaskStatus::Completed
            | TaskStatus::Failed
            | TaskStatus::PartiallyCompleted
    )
}

fn allowed_next_statuses(status: &TaskStatus) -> &'static [TaskStatus] {
    match status {
        TaskStatus::Queued => &[TaskStatus::Validating, TaskStatus::Cancelling],
        TaskStatus::Validating => &[TaskStatus::Running, TaskStatus::Failed, TaskStatus::Cancelling],
        TaskStatus::Running => &[
            TaskStatus::WaitingForConflictDecision,
            TaskStatus::Cancelling,
            TaskStatus::Completed,
            TaskStatus::Failed,
            TaskStatus::PartiallyCompleted,
        ],
        TaskStatus::WaitingForConflictDecision => {
            &[TaskStatus::Running, TaskStatus::Cancelling, TaskStatus::Failed]
        }
        TaskStatus::Cancelling => &[
            TaskStatus::Cancelled,
            TaskStatus::PartiallyCompleted,
            TaskStatus::Failed,
        ],
        TaskStatus::Cancelled | TaskStatus::Completed | TaskStatus::Failed | TaskStatus::PartiallyCompleted => &[],
    }
}

pub fn apply_task_transition(
    current: &TaskSummary,
    next_status: TaskStatus,
) -> Result<TaskSummary, AppError> {
    if current.status == next_status {
        return Ok(current.clone());
    }

    if is_terminal(&current.status) {
        return Err(AppError::new(
            ErrorCode::InternalError,
            format!("终态任务不能再次迁移: {} -> {:?}", current.id, next_status),
        ));
    }

    if !allowed_next_statuses(&current.status).contains(&next_status) {
        return Err(AppError::new(
            ErrorCode::InternalError,
            format!("非法任务迁移: {:?} -> {:?}", current.status, next_status),
        ));
    }

    let mut next = current.clone();
    next.status = next_status;
    next.can_cancel = matches!(
        next.status,
        TaskStatus::Queued
            | TaskStatus::Validating
            | TaskStatus::Running
            | TaskStatus::WaitingForConflictDecision
            | TaskStatus::Cancelling
    );
    Ok(next)
}

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

// ========================================================================
// 文件操作任务注册表
// ========================================================================

static FILE_OP_TASKS: OnceLock<Mutex<HashMap<String, FileTask>>> = OnceLock::new();

fn file_op_registry() -> &'static Mutex<HashMap<String, FileTask>> {
    FILE_OP_TASKS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn create_file_op_id() -> TaskId {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let seq = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    TaskId(format!("fileop-{}-{}", ts, seq))
}

fn set_file_op_task_status(task_id: &TaskId, status: TaskStatus) {
    if let Ok(mut registry) = file_op_registry().lock() {
        if let Some(task) = registry.get_mut(task_id.as_str()) {
            task.status = status;
        }
    }
}

fn set_file_op_task_failed(task_id: &TaskId, error: &AppError) {
    if let Ok(mut registry) = file_op_registry().lock() {
        if let Some(task) = registry.get_mut(task_id.as_str()) {
            task.status = TaskStatus::Failed;
            task.error_message = Some(error.message.clone());
        }
    }
}

pub fn get_file_op_task_status(task_id: &str) -> Option<FileTask> {
    file_op_registry().lock().ok()?.get(task_id).cloned()
}

pub fn execute_create_folder(
    parent_path: &str,
    name: &str,
    test_root: Option<&str>,
) -> Result<TaskId, AppError> {
    path_safety::validate_child_name(name)?;

    let target_path = Path::new(parent_path).join(name);
    let target_str = target_path.to_string_lossy().to_string();
    path_safety::guard_destructive_path(&target_str, test_root)?;

    let task_id = create_file_op_id();
    let task = FileTask {
        id: task_id.0.clone(),
        source: parent_path.to_string(),
        target: Some(target_str.clone()),
        status: TaskStatus::Queued,
        progress_current: 0,
        progress_total: 0,
        error_message: None,
    };
    file_op_registry()
        .lock()
        .expect("file op registry poisoned")
        .insert(task_id.0.clone(), task);
    set_file_op_task_status(&task_id, TaskStatus::Validating);
    set_file_op_task_status(&task_id, TaskStatus::Running);

    match std::fs::create_dir(&target_path) {
        Ok(_) => {
            set_file_op_task_status(&task_id, TaskStatus::Completed);
            Ok(task_id)
        }
        Err(e) => {
            let app_err = AppError::new(
                ErrorCode::PathNotFound,
                format!("创建文件夹失败: {}", e),
            );
            set_file_op_task_failed(&task_id, &app_err);
            Err(app_err)
        }
    }
}

pub fn execute_rename_item(
    path: &str,
    new_name: &str,
    test_root: Option<&str>,
) -> Result<TaskId, AppError> {
    path_safety::validate_child_name(new_name)?;

    let old_path = Path::new(path);
    let parent = old_path.parent().ok_or_else(|| {
        AppError::new(ErrorCode::PathNotFound, "无法获取父目录路径")
    })?;
    let new_path = parent.join(new_name);
    let new_path_str = new_path.to_string_lossy().to_string();

    path_safety::guard_destructive_path(path, test_root)?;
    path_safety::guard_destructive_path(&new_path_str, test_root)?;

    let task_id = create_file_op_id();
    let task = FileTask {
        id: task_id.0.clone(),
        source: path.to_string(),
        target: Some(new_path_str.clone()),
        status: TaskStatus::Queued,
        progress_current: 0,
        progress_total: 0,
        error_message: None,
    };
    file_op_registry()
        .lock()
        .expect("file op registry poisoned")
        .insert(task_id.0.clone(), task);
    set_file_op_task_status(&task_id, TaskStatus::Validating);
    set_file_op_task_status(&task_id, TaskStatus::Running);

    match std::fs::rename(path, &new_path) {
        Ok(_) => {
            set_file_op_task_status(&task_id, TaskStatus::Completed);
            Ok(task_id)
        }
        Err(e) => {
            let code = if e.kind() == std::io::ErrorKind::NotFound {
                ErrorCode::PathNotFound
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                ErrorCode::PermissionDenied
            } else {
                ErrorCode::InternalError
            };
            let app_err = AppError::new(code, format!("重命名失败: {}", e));
            set_file_op_task_failed(&task_id, &app_err);
            Err(app_err)
        }
    }
}

pub fn execute_delete_to_recycle_bin(
    path: &str,
    test_root: Option<&str>,
) -> Result<TaskId, AppError> {
    path_safety::guard_destructive_path(path, test_root)?;

    let task_id = create_file_op_id();
    let task = FileTask {
        id: task_id.0.clone(),
        source: path.to_string(),
        target: None,
        status: TaskStatus::Queued,
        progress_current: 0,
        progress_total: 0,
        error_message: None,
    };
    file_op_registry()
        .lock()
        .expect("file op registry poisoned")
        .insert(task_id.0.clone(), task);
    set_file_op_task_status(&task_id, TaskStatus::Validating);
    set_file_op_task_status(&task_id, TaskStatus::Running);

    match system::delete_to_recycle_bin(path) {
        Ok(_) => {
            set_file_op_task_status(&task_id, TaskStatus::Completed);
            Ok(task_id)
        }
        Err(app_err) => {
            set_file_op_task_failed(&task_id, &app_err);
            Err(app_err)
        }
    }
}

fn copy_single(source: &Path, target_dir: &Path) -> Result<(), AppError> {
    let file_name = source.file_name().ok_or_else(|| {
        AppError::new(ErrorCode::InternalError, "无法获取源文件名")
    })?;
    let dest = target_dir.join(file_name);

    if dest.exists() {
        // TODO: Task 7.1 will replace this with conflict dialog
        return Err(AppError::new(
            ErrorCode::TargetAlreadyExists,
            format!("目标已存在: {}", dest.display()),
        ));
    }

    if source.is_dir() {
        std::fs::create_dir_all(&dest).map_err(|e| {
            AppError::new(ErrorCode::InternalError, format!("创建目标目录失败: {}", e))
        })?;
        for entry in std::fs::read_dir(source).map_err(|e| {
            AppError::new(ErrorCode::InternalError, format!("读取源目录失败: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                AppError::new(ErrorCode::InternalError, format!("读取目录项失败: {}", e))
            })?;
            copy_single(&entry.path(), &dest)?;
        }
    } else {
        std::fs::copy(source, &dest).map_err(|e| {
            AppError::new(ErrorCode::InternalError, format!("复制文件失败: {}", e))
        })?;
    }

    Ok(())
}

/// 复制多个源路径到目标目录。
/// 逐项复制，收集成功和失败路径。
/// 全部成功 → Completed，部分失败 → PartiallyCompleted，
/// 全部失败 → Failed。
pub fn execute_copy_items(
    sources: &[String],
    target_dir: &str,
    test_root: Option<&str>,
) -> Result<TaskId, AppError> {
    let target_path = Path::new(target_dir);

    // 先校验所有路径
    for src in sources {
        path_safety::guard_destructive_path(src, test_root)?;
    }
    path_safety::guard_destructive_path(target_dir, test_root)?;

    let task_id = create_file_op_id();
    let source_display = sources.first().cloned().unwrap_or_else(|| "multiple".to_string());
    let total = sources.len() as u64;
    let task = FileTask {
        id: task_id.0.clone(),
        source: source_display,
        target: Some(target_dir.to_string()),
        status: TaskStatus::Queued,
        progress_current: 0,
        progress_total: total,
        error_message: None,
    };
    file_op_registry()
        .lock()
        .expect("file op registry poisoned")
        .insert(task_id.0.clone(), task);
    set_file_op_task_status(&task_id, TaskStatus::Validating);
    set_file_op_task_status(&task_id, TaskStatus::Running);

    let mut completed: Vec<String> = Vec::new();
    let mut incomplete: Vec<String> = Vec::new();
    let mut first_error_code: Option<ErrorCode> = None;

    for src in sources {
        let src_path = Path::new(src);
        if !src_path.exists() {
            incomplete.push(src.clone());
            first_error_code.get_or_insert(ErrorCode::PathNotFound);
            continue;
        }
        match copy_single(src_path, target_path) {
            Ok(_) => {
                completed.push(src.clone());
            }
            Err(e) => {
                first_error_code.get_or_insert(e.code);
                incomplete.push(src.clone());
            }
        }
    }

    if completed.is_empty() && !incomplete.is_empty() {
        let err_code = first_error_code.unwrap_or(ErrorCode::PathNotFound);
        let err = AppError::new(err_code, "所有源路径复制失败");
        set_file_op_task_failed(&task_id, &err);
        return Err(err);
    }

    if incomplete.is_empty() {
        set_file_op_task_status(&task_id, TaskStatus::Completed);
    } else {
        set_file_op_task_status(&task_id, TaskStatus::PartiallyCompleted);
    }

    // 更新进度
    let done = completed.len() as u64;
    if let Ok(mut registry) = file_op_registry().lock() {
        if let Some(task) = registry.get_mut(task_id.as_str()) {
            task.progress_current = done;
        }
    }

    Ok(task_id)
}

/// 移动多个源路径到目标目录。
/// 委托 execute_copy_items 完成复制，复制全部成功后逐项删除源文件。
/// 部分复制成功时不删除源文件（保证数据安全），部分源文件删除失败时标记为 PartiallyCompleted。
pub fn execute_move_items(
    sources: &[String],
    target_dir: &str,
    test_root: Option<&str>,
) -> Result<TaskId, AppError> {
    let copy_result = execute_copy_items(sources, target_dir, test_root)?;

    let status = file_op_registry()
        .lock()
        .map_err(|_| AppError::new(ErrorCode::InternalError, "文件操作注册表锁已损坏"))?
        .get(copy_result.as_str())
        .map(|t| t.status.clone())
        .unwrap_or(TaskStatus::Failed);

    if status == TaskStatus::Completed {
        let mut delete_failures: Vec<String> = Vec::new();
        for src in sources {
            let path = Path::new(src);
            if path.is_dir() {
                if let Err(_) = std::fs::remove_dir_all(path) {
                    delete_failures.push(src.clone());
                }
            } else {
                if let Err(_) = std::fs::remove_file(path) {
                    delete_failures.push(src.clone());
                }
            }
        }
        if !delete_failures.is_empty() {
            set_file_op_task_status(&copy_result, TaskStatus::PartiallyCompleted);
            if let Ok(mut reg) = file_op_registry().lock() {
                if let Some(task) = reg.get_mut(copy_result.as_str()) {
                    task.error_message = Some(format!(
                        "复制成功但部分源文件删除失败（{} 项）",
                        delete_failures.len()
                    ));
                }
            }
        }
    }

    Ok(copy_result)
}

pub fn execute_delete_permanently(
    path: &str,
    test_root: Option<&str>,
) -> Result<TaskId, AppError> {
    path_safety::guard_destructive_path(path, test_root)?;

    let task_id = create_file_op_id();
    let task = FileTask {
        id: task_id.0.clone(),
        source: path.to_string(),
        target: None,
        status: TaskStatus::Queued,
        progress_current: 0,
        progress_total: 0,
        error_message: None,
    };
    file_op_registry()
        .lock()
        .expect("file op registry poisoned")
        .insert(task_id.0.clone(), task);
    set_file_op_task_status(&task_id, TaskStatus::Validating);
    set_file_op_task_status(&task_id, TaskStatus::Running);

    let p = Path::new(path);
    let result = if p.is_dir() {
        std::fs::remove_dir_all(p)
    } else {
        std::fs::remove_file(p)
    };

    match result {
        Ok(_) => {
            set_file_op_task_status(&task_id, TaskStatus::Completed);
            Ok(task_id)
        }
        Err(e) => {
            let code = if e.kind() == std::io::ErrorKind::NotFound {
                ErrorCode::PathNotFound
            } else if e.kind() == std::io::ErrorKind::PermissionDenied {
                ErrorCode::PermissionDenied
            } else {
                ErrorCode::InternalError
            };
            let app_err = AppError::new(code, format!("永久删除失败: {}", e));
            set_file_op_task_failed(&task_id, &app_err);
            Err(app_err)
        }
    }
}

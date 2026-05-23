use crate::core::error::AppError;
use crate::core::runtime::RuntimeGuard;
use crate::core::scheduler::{
    summarize_interaction_state, summarize_viewport_state, SchedulerReportAck, SchedulerSignal,
};
use crate::core::search::{SearchRequest, TaskId};
use crate::core::tasks;
use crate::core::types::{
    DirectoryPage, DriveList, FilterKind, Settings, SidebarRoots, SortKey, TaskStatus,
};

// ============================================================
// 浏览类 commands — 非危险
// ============================================================

#[tauri::command]
pub async fn list_directory(
    path: String,
    sort_key: Option<SortKey>,
    sort_ascending: Option<bool>,
    filter_kind: Option<FilterKind>,
    show_hidden: Option<bool>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<DirectoryPage, AppError> {
    let sort_key = sort_key.unwrap_or(SortKey::Name);
    let sort_ascending = sort_ascending.unwrap_or(true);
    let filter_kind = filter_kind.unwrap_or(FilterKind::All);
    let show_hidden = show_hidden.unwrap_or(false);
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(usize::MAX);
    crate::core::fs::list_directory_paginated(&path, &sort_key, sort_ascending, &filter_kind, show_hidden, offset, limit)
}

#[tauri::command]
pub async fn get_sidebar_roots() -> Result<SidebarRoots, AppError> {
    crate::core::system::get_sidebar_roots()
}

#[tauri::command]
pub async fn get_drives() -> Result<DriveList, AppError> {
    crate::core::system::get_drives()
}

// ============================================================
// 搜索类 commands — 非危险
// ============================================================

#[tauri::command]
pub async fn start_search(request: SearchRequest) -> Result<TaskId, AppError> {
    crate::core::search::start_search(request)
}

#[tauri::command]
pub async fn cancel_task(task_id: TaskId) -> Result<TaskStatus, AppError> {
    crate::core::search::cancel_task(&task_id)
}

// ============================================================
// 文件操作类 commands
// ============================================================

#[tauri::command]
pub async fn create_folder(parent_path: String, name: String) -> Result<TaskId, AppError> {
    let test_root = RuntimeGuard::test_root();
    tasks::execute_create_folder(&parent_path, &name, test_root.as_deref())
}

#[tauri::command]
pub async fn rename_item(path: String, new_name: String) -> Result<TaskId, AppError> {
    let test_root = RuntimeGuard::test_root();
    tasks::execute_rename_item(&path, &new_name, test_root.as_deref())
}

#[tauri::command]
pub async fn delete_to_recycle_bin(path: String) -> Result<TaskId, AppError> {
    let test_root = RuntimeGuard::test_root();
    tasks::execute_delete_to_recycle_bin(&path, test_root.as_deref())
}

/// 危险 command：永久删除，先经过测试模式 guard，再检查确认令牌
#[tauri::command]
pub async fn delete_permanently(path: String, confirmation_token: Option<String>) -> Result<TaskId, AppError> {
    RuntimeGuard::guard_dangerous_operation()?;
    RuntimeGuard::check_confirmation(confirmation_token)?;
    let test_root = RuntimeGuard::test_root();
    tasks::execute_delete_permanently(&path, test_root.as_deref())
}

#[tauri::command]
pub async fn copy_items() -> Result<TaskId, AppError> {
    Err(AppError::not_implemented())
}

/// 危险 command：移动文件，先经过测试模式 guard，再检查确认令牌
#[tauri::command]
pub async fn move_items(confirmation_token: Option<String>) -> Result<TaskId, AppError> {
    RuntimeGuard::guard_dangerous_operation()?;
    RuntimeGuard::check_confirmation(confirmation_token)?;
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn create_clipboard_operation() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn paste_clipboard_operation() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn create_drag_operation() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

/// 危险 command：拖拽释放（默认移动），先经过测试模式 guard，再检查确认令牌
#[tauri::command]
pub async fn drop_drag_operation(confirmation_token: Option<String>) -> Result<(), AppError> {
    RuntimeGuard::guard_dangerous_operation()?;
    RuntimeGuard::check_confirmation(confirmation_token)?;
    Err(AppError::not_implemented())
}

/// 危险 command：冲突解决（覆盖/替换），先经过测试模式 guard，再检查确认令牌
#[tauri::command]
pub async fn resolve_conflict(confirmation_token: Option<String>) -> Result<(), AppError> {
    RuntimeGuard::guard_dangerous_operation()?;
    RuntimeGuard::check_confirmation(confirmation_token)?;
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn get_task_status(task_id: TaskId) -> Result<TaskStatus, AppError> {
    // 先查搜索任务
    if let Some(status) = tasks::get_search_task_status(&task_id) {
        return Ok(status);
    }
    // 再查文件操作任务
    if let Some(task) = tasks::get_file_op_task_status(task_id.as_str()) {
        return Ok(task.status);
    }
    Err(AppError::new(
        crate::core::error::ErrorCode::InternalError,
        format!("找不到任务: {}", task_id.as_str()),
    ))
}

// ============================================================
// 缩略图类 commands — 非危险
// ============================================================

#[tauri::command]
pub async fn request_thumbnails() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn cancel_thumbnail_requests() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

// ============================================================
// 调度类 commands — 非危险
// ============================================================

#[tauri::command]
pub async fn report_viewport_state(signal: SchedulerSignal) -> Result<SchedulerReportAck, AppError> {
    Ok(summarize_viewport_state(signal))
}

#[tauri::command]
pub async fn report_interaction_state(signal: SchedulerSignal) -> Result<SchedulerReportAck, AppError> {
    Ok(summarize_interaction_state(signal))
}

// ============================================================
// 设置类 commands — 非危险
// ============================================================

#[tauri::command]
pub async fn get_settings() -> Result<Settings, AppError> {
    crate::core::settings::read_settings()
}

#[tauri::command]
pub async fn update_settings(settings: Settings) -> Result<Settings, AppError> {
    crate::core::settings::write_settings(&settings)?;
    crate::core::settings::read_settings()
}

// ============================================================
// 系统集成类 commands — 非危险
// ============================================================

#[tauri::command]
pub async fn open_with_default_app(path: String) -> Result<(), AppError> {
    crate::core::system::open_with_default_app(&path)
}

#[tauri::command]
pub async fn open_terminal(path: String) -> Result<(), AppError> {
    crate::core::system::open_terminal(&path)
}

#[tauri::command]
pub async fn show_properties(path: String) -> Result<(), AppError> {
    crate::core::system::show_properties(&path)
}

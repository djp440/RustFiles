use crate::core::clipboard;
use crate::core::drag;
use crate::core::error::{AppError, ErrorCode};
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
pub async fn copy_items(source_paths: Vec<String>, target_dir: String) -> Result<TaskId, AppError> {
    let op_id = clipboard::create_operation(source_paths, "copy", "clipboard");
    paste_clipboard_operation(op_id, target_dir, None).await
}

#[tauri::command]
pub async fn move_items(
    source_paths: Vec<String>,
    target_dir: String,
    confirmation_token: Option<String>,
) -> Result<TaskId, AppError> {
    RuntimeGuard::guard_dangerous_operation()?;
    RuntimeGuard::check_confirmation(confirmation_token)?;
    let op_id = clipboard::create_operation(source_paths, "cut", "clipboard");
    paste_clipboard_operation(op_id, target_dir, Some("pre-confirmed".to_string())).await
}

#[tauri::command]
pub async fn create_clipboard_operation(
    source_paths: Vec<String>,
    op_type: String,
    source_tab_id: String,
) -> Result<String, AppError> {
    RuntimeGuard::guard_dangerous_operation()?;
    let test_root = RuntimeGuard::test_root();
    for path in &source_paths {
        crate::core::path_safety::guard_destructive_path(path, test_root.as_deref())?;
    }
    let op_id = clipboard::create_operation(source_paths, &op_type, &source_tab_id);
    Ok(op_id)
}

#[tauri::command]
pub async fn paste_clipboard_operation(
    operation_id: String,
    target_dir: String,
    confirmation_token: Option<String>,
) -> Result<TaskId, AppError> {
    RuntimeGuard::guard_dangerous_operation()?;

    let op = clipboard::get_operation(&operation_id).ok_or_else(|| {
        AppError::new(ErrorCode::PathNotFound, "剪贴板操作不存在或已过期")
    })?;

    if op.status != clipboard::ClipOpStatus::Active {
        return Err(AppError::new(
            ErrorCode::InternalError,
            "此剪贴板操作已被使用或已失效，不能再次粘贴",
        ));
    }

    let test_root = RuntimeGuard::test_root();

    // 对 cut 操作需要确认令牌
    if op.op_type == clipboard::ClipOpType::Cut {
        RuntimeGuard::check_confirmation(confirmation_token)?;
    }

    // 校验每个源路径是否存在
    for src in &op.source_paths {
        if !std::path::Path::new(src).exists() {
            return Err(AppError::new(
                ErrorCode::PathNotFound,
                format!("源路径不存在: {}", src),
            ));
        }
    }

    // 对目标路径进行安全校验
    crate::core::path_safety::guard_destructive_path(&target_dir, test_root.as_deref())?;

    let task_id = match op.op_type {
        clipboard::ClipOpType::Copy => {
            tasks::execute_copy_items(&op.source_paths, &target_dir, test_root.as_deref())?
        }
        clipboard::ClipOpType::Cut => {
            tasks::execute_move_items(&op.source_paths, &target_dir, test_root.as_deref())?
        }
    };

    clipboard::mark_pasted(&operation_id);
    Ok(task_id)
}

#[tauri::command]
pub async fn create_drag_operation(
    source_paths: Vec<String>,
    drag_type: String,
    source_tab_id: String,
) -> Result<String, AppError> {
    let test_root = RuntimeGuard::test_root();
    for path in &source_paths {
        crate::core::path_safety::guard_destructive_path(path, test_root.as_deref())?;
    }
    let op_id = drag::create_operation(source_paths, &drag_type, &source_tab_id);
    Ok(op_id)
}

/// 危险 command：拖拽释放（默认移动），先经过测试模式 guard，再检查确认令牌
#[tauri::command]
pub async fn drop_drag_operation(
    operation_id: String,
    target_dir: String,
    requested_type: Option<String>,
    confirmation_token: Option<String>,
) -> Result<String, AppError> {
    RuntimeGuard::guard_dangerous_operation()?;
    RuntimeGuard::check_confirmation(confirmation_token)?;

    let op = drag::get_operation(&operation_id).ok_or_else(|| {
        AppError::new(ErrorCode::PathNotFound, "拖拽操作不存在或已过期")
    })?;

    if op.status != drag::DragOpStatus::Active {
        return Err(AppError::new(
            ErrorCode::InternalError,
            "此拖拽操作已被使用，不能再次释放",
        ));
    }

    let test_root = RuntimeGuard::test_root();

    // 校验每个源路径是否存在
    for src in &op.source_paths {
        if !std::path::Path::new(src).exists() {
            return Err(AppError::new(
                ErrorCode::PathNotFound,
                format!("源路径不存在: {}", src),
            ));
        }
    }

    // 对目标路径进行安全校验
    crate::core::path_safety::guard_destructive_path(&target_dir, test_root.as_deref())?;

    // 跨卷检测
    let cross_volume = op.source_paths.first().map(|src| drag::is_cross_volume(src, &target_dir)).unwrap_or(false);

    // 确定最终执行类型
    let effective_type = match &requested_type {
        Some(t) if t == "copy" => drag::DragOpType::Copy,
        Some(t) if t == "move" => drag::DragOpType::Move,
        _ if cross_volume => drag::DragOpType::Copy,
        _ => op.drag_type.clone(),
    };

    let task_id = match effective_type {
        drag::DragOpType::Copy => {
            tasks::execute_copy_items(&op.source_paths, &target_dir, test_root.as_deref())?
        }
        drag::DragOpType::Move => {
            tasks::execute_move_items(&op.source_paths, &target_dir, test_root.as_deref())?
        }
    };

    drag::mark_dropped(&operation_id);
    Ok(task_id.as_str().to_string())
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

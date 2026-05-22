use crate::core::error::AppError;
use crate::core::runtime::RuntimeGuard;

// ============================================================
// 浏览类 commands — 非危险，直接返回 not_implemented
// ============================================================

#[tauri::command]
pub async fn list_directory() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn get_sidebar_roots() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn get_drives() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

// ============================================================
// 搜索类 commands — 非危险
// ============================================================

#[tauri::command]
pub async fn start_search() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn cancel_task() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

// ============================================================
// 文件操作类 commands
// ============================================================

#[tauri::command]
pub async fn create_folder() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn rename_item() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn delete_to_recycle_bin() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

/// 危险 command：永久删除，先经过测试模式 guard，再检查确认令牌
#[tauri::command]
pub async fn delete_permanently(confirmation_token: Option<String>) -> Result<(), AppError> {
    RuntimeGuard::guard_dangerous_operation()?;
    RuntimeGuard::check_confirmation(confirmation_token)?;
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn copy_items() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

/// 危险 command：移动文件，先经过测试模式 guard，再检查确认令牌
#[tauri::command]
pub async fn move_items(confirmation_token: Option<String>) -> Result<(), AppError> {
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
pub async fn get_task_status() -> Result<(), AppError> {
    Err(AppError::not_implemented())
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
pub async fn report_viewport_state() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn report_interaction_state() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

// ============================================================
// 设置类 commands — 非危险
// ============================================================

#[tauri::command]
pub async fn get_settings() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn update_settings() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

// ============================================================
// 系统集成类 commands — 非危险
// ============================================================

#[tauri::command]
pub async fn open_with_default_app() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn open_terminal() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

#[tauri::command]
pub async fn show_properties() -> Result<(), AppError> {
    Err(AppError::not_implemented())
}

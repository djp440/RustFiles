use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

/// 剪贴板操作类型
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClipOpType {
    Copy,
    Cut,
}

/// 剪贴板操作状态
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ClipOpStatus {
    Active,
    Pasted,
    Invalidated,
}

/// 剪贴板操作元数据
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClipOperation {
    pub operation_id: String,
    pub source_paths: Vec<String>,
    pub op_type: ClipOpType,
    pub source_tab_id: String,
    pub created_at: i64,
    pub status: ClipOpStatus,
}

static CLIPBOARD: OnceLock<Mutex<HashMap<String, ClipOperation>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, ClipOperation>> {
    CLIPBOARD.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> String {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let seq = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("clip-{}-{}", ts, seq)
}

/// 创建一个新的剪贴板操作。
/// `sources` 是源路径列表，`op_type` 为 "copy" 或 "cut"，`tab_id` 是标签页 ID。
/// 返回操作 ID。
pub fn create_operation(sources: Vec<String>, op_type: &str, tab_id: &str) -> String {
    let op_id = next_id();
    let op_type_enum = match op_type {
        "cut" => ClipOpType::Cut,
        _ => ClipOpType::Copy,
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let operation = ClipOperation {
        operation_id: op_id.clone(),
        source_paths: sources,
        op_type: op_type_enum,
        source_tab_id: tab_id.to_string(),
        created_at: now,
        status: ClipOpStatus::Active,
    };

    registry()
        .lock()
        .expect("clipboard registry mutex poisoned")
        .insert(op_id.clone(), operation);
    op_id
}

/// 根据操作 ID 获取剪贴板操作。
pub fn get_operation(op_id: &str) -> Option<ClipOperation> {
    registry()
        .lock()
        .ok()
        .and_then(|reg| reg.get(op_id).cloned())
}

/// 删除剪贴板操作。
pub fn delete_operation(op_id: &str) {
    if let Ok(mut reg) = registry().lock() {
        reg.remove(op_id);
    }
}

/// 将操作标记为 Invalidated。
pub fn invalidate_operation(op_id: &str) {
    if let Ok(mut reg) = registry().lock() {
        if let Some(op) = reg.get_mut(op_id) {
            op.status = ClipOpStatus::Invalidated;
        }
    }
}

/// 将操作标记为 Pasted（已粘贴）。
pub fn mark_pasted(op_id: &str) {
    if let Ok(mut reg) = registry().lock() {
        if let Some(op) = reg.get_mut(op_id) {
            op.status = ClipOpStatus::Pasted;
        }
    }
}

/// 列出所有剪贴板操作。
pub fn list_operations() -> Vec<ClipOperation> {
    registry()
        .lock()
        .map(|reg| reg.values().cloned().collect())
        .unwrap_or_default()
}

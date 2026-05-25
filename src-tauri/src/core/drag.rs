use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DragOpType {
    Move,
    Copy,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum DragOpStatus {
    Active,
    Dropped,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DragOperation {
    pub operation_id: String,
    pub source_paths: Vec<String>,
    pub drag_type: DragOpType,
    pub source_tab_id: String,
    pub created_at: i64,
    pub status: DragOpStatus,
}

static DRAG: OnceLock<Mutex<HashMap<String, DragOperation>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<String, DragOperation>> {
    DRAG.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> String {
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    let seq = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("drag-{}-{}", ts, seq)
}

pub fn create_operation(sources: Vec<String>, drag_type: &str, tab_id: &str) -> String {
    let op_id = next_id();
    let drag_type_enum = match drag_type {
        "copy" => DragOpType::Copy,
        _ => DragOpType::Move,
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    let operation = DragOperation {
        operation_id: op_id.clone(),
        source_paths: sources,
        drag_type: drag_type_enum,
        source_tab_id: tab_id.to_string(),
        created_at: now,
        status: DragOpStatus::Active,
    };

    registry()
        .lock()
        .expect("drag registry mutex poisoned")
        .insert(op_id.clone(), operation);
    op_id
}

pub fn get_operation(op_id: &str) -> Option<DragOperation> {
    registry()
        .lock()
        .ok()
        .and_then(|reg| reg.get(op_id).cloned())
}

pub fn delete_operation(op_id: &str) {
    if let Ok(mut reg) = registry().lock() {
        reg.remove(op_id);
    }
}

pub fn mark_dropped(op_id: &str) {
    if let Ok(mut reg) = registry().lock() {
        if let Some(op) = reg.get_mut(op_id) {
            op.status = DragOpStatus::Dropped;
        }
    }
}

pub fn list_operations() -> Vec<DragOperation> {
    registry()
        .lock()
        .map(|reg| reg.values().cloned().collect())
        .unwrap_or_default()
}

pub fn is_cross_volume(src_path: &str, target_path: &str) -> bool {
    let src_root = Path::new(src_path).components().next();
    let target_root = Path::new(target_path).components().next();
    src_root != target_root
}

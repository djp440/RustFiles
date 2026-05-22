use std::time::SystemTime;

pub fn compute_snapshot_version(path: &str, total_count: usize) -> u64 {
    let dir_mtime_ms = std::fs::metadata(path)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_millis())
        .unwrap_or(0);
    (dir_mtime_ms as u64) * 31 + (total_count as u64)
}

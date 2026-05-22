use std::os::windows::fs::MetadataExt;
use std::path::Path;
use std::time::SystemTime;

use crate::core::error::{AppError, ErrorCode};
use crate::core::types::{DirectoryPage, FileEntry, FilterKind, SortKey};

pub fn list_directory(
    path: &str,
    sort_key: &SortKey,
    sort_ascending: bool,
    filter_kind: &FilterKind,
    show_hidden: bool,
) -> Result<DirectoryPage, AppError> {
    let p = Path::new(path);
    if !p.exists() {
        return Err(AppError::new(
            ErrorCode::PathNotFound,
            format!("路径不存在: {}", path),
        ));
    }
    if !p.is_dir() {
        return Err(AppError::new(
            ErrorCode::PathNotFound,
            format!("不是目录: {}", path),
        ));
    }

    let read_dir = match std::fs::read_dir(p) {
        Ok(rd) => rd,
        Err(e) => {
            return Err(match e.kind() {
                std::io::ErrorKind::PermissionDenied => AppError::new(
                    ErrorCode::PermissionDenied,
                    format!("权限不足: {}", path),
                ),
                std::io::ErrorKind::NotFound => AppError::new(
                    ErrorCode::PathNotFound,
                    format!("路径不存在: {}", path),
                ),
                _ => AppError::new(
                    ErrorCode::InternalError,
                    format!("读取目录失败: {}", e),
                ),
            });
        }
    };

    let mut entries: Vec<FileEntry> = Vec::new();
    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let is_hidden = (metadata.file_attributes() & 0x2) != 0;
        if is_hidden && !show_hidden {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();
        let is_folder = metadata.is_dir();
        let size = if is_folder { 0 } else { metadata.len() };
        let modified = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        match filter_kind {
            FilterKind::Folders => {
                if !is_folder {
                    continue;
                }
            }
            FilterKind::Files => {
                if is_folder {
                    continue;
                }
            }
            FilterKind::Images => {
                if is_folder {
                    continue;
                }
                let lower = name.to_lowercase();
                if !lower.ends_with(".png")
                    && !lower.ends_with(".jpg")
                    && !lower.ends_with(".jpeg")
                    && !lower.ends_with(".gif")
                    && !lower.ends_with(".bmp")
                    && !lower.ends_with(".webp")
                {
                    continue;
                }
            }
            FilterKind::Documents => {
                if is_folder {
                    continue;
                }
                let lower = name.to_lowercase();
                if !lower.ends_with(".pdf")
                    && !lower.ends_with(".docx")
                    && !lower.ends_with(".xlsx")
                    && !lower.ends_with(".txt")
                    && !lower.ends_with(".md")
                    && !lower.ends_with(".rs")
                    && !lower.ends_with(".toml")
                    && !lower.ends_with(".json")
                {
                    continue;
                }
            }
            FilterKind::All => {}
        }

        entries.push(FileEntry {
            path: entry.path().to_string_lossy().to_string(),
            name,
            size,
            modified,
            is_hidden,
            is_folder,
        });
    }

    entries.sort_by(|a, b| {
        if a.is_folder != b.is_folder {
            return if a.is_folder {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            };
        }
        let cmp = match sort_key {
            SortKey::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            SortKey::Modified => a.modified.cmp(&b.modified),
            SortKey::Size => a.size.cmp(&b.size),
            SortKey::FileType => {
                let ext_a = a.name.rsplit('.').next().unwrap_or("");
                let ext_b = b.name.rsplit('.').next().unwrap_or("");
                ext_a.cmp(ext_b)
            }
        };
        if sort_ascending {
            cmp
        } else {
            cmp.reverse()
        }
    });

    let total_count = entries.len();

    let dir_mtime_ms = std::fs::metadata(p)
        .ok()
        .and_then(|m| m.modified().ok())
        .and_then(|t| t.duration_since(SystemTime::UNIX_EPOCH).ok())
        .map(|d| d.as_millis())
        .unwrap_or(0);

    let snapshot_version = (dir_mtime_ms as u64) * 31 + (total_count as u64);

    Ok(DirectoryPage {
        path: p.to_string_lossy().to_string(),
        entries,
        total_count,
        sort_key: sort_key.clone(),
        sort_ascending,
        filter_kind: filter_kind.clone(),
        show_hidden,
        snapshot_version,
    })
}

use std::io;
use std::os::windows::fs::MetadataExt as _;
use std::path::{Component, Path, PathBuf, Prefix};

use crate::core::error::{AppError, ErrorCode};

/// Windows 保留文件名（不区分大小写）
/// 这些名称不能用作文件或目录名，即使带扩展名也不行（如 CON.txt）
const RESERVED_NAMES: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Windows 文件名非法字符
const ILLEGAL_CHARS: &[char] = &['<', '>', ':', '"', '|', '?', '*'];

/// 最大单个路径组件长度（Windows 限制）
const MAX_COMPONENT_LENGTH: usize = 255;

/// 传统 MAX_PATH（不含 \\?\ 前缀）
#[allow(dead_code)]
const LEGACY_MAX_PATH: usize = 260;

// ============================================================================
// PathClass
// ============================================================================

/// 路径分类结果
#[derive(Debug, Clone, PartialEq)]
pub enum PathClass {
    /// 普通本地文件或目录
    LocalFile,
    /// 重解析点（符号链接 / junction / mount point 等）
    ReparsePoint {
        is_symlink: bool,
        target: Option<String>,
    },
    /// UNC 路径（\\server\share\...）
    UncPath,
    /// subst 或网络盘映射路径
    SubstDrive,
    /// 路径不存在
    NotFound,
    /// 无法确定分类（如 IO 错误）
    Unknown,
}

// ============================================================================
// normalize_path
// ============================================================================

/// 规范化 Windows 路径：
/// - 将所有 `/` 替换为 `\`
/// - 解析 `.` 和 `..` 组件
/// - 去除尾部斜杠（保留驱动器根目录的 `C:\`）
/// - 对已有路径优先使用 `canonicalize` 解析真实路径
pub fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }

    let path = path.replace('/', "\\");

    // 对存在的路径尝试 canonicalize 获取最干净的真实形式
    // Windows 上 canonicalize 会返回 \\?\C:\... 形式，需要去掉扩展前缀
    let simple = Path::new(&path);
    if simple.exists() {
        if let Ok(canonical) = simple.canonicalize() {
            let s = canonical.to_string_lossy().to_string();
            // 去掉 canonicalize 添加的 \\?\ 前缀（除非输入自身就有此前缀）
            if s.starts_with("\\\\?\\") && !path.starts_with("\\\\?\\") {
                return s[4..].to_string();
            }
            return s;
        }
    }

    // 对不存在的路径 / UNC / 长路径前缀 → 纯字符串规范化
    let path_buf = PathBuf::from(&path);

    // 判断是否以 \\?\ 或 \\ 开头
    let has_extended_prefix = path.starts_with("\\\\?\\");
    let has_unc_prefix = path.starts_with("\\\\") && !has_extended_prefix;

    let components: Vec<Component> = path_buf.components().collect();
    let mut normalized = Vec::new();

    for (i, comp) in components.iter().enumerate() {
        match comp {
            Component::ParentDir => {
                // 弹出上一级（若存在）
                if let Some(last) = normalized.last() {
                    match last {
                        Component::Normal(_) | Component::ParentDir => {
                            normalized.pop();
                        }
                        Component::RootDir | Component::Prefix(_) => {
                            // 在根/前缀位置遇到 .. 时保留该 ..（如 UNC/前缀无法回退）
                            if has_unc_prefix || has_extended_prefix {
                                normalized.push(*comp);
                            }
                        }
                        Component::CurDir => {
                            normalized.pop();
                            // 再尝试弹一级
                            if let Some(prev) = normalized.last() {
                                match prev {
                                    Component::Normal(_) | Component::ParentDir => {
                                        normalized.pop();
                                    }
                                    Component::RootDir | Component::Prefix(_) => {
                                        if has_unc_prefix || has_extended_prefix {
                                            normalized.push(Component::ParentDir);
                                        }
                                    }
                                    Component::CurDir => {}
                                }
                            }
                        }
                    }
                }
            }
            Component::CurDir => {
                // 跳过 .
            }
            Component::Prefix(_) | Component::RootDir => {
                normalized.push(*comp);
            }
            Component::Normal(_) => {
                // 最后一个组件如果是空字符串则跳过
                if i == components.len() - 1 && comp.as_os_str().is_empty() {
                    continue;
                }
                normalized.push(*comp);
            }
        }
    }

    let result_path: PathBuf = normalized.iter().collect();
    let result = result_path.to_string_lossy().to_string();

    // 去除尾部 \（保留 C:\ 这类仅有前缀+根的形式，以及 UNC 根 \\server\share）
    let trimmed = result.trim_end_matches('\\').to_string();
    // 如果是纯驱动器根（如 "C:" → "C:\"），则加回反斜杠
    if trimmed.len() >= 2
        && trimmed.as_bytes()[trimmed.len() - 2] == b':'
        && trimmed.as_bytes()[trimmed.len() - 1].is_ascii_alphabetic()
    {
        return format!("{}\\", trimmed);
    }

    // 如果 trimmed 后只剩 "\\server\share" 且以 \\ 开头，不应去掉
    // 但如果 trimmed 是 UNC 共享根，需要点特殊处理
    // 确保 UNC 根被正确保留
    if result.starts_with("\\\\") {
        // UNC 路径规范化为 \\server\share 或 \\server\share\path
        // 只在尾部没有路径时保留双斜杠
        // 统计斜杠数来确定是否是根
        let parts: Vec<&str> = trimmed.split('\\').filter(|s| !s.is_empty()).collect();
        // \\server\share → parts = ["server", "share"] (2 parts after filtering)
        if parts.len() <= 2 {
            // 共享根，保留双斜杠格式
            if parts.is_empty() {
                return trimmed;
            }
            let server = parts[0];
            let reconstructed = if parts.len() == 1 {
                format!("\\\\{}", server)
            } else {
                format!("\\\\{}\\{}", parts[0], parts[1])
            };
            return reconstructed;
        }
    }

    // 对于以 \\?\ 开头的路径，保留前缀
    if trimmed.starts_with("\\\\?\\") {
        return trimmed;
    }

    // 如果处理后的路径为空（例如输入只有 / 或 \），返回空
    if trimmed.is_empty() {
        return String::new();
    }

    trimmed
}

// ============================================================================
// classify_path
// ============================================================================

/// 分类路径：检测是否为 reparse point、UNC 路径、subst 盘等
pub fn classify_path(path: &str) -> PathClass {
    if path.is_empty() {
        return PathClass::Unknown;
    }

    let normalized = normalize_path(path);

    // 检查 UNC 路径（\\server\share 或 \\?\UNC\server\share）
    if is_unc_path(&normalized) {
        return PathClass::UncPath;
    }

    // 检查是否为 subst 或网络盘映射驱动器
    if let Some(class) = classify_as_subst_or_network_drive(&normalized, path) {
        return class;
    }

    // 尝试获取文件元数据来判断 reparse point
    let p = Path::new(&normalized);
    // 也尝试原始路径（在 canonicalize 不可用的情况下）
    let metadata = p.symlink_metadata().or_else(|_| Path::new(path).symlink_metadata());

    match metadata {
        Ok(meta) => {
            // 检查是否是 reparse point（FILE_ATTRIBUTE_REPARSE_POINT = 0x400）
            let attrs = meta.file_attributes();
            const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;

            if attrs & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                let is_symlink = meta.file_type().is_symlink();
                let target = if is_symlink {
                    // 尝试读取 symlink 目标
                    std::fs::read_link(&normalized)
                        .or_else(|_| std::fs::read_link(path))
                        .ok()
                        .map(|t| t.to_string_lossy().to_string())
                } else {
                    None
                };
                return PathClass::ReparsePoint { is_symlink, target };
            }

            PathClass::LocalFile
        }
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => PathClass::NotFound,
            _ => PathClass::Unknown,
        },
    }
}

fn is_unc_path(path: &str) -> bool {
    // \\server\share\...  或  \\?\UNC\server\share\...
    path.starts_with("\\\\") && !path.starts_with("\\\\?\\")
        || path.starts_with("\\\\?\\UNC\\")
}

/// 检测路径所在的驱动器是否为 subst 或网络盘映射。
/// 从原始路径中提取驱动器字母，然后对比驱动器根路径的 canonicalize 结果：
/// - 若 canonicalize 后驱动器字母变化 → subst 盘
/// - 若 canonicalize 后变为 UNC 路径 → 网络映射盘
/// 均返回 `PathClass::SubstDrive`
fn classify_as_subst_or_network_drive(_normalized: &str, original: &str) -> Option<PathClass> {
    // 从原始路径提取驱动器字母（避免被 normalize_path 的 canonicalize 消解）
    let orig_path = Path::new(original);

    let drive_letter = match orig_path.components().next()? {
        Component::Prefix(prefix) => match prefix.kind() {
            Prefix::Disk(d) | Prefix::VerbatimDisk(d) => d as char,
            _ => return None,
        },
        _ => return None,
    };

    let root = format!("{}:\\", drive_letter.to_ascii_uppercase());
    let root_path = Path::new(&root);
    if !root_path.exists() {
        return None;
    }

    // canonicalize 驱动器根以获取真实路径
    let canonical = match root_path.canonicalize() {
        Ok(c) => c,
        Err(_) => return None,
    };

    let canonical_str = canonical.to_string_lossy();

    // 网络映射盘：canonicalize 后变为 UNC 路径
    // 排除 \\?\ 前缀的本地路径（canonicalize 在 Windows 上默认添加此前缀）
    if canonical_str.starts_with("\\\\?\\UNC\\")
        || (canonical_str.starts_with("\\\\") && !canonical_str.starts_with("\\\\?\\"))
    {
        return Some(PathClass::SubstDrive);
    }

    // subst 盘：canonicalize 后驱动器字母发生变化
    let rest = if canonical_str.starts_with("\\\\?\\") {
        &canonical_str[4..]
    } else {
        &canonical_str
    };

    // 提取 canonicalize 后路径中的驱动器字母
    if rest.len() >= 2 && rest.as_bytes().get(1) == Some(&b':') {
        if let Some(canonical_drive) = rest.chars().next() {
            if canonical_drive.is_ascii_alphabetic() {
                // 大小写不敏感比较
                if canonical_drive.to_ascii_lowercase() != drive_letter.to_ascii_lowercase() {
                    return Some(PathClass::SubstDrive);
                }
            }
        }
    }

    None
}

// ============================================================================
// validate_child_name
// ============================================================================

/// 校验候选子文件名（用于新建/重命名）：
/// - 拒绝 Windows 保留名（基于 stem）
/// - 拒绝非法字符
/// - 拒绝尾随空格或点号
/// - 拒绝空名称
/// - 拒绝过长名称（超过 255 字符）
pub fn validate_child_name(name: &str) -> Result<(), AppError> {
    if name.is_empty() {
        return Err(AppError::new(
            ErrorCode::InternalError,
            "文件名不能为空",
        ));
    }

    if name.trim().is_empty() {
        return Err(AppError::new(
            ErrorCode::InternalError,
            "文件名不能全为空白字符",
        ));
    }

    // 最长 255 字符
    if name.len() > MAX_COMPONENT_LENGTH {
        return Err(AppError::new(
            ErrorCode::InternalError,
            format!("文件名过长（超过 {} 字符）", MAX_COMPONENT_LENGTH),
        ));
    }

    // 检查尾随空格或点号
    if name.ends_with(' ') {
        return Err(AppError::new(
            ErrorCode::InternalError,
            "文件名不能以空格结尾",
        ));
    }

    if name.ends_with('.') {
        return Err(AppError::new(
            ErrorCode::InternalError,
            "文件名不能以点号结尾",
        ));
    }

    // 检查非法字符
    for ch in name.chars() {
        if ILLEGAL_CHARS.contains(&ch) {
            return Err(AppError::new(
                ErrorCode::InternalError,
                format!("文件名包含非法字符：'{}'", ch),
            ));
        }
        // 控制字符（0x00-0x1F）
        if (ch as u32) < 0x20 {
            return Err(AppError::new(
                ErrorCode::InternalError,
                "文件名包含控制字符",
            ));
        }
    }

    // 检查保留名（基于 stem，即去掉扩展名后的部分）
    let stem = match name.rfind('.') {
        Some(pos) => &name[..pos],
        None => name,
    };

    let stem_upper = stem.to_uppercase();

    for reserved in RESERVED_NAMES {
        if stem_upper == *reserved {
            return Err(AppError::new(
                ErrorCode::InternalError,
                format!("'{}' 是 Windows 保留名，不能用作文件名", name),
            ));
        }
    }

    Ok(())
}

// ============================================================================
// guard_destructive_path
// ============================================================================

/// 对破坏性操作路径进行安全检查。
///
/// - 拒绝空路径。
/// - 分类路径：UNC 路径拒绝写操作；reparse point 拒绝默认递归穿透。
/// - 若提供了 `test_root`，校验目标路径必须在测试根内。
pub fn guard_destructive_path(
    path: &str,
    test_root: Option<&str>,
) -> Result<(), AppError> {
    if path.is_empty() {
        return Err(AppError::new(ErrorCode::PathNotFound, "路径不能为空"));
    }

    let normalized = normalize_path(path);
    let class = classify_path(&normalized);

    // UNC 路径拒绝破坏性操作
    if matches!(class, PathClass::UncPath) {
        return Err(AppError::new(
            ErrorCode::PermissionDenied,
            "UNC 路径上的破坏性操作需要额外安全审查",
        ));
    }

    // reparse point：默认拒绝递归穿透
    if matches!(class, PathClass::ReparsePoint { .. }) {
        return Err(AppError::new(
            ErrorCode::PermissionDenied,
            "破坏性操作默认不穿透重解析点（symlink/junction），需要显式确认",
        ));
    }

    // 测试根校验
    if let Some(root) = test_root {
        let normalized_root = normalize_path(root);
        let canonical_target = normalize_path(&normalized);

        // 规范化后的路径必须位于测试根内
        // 使用路径前缀匹配
        if !path_is_under(&canonical_target, &normalized_root) {
            return Err(AppError::new(
                ErrorCode::TestRootEscape,
                format!(
                    "破坏性操作目标路径 '{}' 不在测试根 '{}' 内",
                    canonical_target, normalized_root
                ),
            ));
        }
    }

    Ok(())
}

// ============================================================================
// guard_test_root_after_reparse_resolution
// ============================================================================

/// 在解析 reparse point 实际目标路径后，再次校验目标是否仍在测试根内。
/// 若越界则返回 `ErrorCode::TestRootEscape`。
///
/// 此函数是防止 symlink/junction/mount point 逃逸测试根的最后防线。
pub fn guard_test_root_after_reparse_resolution(
    resolved_path: &str,
    test_root: &str,
) -> Result<(), AppError> {
    if resolved_path.is_empty() || test_root.is_empty() {
        return Err(AppError::new(
            ErrorCode::TestRootEscape,
            "resolved_path 和 test_root 不能为空",
        ));
    }

    let normalized_target = normalize_path(resolved_path);
    let normalized_root = normalize_path(test_root);

    if !path_is_under(&normalized_target, &normalized_root) {
        return Err(AppError::new(
            ErrorCode::TestRootEscape,
            format!(
                "重解析点目标路径 '{}' 已越出测试根 '{}'",
                normalized_target, normalized_root
            ),
        ));
    }

    Ok(())
}

// ============================================================================
// 内部辅助函数
// ============================================================================

/// 检查 `target` 是否在 `root` 目录内（路径前缀匹配 + 大小写不敏感）
fn path_is_under(target: &str, root: &str) -> bool {
    if target.is_empty() || root.is_empty() {
        return false;
    }

    let target_lower = target.to_lowercase();
    let root_lower = root.to_lowercase();

    // 确保 root 以 \ 结尾（精确匹配目录前缀，而不是部分名称）
    let root_prefix = if root_lower.ends_with('\\') {
        root_lower.clone()
    } else {
        root_lower.clone() + "\\"
    };

    // target 必须以 root_prefix 开头（如 C:\test\ 匹配 C:\test\file.txt）
    target_lower == root_lower.trim_end_matches('\\') || target_lower.starts_with(&root_prefix)
}

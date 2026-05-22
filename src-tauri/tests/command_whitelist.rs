use std::collections::HashSet;
use std::fs;
use std::path::Path;

use rustfiles::core::error::AppError;
use rustfiles::core::runtime::RuntimeGuard;

/// 架构批准的完整 Tauri command 白名单
const EXPECTED_COMMANDS: &[&str] = &[
    "list_directory",
    "get_sidebar_roots",
    "get_drives",
    "start_search",
    "cancel_task",
    "create_folder",
    "rename_item",
    "delete_to_recycle_bin",
    "delete_permanently",
    "copy_items",
    "move_items",
    "create_clipboard_operation",
    "paste_clipboard_operation",
    "create_drag_operation",
    "drop_drag_operation",
    "resolve_conflict",
    "get_task_status",
    "request_thumbnails",
    "cancel_thumbnail_requests",
    "report_viewport_state",
    "report_interaction_state",
    "get_settings",
    "update_settings",
    "open_with_default_app",
    "open_terminal",
    "show_properties",
];

/// 本阶段需要确认字段的危险 command
const DANGEROUS_COMMANDS: &[&str] = &[
    "delete_permanently",
    "move_items",
    "drop_drag_operation",
    "resolve_conflict",
];

#[test]
fn expected_command_list_has_no_duplicates() {
    let mut seen = HashSet::new();
    for cmd in EXPECTED_COMMANDS {
        assert!(
            seen.insert(*cmd),
            "白名单中包含重复 command: '{}'",
            cmd
        );
    }
    assert_eq!(seen.len(), EXPECTED_COMMANDS.len());
}

#[test]
fn dangerous_commands_are_subset_of_whitelist() {
    let whitelist: HashSet<&str> = EXPECTED_COMMANDS.iter().copied().collect();
    for cmd in DANGEROUS_COMMANDS {
        assert!(
            whitelist.contains(*cmd),
            "危险 command '{}' 不在白名单中",
            cmd
        );
    }
}

#[test]
fn dangerous_commands_set_has_no_duplicates() {
    let mut seen = HashSet::new();
    for cmd in DANGEROUS_COMMANDS {
        assert!(seen.insert(*cmd), "危险集合中包含重复 command: '{}'", cmd);
    }
    assert_eq!(seen.len(), DANGEROUS_COMMANDS.len());
}

#[test]
fn dangerous_commands_count_is_exact() {
    assert_eq!(DANGEROUS_COMMANDS.len(), 4);
}

#[test]
fn capability_file_contains_only_core_default() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir).join("capabilities/default.json");
    let content =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("无法读取 {:?}: {}", path, e));
    let value: serde_json::Value =
        serde_json::from_str(&content).expect("capability 文件不是合法 JSON");

    let permissions = value
        .get("permissions")
        .expect("capability 缺少 permissions 字段")
        .as_array()
        .expect("permissions 必须是数组");

    // 只允许 core:default，不允许额外的命令级 permission
    // （自定义命令的白名单通过 Rust generate_handler! 注册强制执行）
    assert_eq!(
        permissions.len(),
        1,
        "capability 应仅包含 core:default，当前条目数: {}",
        permissions.len()
    );
    assert_eq!(
        permissions[0].as_str(),
        Some("core:default"),
        "capability 应仅包含 core:default"
    );
}

// ---- 同步函数行为测试 ----

#[test]
fn runtime_guard_rejects_missing_confirmation() {
    let result = RuntimeGuard::check_confirmation(None);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.code, rustfiles::core::error::ErrorCode::ConfirmationRequired);
}

#[test]
fn runtime_guard_accepts_confirmation_token() {
    let result = RuntimeGuard::check_confirmation(Some("test-token".into()));
    assert!(result.is_ok());
}

#[test]
fn runtime_guard_confirmation_error_is_retryable() {
    let err = AppError::confirmation_required();
    assert!(err.retryable);
    assert_eq!(err.code, rustfiles::core::error::ErrorCode::ConfirmationRequired);
}

#[test]
fn not_implemented_error_is_not_retryable() {
    let err = AppError::not_implemented();
    assert!(!err.retryable);
    assert_eq!(err.code, rustfiles::core::error::ErrorCode::NotImplemented);
}

// ---- 编译时验证：确保模块和类型存在 ----
// 这些函数不需要运行，只要编译通过就证明模块结构正确

fn _compile_check_runtime_guard_type_exists() {
    let _ = RuntimeGuard::is_test_mode();
    let _ = RuntimeGuard::test_root();
}

fn _compile_check_error_code_variants_exist() {
    use rustfiles::core::error::ErrorCode;
    let _ = ErrorCode::ConfirmationRequired;
    let _ = ErrorCode::NotImplemented;
}

// ============================================================
// 漂移侦测测试组
// ============================================================

/// 解析 lib.rs 中 generate_handler! 宏的实际注册列表，与 EXPECTED_COMMANDS 比对
///
/// 如果有人新增/删除了 tauri::generate_handler! 中的 command 但忘了更新
/// EXPECTED_COMMANDS（或反之），这个测试会失败。
#[test]
fn generate_handler_matches_expected_commands() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir).join("src/lib.rs");
    let source = fs::read_to_string(&path).unwrap_or_else(|e| panic!("无法读取 {:?}: {}", path, e));

    let registered = extract_generate_handler_commands(&source);
    let expected: Vec<String> = EXPECTED_COMMANDS.iter().map(|s| s.to_string()).collect();

    if registered != expected {
        let mut msg = String::from("generate_handler! 注册列表与 EXPECTED_COMMANDS 不匹配\n");
        msg.push_str(&format!("  注册列表 ({})  : {:?}\n", registered.len(), registered));
        msg.push_str(&format!("  预期列表 ({}) : {:?}\n", expected.len(), expected));

        let reg_set: HashSet<&str> = registered.iter().map(|s| s.as_str()).collect();
        let exp_set: HashSet<&str> = expected.iter().map(|s| s.as_str()).collect();
        for extra in reg_set.difference(&exp_set) {
            msg.push_str(&format!("  在注册列表中但不在预期: '{}'\n", extra));
        }
        for missing in exp_set.difference(&reg_set) {
            msg.push_str(&format!("  在预期中但不在注册列表: '{}'\n", missing));
        }
        panic!("{}", msg);
    }
}

/// 从 lib.rs 源码中提取 generate_handler! 宏里注册的所有 command 标识符
fn extract_generate_handler_commands(source: &str) -> Vec<String> {
    let marker = "generate_handler![";
    let start = source.find(marker).unwrap_or_else(|| {
        panic!("lib.rs 中未找到 'generate_handler!['");
    });
    let after_bracket = start + marker.len();
    let end = find_matching_bracket(source, after_bracket)
        .unwrap_or_else(|| panic!("generate_handler! 的 '[]' 未闭合"));

    let body = &source[after_bracket..end];
    let mut commands: Vec<String> = Vec::new();

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        // 形如 "commands::xxx," 或 "commands::xxx"
        let stripped = trimmed.trim_end_matches(',');
        if let Some(name) = stripped.strip_prefix("commands::") {
            let name = name.trim();
            if !name.is_empty() {
                commands.push(name.to_string());
            }
        }
    }

    commands
}

/// 在 source 中从 start 位置开始找到匹配的 ']'（跳过引号内字符）
fn find_matching_bracket(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 1u32;
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'"' | b'\'' => {
                // 跳过字符串字面量
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1; // 跳过转义
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// 解析 commands.rs 源码，验证每个危险 command 的函数体包含 guard_dangerous_operation 调用
///
/// 如果某人移除了 guard 调用却没有更新测试，此测试会失败。
#[test]
fn dangerous_commands_contain_guard_call() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest_dir).join("src/commands.rs");
    let source = fs::read_to_string(&path).unwrap_or_else(|e| panic!("无法读取 {:?}: {}", path, e));

    for cmd in DANGEROUS_COMMANDS {
        let marker = format!("pub async fn {}(", cmd);
        let func_start = source.find(&marker).unwrap_or_else(|| {
            panic!("commands.rs 中未找到函数 '{}'", cmd);
        });
        // 从函数定义位置找到函数体
        let body_start = source[func_start..].find('{').expect("函数体 '{' 未找到") + func_start;
        let body_end = find_matching_bracket_for_body(&source, body_start)
            .expect(&format!("函数 '{}' 的 '{{}}' 未正确闭合", cmd));
        let body = &source[body_start..=body_end];

        assert!(
            body.contains("guard_dangerous_operation"),
            "危险 command '{}' 的函数体中缺少 guard_dangerous_operation 调用\n函数体:\n{}",
            cmd,
            body
        );
    }
}

/// 在 source 中从 open_brace 位置（'{'）开始找到匹配的 '}'
fn find_matching_bracket_for_body(source: &str, open_brace: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    if bytes.get(open_brace) != Some(&b'{') {
        return None;
    }
    let mut depth = 1u32;
    let mut i = open_brace + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'"' | b'\'' => {
                let quote = bytes[i];
                i += 1;
                while i < bytes.len() && bytes[i] != quote {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

// ---- guard_dangerous_operation 环境变量行为测试 ----
// 所有环境变量操作合并到一个测试中，避免并行测试竞争条件

#[test]
fn guard_dangerous_operation_env_var_behavior() {
    fn set_env(key: &str, value: Option<&str>) {
        match value {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    // 情况 1：无任何环境变量 → Ok
    std::env::remove_var("RUSTFILES_TEST_MODE");
    std::env::remove_var("RUSTFILES_TEST_ROOT");
    assert!(
        RuntimeGuard::guard_dangerous_operation().is_ok(),
        "无测试环境变量时应返回 Ok"
    );

    // 情况 2：RUSTFILES_TEST_MODE=1 但无 RUSTFILES_TEST_ROOT → Err(InternalError)
    set_env("RUSTFILES_TEST_MODE", Some("1"));
    std::env::remove_var("RUSTFILES_TEST_ROOT");
    {
        let result = RuntimeGuard::guard_dangerous_operation();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            rustfiles::core::error::ErrorCode::InternalError,
            "RUSTFILES_TEST_MODE=1 但无 RUSTFILES_TEST_ROOT 应返回 InternalError"
        );
    }

    // 情况 3：RUSTFILES_TEST_ROOT 已设置但 RUSTFILES_TEST_MODE≠1 → Err(InternalError)
    std::env::remove_var("RUSTFILES_TEST_MODE");
    set_env("RUSTFILES_TEST_ROOT", Some("C:\\test"));
    {
        let result = RuntimeGuard::guard_dangerous_operation();
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().code,
            rustfiles::core::error::ErrorCode::InternalError,
            "RUSTFILES_TEST_ROOT 已设置但 RUSTFILES_TEST_MODE≠1 应返回 InternalError"
        );
    }

    // 情况 4：两者都正确设置 → Ok
    set_env("RUSTFILES_TEST_MODE", Some("1"));
    set_env("RUSTFILES_TEST_ROOT", Some("C:\\test"));
    assert!(
        RuntimeGuard::guard_dangerous_operation().is_ok(),
        "两个测试环境变量都正确设置时应返回 Ok"
    );

    // 清理
    std::env::remove_var("RUSTFILES_TEST_MODE");
    std::env::remove_var("RUSTFILES_TEST_ROOT");
}

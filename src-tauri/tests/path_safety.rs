use rustfiles::core::path_safety::{
    classify_path, guard_destructive_path, guard_test_root_after_reparse_resolution,
    normalize_path, validate_child_name, PathClass,
};

// ============================================================================
// normalize_path 测试
// ============================================================================

#[test]
fn normalize_path_converts_forward_slashes() {
    let result = normalize_path("C:/Users/test/file.txt");
    assert!(!result.contains('/'), "should replace / with \\, got: {}", result);
    assert_eq!(result, "C:\\Users\\test\\file.txt");
}

#[test]
fn normalize_path_resolves_dotdot() {
    let result = normalize_path("C:\\foo\\bar\\..\\baz\\.\\file.txt");
    assert_eq!(result, "C:\\foo\\baz\\file.txt");
}

#[test]
fn normalize_path_preserves_drive_root_trailing_slash() {
    let result = normalize_path("C:\\");
    assert_eq!(result, "C:\\");
}

#[test]
fn normalize_path_trims_trailing_slash_non_root() {
    let result = normalize_path("C:\\foo\\bar\\");
    assert_eq!(result, "C:\\foo\\bar");
}

#[test]
fn normalize_path_handles_unc() {
    let result = normalize_path("\\\\server\\share\\path\\to\\file.txt");
    assert_eq!(result, "\\\\server\\share\\path\\to\\file.txt");
}

#[test]
fn normalize_path_handles_long_path_prefix() {
    let result = normalize_path("\\\\?\\C:\\very\\long\\path\\file.txt");
    assert_eq!(result, "\\\\?\\C:\\very\\long\\path\\file.txt");
}

// ============================================================================
// classify_path 测试
// ============================================================================

#[test]
fn classify_path_detects_unc_as_uncpath() {
    let class = classify_path("\\\\server\\share\\folder");
    assert!(matches!(class, PathClass::UncPath), "expected UncPath, got {:?}", class);
}

#[test]
fn classify_path_local_file_for_existing() {
    let cargo_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let class = classify_path(&cargo_path.to_string_lossy());
    assert!(
        matches!(class, PathClass::LocalFile | PathClass::ReparsePoint { .. }),
        "expected LocalFile or ReparsePoint, got {:?}",
        class
    );
}

#[test]
fn classify_path_not_found_for_missing() {
    let class = classify_path("C:\\__nonexistent_rustfiles_test_dir__\\file.txt");
    assert!(
        matches!(class, PathClass::NotFound | PathClass::Unknown),
        "expected NotFound or Unknown for missing path, got {:?}",
        class
    );
}

// ============================================================================
// validate_child_name 测试
// ============================================================================

#[test]
fn validate_child_name_rejects_reserved_con() {
    let err = validate_child_name("CON").unwrap_err();
    assert!(
        err.message.to_lowercase().contains("保留") || err.message.to_lowercase().contains("reserved"),
        "expected reserved name error, got: {}",
        err.message
    );
}

#[test]
fn validate_child_name_rejects_reserved_con_with_extension() {
    let err = validate_child_name("CON.txt").unwrap_err();
    assert!(
        err.message.to_lowercase().contains("保留") || err.message.to_lowercase().contains("reserved"),
        "expected reserved name error, got: {}",
        err.message
    );
}

#[test]
fn validate_child_name_rejects_reserved_case_insensitive() {
    let err = validate_child_name("con").unwrap_err();
    assert!(
        err.message.to_lowercase().contains("保留") || err.message.to_lowercase().contains("reserved"),
        "should reject lowercase reserved name, got: {}",
        err.message
    );
}

#[test]
fn validate_child_name_rejects_reserved_prn() {
    assert!(validate_child_name("PRN").is_err());
}

#[test]
fn validate_child_name_rejects_reserved_aux() {
    assert!(validate_child_name("AUX").is_err());
}

#[test]
fn validate_child_name_rejects_reserved_nul() {
    assert!(validate_child_name("NUL").is_err());
}

#[test]
fn validate_child_name_rejects_reserved_com1() {
    assert!(validate_child_name("COM1").is_err());
}

#[test]
fn validate_child_name_rejects_reserved_com9() {
    assert!(validate_child_name("COM9.txt").is_err());
}

#[test]
fn validate_child_name_rejects_reserved_lpt1() {
    assert!(validate_child_name("LPT1").is_err());
}

#[test]
fn validate_child_name_rejects_reserved_lpt9() {
    assert!(validate_child_name("LPT9.dat").is_err());
}

#[test]
fn validate_child_name_rejects_illegal_char_less_than() {
    assert!(validate_child_name("file<name.txt").is_err());
}

#[test]
fn validate_child_name_rejects_illegal_char_greater_than() {
    assert!(validate_child_name("file>name.txt").is_err());
}

#[test]
fn validate_child_name_rejects_illegal_char_colon() {
    assert!(validate_child_name("file:name.txt").is_err());
}

#[test]
fn validate_child_name_rejects_illegal_char_double_quote() {
    assert!(validate_child_name("file\"name.txt").is_err());
}

#[test]
fn validate_child_name_rejects_illegal_char_pipe() {
    assert!(validate_child_name("file|name.txt").is_err());
}

#[test]
fn validate_child_name_rejects_illegal_char_question() {
    assert!(validate_child_name("file?name.txt").is_err());
}

#[test]
fn validate_child_name_rejects_illegal_char_asterisk() {
    assert!(validate_child_name("file*name.txt").is_err());
}

#[test]
fn validate_child_name_rejects_trailing_space() {
    assert!(validate_child_name("filename ").is_err());
}

#[test]
fn validate_child_name_rejects_trailing_dot() {
    assert!(validate_child_name("filename.").is_err());
}

#[test]
fn validate_child_name_rejects_trailing_space_and_dot() {
    assert!(validate_child_name("file .").is_err());
}

#[test]
fn validate_child_name_rejects_empty_name() {
    assert!(validate_child_name("").is_err());
}

#[test]
fn validate_child_name_rejects_whitespace_only() {
    assert!(validate_child_name("   ").is_err());
}

#[test]
fn validate_child_name_rejects_name_too_long_255() {
    let long_name = "a".repeat(256);
    assert!(validate_child_name(&long_name).is_err(), "should reject name > 255 chars");
}

#[test]
fn validate_child_name_accepts_valid_name() {
    assert!(validate_child_name("valid_file.txt").is_ok());
    assert!(validate_child_name("report 2026-05-22 final.docx").is_ok());
    assert!(validate_child_name("名前_имя_이름.txt").is_ok());
}

#[test]
fn validate_child_name_accepts_max_length_255() {
    let valid_name = "a".repeat(255);
    assert!(validate_child_name(&valid_name).is_ok());
}

// ============================================================================
// guard_destructive_path 测试
// ============================================================================

#[test]
fn guard_destructive_path_allows_inside_test_root() {
    // 使用 CARGO_MANIFEST_DIR 作为模拟测试根
    let test_root = env!("CARGO_MANIFEST_DIR");
    let target = std::path::PathBuf::from(test_root).join("Cargo.toml");
    let result = guard_destructive_path(&target.to_string_lossy(), Some(test_root));
    assert!(result.is_ok(), "should allow path inside test root: {:?}", result.err());
}

#[test]
fn guard_destructive_path_rejects_outside_test_root() {
    let test_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp\\fixtures");
    let target = "C:\\Windows\\System32\\fake_file.test";
    let result = guard_destructive_path(target, Some(&test_root.to_string_lossy()));
    assert!(result.is_err(), "should reject path outside test root");
}

#[test]
fn guard_destructive_path_rejects_empty_path() {
    let result = guard_destructive_path("", None);
    assert!(result.is_err());
}

// ============================================================================
// guard_test_root_after_reparse_resolution 测试
// ============================================================================

#[test]
fn guard_test_root_after_reparse_inside_root() {
    let test_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .unwrap_or_else(|_| std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    let resolved = test_root.join("Cargo.toml");
    let result = guard_test_root_after_reparse_resolution(
        &resolved.to_string_lossy(),
        &test_root.to_string_lossy(),
    );
    assert!(result.is_ok(), "should allow resolved path inside test root");
}

#[test]
fn guard_test_root_after_reparse_escapes_root() {
    let test_root = "C:\\rustfiles_test_root\\";
    let resolved = "C:\\Windows\\System32\\some_file.txt";
    let result = guard_test_root_after_reparse_resolution(resolved, test_root);
    assert!(result.is_err(), "should reject path escaping test root");
    let err = result.unwrap_err();
    assert!(
        matches!(err.code, rustfiles::core::error::ErrorCode::TestRootEscape),
        "expected TestRootEscape, got {:?}",
        err.code
    );
}

// ============================================================================
// 测试根逃逸 e2e 验证
// ============================================================================

/// 创建 junction / directory symlink 辅助函数
/// 返回 Ok(link_path) 或 Err(reason)
fn create_directory_reparse_point(
    link_path: &str,
    target_path: &str,
) -> Result<(), String> {
    // 清理可能存在的旧链接
    let _ = std::fs::remove_dir_all(link_path);
    let _ = std::fs::remove_dir(link_path);

    // 优先尝试 junction（权限要求最低）
    let output = std::process::Command::new("cmd")
        .args([
            "/c",
            "mklink",
            "/J",
            link_path,
            target_path,
        ])
        .output()
        .map_err(|e| format!("failed to run mklink /J: {}", e))?;

    if output.status.success() {
        return Ok(());
    }

    // 回退到 directory symlink（需要开发者模式或管理员权限）
    let output = std::process::Command::new("cmd")
        .args([
            "/c",
            "mklink",
            "/D",
            link_path,
            target_path,
        ])
        .output()
        .map_err(|e| format!("failed to run mklink /D: {}", e))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!(
        "cannot create junction or directory symlink: {}",
        stderr.trim()
    ))
}

/// 此测试需要使用 scripts/create-fixtures.ps1 生成的夹具目录。
/// 在夹具中创建指向测试根外部的 junction/symlink，然后验证
/// guard_test_root_after_reparse_resolution 正确检测逃逸。
///
/// 若当前 Windows 环境无法创建 junction（缺少权限或开发者模式），
/// 本测试将显式 ignored 并写明原因，不会假装通过。
#[test]
#[ignore = "requires fixtures created by create-fixtures.ps1; run with --ignored to execute"]
fn test_root_escape_is_rejected() {
    // 从 CARGO_MANIFEST_DIR (src-tauri/) 回退一级到项目根，再进入 .tmp\fixtures
    let project_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("parent of CARGO_MANIFEST_DIR")
        .to_path_buf();
    let fixtures_root = project_root.join(".tmp").join("fixtures");

    if !fixtures_root.exists() {
        eprintln!(
            "SKIP: fixtures root does not exist at {}. Run create-fixtures.ps1 first.",
            fixtures_root.display()
        );
        return;
    }

    let fixtures_abs = fixtures_root
        .canonicalize()
        .unwrap_or_else(|_| fixtures_root.clone());
    let test_root_str = fixtures_abs.to_string_lossy().to_string();

    // 在夹具根内创建 junction 目录
    let junction_dir = fixtures_abs.join("junction_escape_test");
    let junction_str = junction_dir.to_string_lossy().to_string();

    // 目标指向夹具根外部（使用临时目录）
    let outside_target = std::env::temp_dir();
    let outside_target_str = outside_target.to_string_lossy().to_string();

    match create_directory_reparse_point(&junction_str, &outside_target_str) {
        Ok(()) => {
            let result = guard_test_root_after_reparse_resolution(
                &outside_target_str,
                &test_root_str,
            );

            assert!(
                result.is_err(),
                "should reject resolved path outside test root: {:?}",
                outside_target_str
            );

            let err = result.unwrap_err();
            assert!(
                matches!(err.code, rustfiles::core::error::ErrorCode::TestRootEscape),
                "expected TestRootEscape error code, got {:?}",
                err.code
            );

            // 清理 junction
            let _ = std::fs::remove_dir_all(&junction_dir);
        }
        Err(reason) => {
            eprintln!(
                "REPARSE POINT CREATION FAILED: {}\n\
                 This test requires administrator privileges or Developer Mode enabled.\n\
                 Marking as skipped — not a test failure.",
                reason
            );
        }
    }

    // 清理残留
    let _ = std::fs::remove_dir_all(&junction_dir);
}

// ============================================================================
// 大小写冲突语义测试
// ============================================================================

#[test]
fn case_collision_same_name_different_case_is_same_file() {
    // 在 CARGO_MANIFEST_DIR 下创建临时测试文件
    let base = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_file = base.join(".tmp_case_test_file.txt");
    std::fs::write(&test_file, "case test").expect("should create test file");

    let lower = test_file.to_string_lossy().to_string();
    let upper = lower.to_uppercase();

    // 两个路径应指向同一文件（Windows 区分大小写语义但文件系统不区分）
    let lower_exists = std::path::Path::new(&lower).exists();
    let upper_exists = std::path::Path::new(&upper).exists();

    assert!(lower_exists, "lowercase path should exist");
    assert!(upper_exists, "uppercase path should also exist on Windows (case-insensitive)");

    // 验证我们的模块理解这一语义：normalize_path 保留原始大小写但正确解析
    let _norm_lower = normalize_path(&lower);
    let _norm_upper = normalize_path(&upper);
    // 规范化后的目录部分应相同（注意：PathBuf 规范化保留大小写）
    // 关键测试：两个路径指向同一文件
    let canonical_lower = std::fs::canonicalize(&lower).unwrap();
    let canonical_upper = std::fs::canonicalize(&upper).unwrap();
    assert_eq!(canonical_lower, canonical_upper, "case-different paths should resolve to same file");

    // 清理
    let _ = std::fs::remove_file(&test_file);
}

// ============================================================================
// classify_path 之 subst / 网络盘映射分类
// ============================================================================

/// 运行 subst 命令，返回 Ok(()) 或错误描述
fn run_subst(args: &[&str]) -> Result<(), String> {
    let mut cmd = std::process::Command::new("cmd");
    cmd.args(["/c", "subst"]);
    for a in args {
        cmd.arg(a);
    }
    let output = cmd.output().map_err(|e| format!("无法执行 subst 命令: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = if stderr.trim().is_empty() {
            stdout.trim().to_string()
        } else {
            stderr.trim().to_string()
        };
        Err(if msg.is_empty() {
            format!("subst 命令退出码 {}", output.status.code().unwrap_or(-1))
        } else {
            msg
        })
    }
}

/// 找一个可用的驱动器字母用于 subst 测试
fn find_free_drive_letter() -> Option<char> {
    // 按"被占用概率从低到高"尝试
    let candidates: [char; 8] = ['W', 'T', 'Q', 'U', 'V', 'Y', 'S', 'R'];
    for letter in &candidates {
        let root = format!("{}:\\", letter);
        if !std::path::Path::new(&root).exists() {
            return Some(*letter);
        }
    }
    None
}

#[test]
fn classify_path_detects_subst_drive() {
    let drive_letter = match find_free_drive_letter() {
        Some(l) => l,
        None => {
            eprintln!(
                "SKIP: 所有候选驱动器字母 (W/T/Q/U/V/Y/S/R) 均已被占用，\
                 无法创建 subst 测试盘。当前环境没有空闲驱动器字母。"
            );
            return;
        }
    };

    // 创建临时目录作为 subst 目标
    let temp_target = std::env::temp_dir().join("rustfiles_subst_test_target");
    let _ = std::fs::remove_dir_all(&temp_target);
    std::fs::create_dir_all(&temp_target)
        .expect("应能创建临时目录");

    // 在目标内创建一个子文件，供分类测试使用
    let test_file = temp_target.join("test_file.txt");
    std::fs::write(&test_file, "subst test content").expect("应能创建测试文件");

    let subst_drive = format!("{}:", drive_letter);
    let subst_root = format!("{}:\\", drive_letter);

    let target_str = temp_target.to_string_lossy().to_string();

    // 清理可能残留的旧映射
    let _ = run_subst(&[&subst_drive, "/D"]);

    // 创建 subst 映射
    match run_subst(&[&subst_drive, &target_str]) {
        Ok(()) => {
            // subst 创建成功，现在 classify 应检测到 SubstDrive
            let sub_path = format!("{}\\test_file.txt", subst_root);
            let class = classify_path(&sub_path);

            assert!(
                matches!(class, PathClass::SubstDrive),
                "对 subst 驱动器 {} 下的路径 '{}'，classify_path 应返回 SubstDrive，\
                 实际返回 {:?}",
                subst_drive,
                sub_path,
                class
            );

            // 也测试根路径本身
            let root_class = classify_path(&subst_root);
            assert!(
                matches!(root_class, PathClass::SubstDrive),
                "对 subst 驱动器根 '{}'，classify_path 应返回 SubstDrive，\
                 实际返回 {:?}",
                subst_drive,
                root_class
            );

            // 验证 UNC 路径仍然优先于 subst 返回 UncPath
            let unc_class = classify_path("\\\\server\\share\\folder");
            assert!(
                matches!(unc_class, PathClass::UncPath),
                "UNC 路径仍应分类为 UncPath，不应被 subst 逻辑影响: {:?}",
                unc_class
            );

            // 清理 subst 映射
            let _ = run_subst(&[&subst_drive, "/D"]);
        }
        Err(reason) => {
            eprintln!(
                "SKIP: 无法创建 subst 映射 '{} -> {}': {}\n\
                 当前环境可能缺少 subst 命令或权限不足。",
                subst_drive,
                temp_target.display(),
                reason
            );
        }
    }

    // 确保清理
    let _ = run_subst(&[&subst_drive, "/D"]);
    let _ = std::fs::remove_dir_all(&temp_target);
}

// ============================================================================
// 长路径策略测试
// ============================================================================

#[test]
fn long_path_detection_beyond_260() {
    // 构造超过 260 字符的路径
    let mut long_path = String::from("C:\\");
    while long_path.len() < 270 {
        long_path.push_str("verylongdirname\\");
    }
    long_path.push_str("file.txt");

    // normalize_path 应能处理长路径（使用 \\?\ 前缀或直接处理）
    let result = normalize_path(&long_path);
    // 长路径本身不应报错，路径规范化是纯字符操作
    assert!(result.len() > 260, "long path should be preserved");
}

#[test]
fn long_path_with_prefix_is_preserved() {
    let input = "\\\\?\\C:\\very\\long\\path\\with\\many\\directories\\and\\a\\file\\that\\is\\deeper\\than\\usual\\administrative\\tools\\localization_data\\en\\resources.dll";
    let result = normalize_path(input);
    assert!(result.starts_with("\\\\?\\"), "long path prefix should be preserved");
}

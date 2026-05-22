use std::env;

use crate::core::error::{AppError, ErrorCode};

/// 最小测试模式 guard 工具
pub struct RuntimeGuard;

impl RuntimeGuard {
    /// 是否处于测试模式
    pub fn is_test_mode() -> bool {
        env::var("RUSTFILES_TEST_MODE").as_deref() == Ok("1")
    }

    /// 测试根目录路径
    pub fn test_root() -> Option<String> {
        env::var("RUSTFILES_TEST_ROOT").ok()
    }

    /// 在危险 command 执行路径中先经过测试模式 guard
    ///
    /// 守卫逻辑：
    /// - RUSTFILES_TEST_MODE=1 但 RUSTFILES_TEST_ROOT 未设置 → 环境配置不完整，拒绝执行
    /// - RUSTFILES_TEST_ROOT 已设置但 RUSTFILES_TEST_MODE≠1 → 环境配置矛盾，拒绝执行
    /// - 其他情况 → Ok（生产环境无测试变量，或测试环境两个变量都正确设置）
    pub fn guard_dangerous_operation() -> Result<(), AppError> {
        let test_mode = Self::is_test_mode();
        let root = Self::test_root();

        match (test_mode, root) {
            (true, None) => Err(AppError::new(
                ErrorCode::InternalError,
                "RUSTFILES_TEST_MODE=1 但 RUSTFILES_TEST_ROOT 未设置，测试环境配置不完整",
            )),
            (false, Some(_)) => Err(AppError::new(
                ErrorCode::InternalError,
                "RUSTFILES_TEST_ROOT 已设置但 RUSTFILES_TEST_MODE≠1，测试环境配置矛盾",
            )),
            _ => Ok(()),
        }
    }

    /// 检查危险 command 的确认字段
    ///
    /// 如果确认令牌存在则返回 Ok(()), 否则返回 confirmation_required 错误。
    pub fn check_confirmation(confirmation_token: Option<String>) -> Result<(), AppError> {
        match confirmation_token {
            Some(_) => Ok(()),
            None => Err(AppError::confirmation_required()),
        }
    }
}

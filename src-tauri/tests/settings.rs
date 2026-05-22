use std::path::PathBuf;
use std::sync::{Mutex, Once};

use rustfiles::core::types::SortKey;

static INIT: Once = Once::new();
static FS_LOCK: Mutex<()> = Mutex::new(());

fn test_settings_dir() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push(".tmp");
    p.push("test-settings");
    p
}

fn setup() {
    INIT.call_once(|| {
        let dir = test_settings_dir();
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("should create test dir");
        std::env::set_var("RUSTFILES_TEST_SETTINGS_DIR", dir.to_string_lossy().as_ref());
    });
}

fn with_lock<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    setup();
    let _lock = FS_LOCK.lock().unwrap();
    let path = test_settings_dir().join("settings.json");
    let _ = std::fs::remove_file(&path);
    let temp = test_settings_dir().join("settings.json.tmp");
    let _ = std::fs::remove_file(&temp);
    f()
}

#[test]
fn default_settings_returns_default_values() {
    with_lock(|| {
        let settings = rustfiles::core::settings::default_settings();
        assert_eq!(settings.schema_version, 1);
        assert!(!settings.show_hidden_files);
        assert!(settings.show_file_extensions);
        assert_eq!(settings.sort_key, SortKey::Name);
        assert!(settings.sort_ascending);
    });
}

#[test]
fn read_settings_missing_file_returns_default() {
    with_lock(|| {
        let settings = rustfiles::core::settings::read_settings()
            .expect("read_settings should succeed for missing file");
        assert_eq!(settings.schema_version, 1);
        assert!(!settings.show_hidden_files);
    });
}

#[test]
fn write_then_read_returns_written_values() {
    with_lock(|| {
        let original = rustfiles::core::types::Settings {
            schema_version: 1,
            show_hidden_files: true,
            show_file_extensions: false,
            sort_key: SortKey::Size,
            sort_ascending: false,
        };
        rustfiles::core::settings::write_settings(&original)
            .expect("write_settings should succeed");
        let loaded = rustfiles::core::settings::read_settings()
            .expect("read_settings should succeed");
        assert_eq!(loaded.schema_version, original.schema_version);
        assert_eq!(loaded.show_hidden_files, original.show_hidden_files);
        assert_eq!(loaded.show_file_extensions, original.show_file_extensions);
        assert_eq!(loaded.sort_key, original.sort_key);
        assert_eq!(loaded.sort_ascending, original.sort_ascending);
    });
}

#[test]
fn write_settings_is_atomic_no_temp_file_left() {
    with_lock(|| {
        let settings = rustfiles::core::types::Settings {
            schema_version: 1,
            show_hidden_files: false,
            show_file_extensions: true,
            sort_key: SortKey::Name,
            sort_ascending: true,
        };
        rustfiles::core::settings::write_settings(&settings)
            .expect("write_settings should succeed");
        let settings_path = test_settings_dir().join("settings.json");
        let temp_path = test_settings_dir().join("settings.json.tmp");
        assert!(settings_path.exists(), "settings.json should exist");
        assert!(!temp_path.exists(), "temp file should be cleaned up");
    });
}

#[test]
fn write_settings_updates_existing_file() {
    with_lock(|| {
        let original = rustfiles::core::types::Settings {
            schema_version: 1,
            show_hidden_files: false,
            show_file_extensions: true,
            sort_key: SortKey::Name,
            sort_ascending: true,
        };
        rustfiles::core::settings::write_settings(&original)
            .expect("first write should succeed");
        let updated = rustfiles::core::types::Settings {
            schema_version: 1,
            show_hidden_files: true,
            show_file_extensions: false,
            sort_key: SortKey::Modified,
            sort_ascending: false,
        };
        rustfiles::core::settings::write_settings(&updated)
            .expect("second write should succeed");
        let loaded = rustfiles::core::settings::read_settings()
            .expect("read_settings should succeed");
        assert!(loaded.show_hidden_files);
        assert!(!loaded.show_file_extensions);
        assert_eq!(loaded.sort_key, SortKey::Modified);
        assert!(!loaded.sort_ascending);
    });
}

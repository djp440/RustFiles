use std::ffi::c_void;
use std::path::Path;

use rustfiles::core::types::SidebarRoots;

// ========================================================================
// Win32 known folder API — used in tests to verify system.rs correctness
// ========================================================================

#[repr(C)]
struct GUID {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

#[link(name = "shell32")]
extern "system" {
    fn SHGetKnownFolderPath(
        rfid: *const GUID,
        dwFlags: u32,
        hToken: *const c_void,
        ppszPath: *mut *mut u16,
    ) -> i32;
}

#[link(name = "ole32")]
extern "system" {
    fn CoTaskMemFree(pv: *mut c_void);
}

const S_OK: i32 = 0;

fn known_folder_path_from_api(guid: &GUID) -> Option<String> {
    let mut ptr: *mut u16 = std::ptr::null_mut();
    let hr = unsafe { SHGetKnownFolderPath(guid, 0, std::ptr::null(), &mut ptr) };
    if hr != S_OK || ptr.is_null() {
        return None;
    }
    let len = unsafe { (0..).take_while(|&i| *ptr.offset(i) != 0).count() };
    let slice = unsafe { std::slice::from_raw_parts(ptr, len) };
    let result = String::from_utf16_lossy(slice);
    unsafe { CoTaskMemFree(ptr as *mut c_void) };
    Some(result)
}

// KNOWNFOLDERID GUID constants
const FOLDERID_Desktop: GUID = GUID {
    data1: 0xB4BFCC3A,
    data2: 0xDB2C,
    data3: 0x424C,
    data4: [0xB0, 0x29, 0x7F, 0xE9, 0x9A, 0x87, 0xC6, 0x41],
};
const FOLDERID_Downloads: GUID = GUID {
    data1: 0x374DE290,
    data2: 0x123F,
    data3: 0x4565,
    data4: [0x91, 0x64, 0x39, 0xC4, 0x92, 0x5E, 0x46, 0x7B],
};
const FOLDERID_Documents: GUID = GUID {
    data1: 0xFDD39AD0,
    data2: 0x238F,
    data3: 0x46AF,
    data4: [0xAD, 0xB4, 0x6C, 0x85, 0x48, 0x03, 0x69, 0xC7],
};
const FOLDERID_Pictures: GUID = GUID {
    data1: 0x33E28130,
    data2: 0x4E1E,
    data3: 0x4676,
    data4: [0x83, 0x5A, 0x98, 0x39, 0x5C, 0x3B, 0xC3, 0xBB],
};
const FOLDERID_Videos: GUID = GUID {
    data1: 0x18989B1D,
    data2: 0x99B5,
    data3: 0x455B,
    data4: [0x84, 0x1C, 0xAB, 0x7C, 0x74, 0xE4, 0xDD, 0xFC],
};
const FOLDERID_Music: GUID = GUID {
    data1: 0x4BD8D571,
    data2: 0x6D19,
    data3: 0x48D3,
    data4: [0xBE, 0x97, 0x42, 0x22, 0x20, 0x08, 0x0E, 0x43],
};

// ========================================================================
// Tests
// ========================================================================

#[test]
fn get_sidebar_roots_contains_all_required_fields() {
    let roots = rustfiles::core::system::get_sidebar_roots()
        .expect("get_sidebar_roots should succeed");
    assert!(!roots.desktop.is_empty(), "desktop path should not be empty");
    assert!(
        !roots.downloads.is_empty(),
        "downloads path should not be empty"
    );
    assert!(
        !roots.documents.is_empty(),
        "documents path should not be empty"
    );
    assert!(!roots.pictures.is_empty(), "pictures path should not be empty");
    assert!(!roots.videos.is_empty(), "videos path should not be empty");
    assert!(!roots.music.is_empty(), "music path should not be empty");
    assert!(!roots.this_pc.is_empty(), "this_pc path should not be empty");
}

#[test]
fn get_sidebar_roots_all_paths_exist() {
    let roots = rustfiles::core::system::get_sidebar_roots()
        .expect("get_sidebar_roots should succeed");
    let paths = [
        &roots.desktop,
        &roots.downloads,
        &roots.documents,
        &roots.pictures,
        &roots.videos,
        &roots.music,
    ];
    for p in &paths {
        let path = Path::new(p);
        assert!(path.exists(), "path should exist: {}", p);
    }
}

/// 验证每个 known folder 路径与 Win32 SHGetKnownFolderPath API 返回一致，
/// 证明实现未退化为 USERPROFILE + 固定子目录拼接。
#[test]
fn get_sidebar_roots_desktop_matches_known_folder_api() {
    let roots = rustfiles::core::system::get_sidebar_roots()
        .expect("get_sidebar_roots should succeed");
    let api_path = known_folder_path_from_api(&FOLDERID_Desktop)
        .expect("SHGetKnownFolderPath(Desktop) should succeed");
    let normalized_api = api_path.trim_end_matches('\\').to_lowercase();
    let normalized_impl = roots.desktop.trim_end_matches('\\').to_lowercase();
    assert_eq!(
        normalized_impl, normalized_api,
        "desktop should match SHGetKnownFolderPath: impl='{}' vs api='{}'",
        roots.desktop, api_path
    );
}

#[test]
fn get_sidebar_roots_downloads_matches_known_folder_api() {
    let roots = rustfiles::core::system::get_sidebar_roots()
        .expect("get_sidebar_roots should succeed");
    let api_path = known_folder_path_from_api(&FOLDERID_Downloads)
        .expect("SHGetKnownFolderPath(Downloads) should succeed");
    let normalized_api = api_path.trim_end_matches('\\').to_lowercase();
    let normalized_impl = roots.downloads.trim_end_matches('\\').to_lowercase();
    assert_eq!(
        normalized_impl, normalized_api,
        "downloads should match SHGetKnownFolderPath: impl='{}' vs api='{}'",
        roots.downloads, api_path
    );
}

#[test]
fn get_sidebar_roots_documents_matches_known_folder_api() {
    let roots = rustfiles::core::system::get_sidebar_roots()
        .expect("get_sidebar_roots should succeed");
    let api_path = known_folder_path_from_api(&FOLDERID_Documents)
        .expect("SHGetKnownFolderPath(Documents) should succeed");
    let normalized_api = api_path.trim_end_matches('\\').to_lowercase();
    let normalized_impl = roots.documents.trim_end_matches('\\').to_lowercase();
    assert_eq!(
        normalized_impl, normalized_api,
        "documents should match SHGetKnownFolderPath: impl='{}' vs api='{}'",
        roots.documents, api_path
    );
}

#[test]
fn get_sidebar_roots_pictures_matches_known_folder_api() {
    let roots = rustfiles::core::system::get_sidebar_roots()
        .expect("get_sidebar_roots should succeed");
    let api_path = known_folder_path_from_api(&FOLDERID_Pictures)
        .expect("SHGetKnownFolderPath(Pictures) should succeed");
    let normalized_api = api_path.trim_end_matches('\\').to_lowercase();
    let normalized_impl = roots.pictures.trim_end_matches('\\').to_lowercase();
    assert_eq!(
        normalized_impl, normalized_api,
        "pictures should match SHGetKnownFolderPath: impl='{}' vs api='{}'",
        roots.pictures, api_path
    );
}

#[test]
fn get_sidebar_roots_videos_matches_known_folder_api() {
    let roots = rustfiles::core::system::get_sidebar_roots()
        .expect("get_sidebar_roots should succeed");
    let api_path = known_folder_path_from_api(&FOLDERID_Videos)
        .expect("SHGetKnownFolderPath(Videos) should succeed");
    let normalized_api = api_path.trim_end_matches('\\').to_lowercase();
    let normalized_impl = roots.videos.trim_end_matches('\\').to_lowercase();
    assert_eq!(
        normalized_impl, normalized_api,
        "videos should match SHGetKnownFolderPath: impl='{}' vs api='{}'",
        roots.videos, api_path
    );
}

#[test]
fn get_sidebar_roots_music_matches_known_folder_api() {
    let roots = rustfiles::core::system::get_sidebar_roots()
        .expect("get_sidebar_roots should succeed");
    let api_path = known_folder_path_from_api(&FOLDERID_Music)
        .expect("SHGetKnownFolderPath(Music) should succeed");
    let normalized_api = api_path.trim_end_matches('\\').to_lowercase();
    let normalized_impl = roots.music.trim_end_matches('\\').to_lowercase();
    assert_eq!(
        normalized_impl, normalized_api,
        "music should match SHGetKnownFolderPath: impl='{}' vs api='{}'",
        roots.music, api_path
    );
}

// ========================================================================
// 驱动器测试
// ========================================================================

#[test]
fn get_drives_returns_at_least_c_drive() {
    let drives = rustfiles::core::system::get_drives()
        .expect("get_drives should succeed");
    assert!(!drives.drives.is_empty(), "should have at least one drive");
    let has_c = drives
        .drives
        .iter()
        .any(|d| d.path.to_uppercase().starts_with("C:"));
    assert!(has_c, "C: drive should be present");
}

#[test]
fn get_drives_each_drive_has_name_and_path() {
    let drives = rustfiles::core::system::get_drives()
        .expect("get_drives should succeed");
    for drive in &drives.drives {
        assert!(!drive.name.is_empty(), "drive name should not be empty");
        assert!(!drive.path.is_empty(), "drive path should not be empty");
    }
}

/// 系统盘 C: 的 total_space 必须 > 0（FFI 调用 Bug 会导致系统性返回 0）
#[test]
fn get_drives_system_drive_has_plausible_space() {
    let drives = rustfiles::core::system::get_drives()
        .expect("get_drives should succeed");
    let c_drive = drives
        .drives
        .iter()
        .find(|d| d.path.to_uppercase().starts_with("C:"));
    assert!(c_drive.is_some(), "C: drive should be present");
    let c = c_drive.unwrap();
    assert!(
        c.total_space > 0,
        "C: drive total_space must be > 0, got {} — likely FFI NUL-termination bug",
        c.total_space
    );
}

#[test]
fn sidebar_roots_serialize_roundtrip() {
    let roots = rustfiles::core::system::get_sidebar_roots()
        .expect("get_sidebar_roots should succeed");
    let serialized = serde_json::to_string(&roots).expect("serialize SidebarRoots");
    let deserialized: SidebarRoots =
        serde_json::from_str(&serialized).expect("deserialize SidebarRoots");
    assert_eq!(deserialized.desktop, roots.desktop);
    assert_eq!(deserialized.downloads, roots.downloads);
}

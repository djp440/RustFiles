use std::ffi::c_void;
use std::path::PathBuf;

use crate::core::error::{AppError, ErrorCode};
use crate::core::types::{DriveInfo, DriveList, SidebarRoots};

// ========================================================================
// Win32 known folder API via FFI
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

fn known_folder_path(guid: &GUID) -> Option<String> {
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

const FOLDERID_DESKTOP: GUID = GUID {
    data1: 0xB4BFCC3A,
    data2: 0xDB2C,
    data3: 0x424C,
    data4: [0xB0, 0x29, 0x7F, 0xE9, 0x9A, 0x87, 0xC6, 0x41],
};
const FOLDERID_DOWNLOADS: GUID = GUID {
    data1: 0x374DE290,
    data2: 0x123F,
    data3: 0x4565,
    data4: [0x91, 0x64, 0x39, 0xC4, 0x92, 0x5E, 0x46, 0x7B],
};
const FOLDERID_DOCUMENTS: GUID = GUID {
    data1: 0xFDD39AD0,
    data2: 0x238F,
    data3: 0x46AF,
    data4: [0xAD, 0xB4, 0x6C, 0x85, 0x48, 0x03, 0x69, 0xC7],
};
const FOLDERID_PICTURES: GUID = GUID {
    data1: 0x33E28130,
    data2: 0x4E1E,
    data3: 0x4676,
    data4: [0x83, 0x5A, 0x98, 0x39, 0x5C, 0x3B, 0xC3, 0xBB],
};
const FOLDERID_VIDEOS: GUID = GUID {
    data1: 0x18989B1D,
    data2: 0x99B5,
    data3: 0x455B,
    data4: [0x84, 0x1C, 0xAB, 0x7C, 0x74, 0xE4, 0xDD, 0xFC],
};
const FOLDERID_MUSIC: GUID = GUID {
    data1: 0x4BD8D571,
    data2: 0x6D19,
    data3: 0x48D3,
    data4: [0xBE, 0x97, 0x42, 0x22, 0x20, 0x08, 0x0E, 0x43],
};

pub fn get_sidebar_roots() -> Result<SidebarRoots, AppError> {
    let desktop = known_folder_path(&FOLDERID_DESKTOP).unwrap_or_else(|| {
        PathBuf::from(
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into()),
        )
        .join("Desktop")
        .to_string_lossy()
        .to_string()
    });
    let downloads = known_folder_path(&FOLDERID_DOWNLOADS).unwrap_or_else(|| {
        PathBuf::from(
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into()),
        )
        .join("Downloads")
        .to_string_lossy()
        .to_string()
    });
    let documents = known_folder_path(&FOLDERID_DOCUMENTS).unwrap_or_else(|| {
        PathBuf::from(
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into()),
        )
        .join("Documents")
        .to_string_lossy()
        .to_string()
    });
    let pictures = known_folder_path(&FOLDERID_PICTURES).unwrap_or_else(|| {
        PathBuf::from(
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into()),
        )
        .join("Pictures")
        .to_string_lossy()
        .to_string()
    });
    let videos = known_folder_path(&FOLDERID_VIDEOS).unwrap_or_else(|| {
        PathBuf::from(
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into()),
        )
        .join("Videos")
        .to_string_lossy()
        .to_string()
    });
    let music = known_folder_path(&FOLDERID_MUSIC).unwrap_or_else(|| {
        PathBuf::from(
            std::env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".into()),
        )
        .join("Music")
        .to_string_lossy()
        .to_string()
    });

    Ok(SidebarRoots {
        desktop,
        downloads,
        documents,
        pictures,
        videos,
        music,
        this_pc: "此电脑".into(),
    })
}

// ========================================================================
// 驱动器枚举 FFI
// ========================================================================

#[link(name = "kernel32")]
extern "system" {
    fn GetLogicalDrives() -> u32;
    fn GetDiskFreeSpaceExW(
        lpDirectoryName: *const u16,
        lpFreeBytesAvailable: *mut u64,
        lpTotalNumberOfBytes: *mut u64,
        lpTotalNumberOfFreeBytes: *mut u64,
    ) -> i32;
}

fn get_drive_space(root: &str) -> (u64, u64) {
    let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
    let mut free_avail: u64 = 0;
    let mut total_bytes: u64 = 0;
    let mut total_free: u64 = 0;

    let result = unsafe {
        GetDiskFreeSpaceExW(
            wide.as_ptr(),
            &mut free_avail,
            &mut total_bytes,
            &mut total_free,
        )
    };

    if result != 0 {
        (free_avail, total_bytes)
    } else {
        (0, 0)
    }
}

pub fn get_drives() -> Result<DriveList, AppError> {
    let bits = unsafe { GetLogicalDrives() };
    if bits == 0 {
        return Err(AppError::new(
            ErrorCode::InternalError,
            "无法获取驱动器列表",
        ));
    }

    let mut drives = Vec::new();
    let letters = b'A'..=b'Z';

    for (i, letter) in letters.enumerate() {
        if (bits & (1 << i)) != 0 {
            let root = format!("{}:\\", letter as char);
            let (available, total) = get_drive_space(&root);
            drives.push(DriveInfo {
                name: format!("{}: 本地磁盘", letter as char),
                path: root,
                available_space: available,
                total_space: total,
            });
        }
    }

    Ok(DriveList { drives })
}

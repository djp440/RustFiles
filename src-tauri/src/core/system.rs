use std::ffi::c_void;
use std::path::{Path, PathBuf};

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

// ========================================================================
// 系统集成 — 回收站、默认应用打开、终端打开、属性
// ========================================================================

#[link(name = "shell32")]
extern "system" {
    fn SHFileOperationW(lpFileOp: *mut SHFILEOPSTRUCTW) -> i32;
    fn ShellExecuteW(
        hwnd: isize,
        lpOperation: *const u16,
        lpFile: *const u16,
        lpParameters: *const u16,
        lpDirectory: *const u16,
        nShowCmd: i32,
    ) -> isize;
}

#[link(name = "kernel32")]
extern "system" {
    fn CreateProcessW(
        lpApplicationName: *const u16,
        lpCommandLine: *mut u16,
        lpProcessAttributes: *mut c_void,
        lpThreadAttributes: *mut c_void,
        bInheritHandles: i32,
        dwCreationFlags: u32,
        lpEnvironment: *mut c_void,
        lpCurrentDirectory: *const u16,
        lpStartupInfo: *mut c_void,
        lpProcessInformation: *mut c_void,
    ) -> i32;
    fn CloseHandle(hObject: *mut c_void) -> i32;
}

const FO_DELETE: u32 = 3;
const FOF_ALLOWUNDO: u16 = 0x0040;
const FOF_NOCONFIRMATION: u16 = 0x0010;
const FOF_SILENT: u16 = 0x0004;
const FOF_NOERRORUI: u16 = 0x0400;
const SW_SHOW: i32 = 5;
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

#[allow(non_snake_case)]
#[repr(C)]
struct SHFILEOPSTRUCTW {
    hwnd: isize,
    wFunc: u32,
    pFrom: *const u16,
    pTo: *const u16,
    fFlags: u16,
    fAnyOperationsAborted: i32,
    hNameMappings: *mut c_void,
    lpszProgressTitle: *const u16,
}

#[allow(non_snake_case)]
#[repr(C)]
struct STARTUPINFOW {
    cb: u32,
    lpReserved: *mut c_void,
    lpDesktop: *mut u16,
    lpTitle: *mut u16,
    dwX: u32,
    dwY: u32,
    dwXSize: u32,
    dwYSize: u32,
    dwXCountChars: u32,
    dwYCountChars: u32,
    dwFillAttribute: u32,
    dwFlags: u32,
    wShowWindow: u16,
    cbReserved2: u16,
    lpReserved2: *mut u8,
    hStdInput: *mut c_void,
    hStdOutput: *mut c_void,
    hStdError: *mut c_void,
}

#[allow(non_snake_case)]
#[repr(C)]
struct PROCESS_INFORMATION {
    hProcess: *mut c_void,
    hThread: *mut c_void,
    dwProcessId: u32,
    dwThreadId: u32,
}

fn path_exists(path: &str) -> bool {
    Path::new(path).exists()
}

fn to_wide(path: &str) -> Vec<u16> {
    path.encode_utf16().chain(std::iter::once(0)).collect()
}

fn to_wide_double_null(path: &str) -> Vec<u16> {
    path.encode_utf16()
        .chain(std::iter::once(0))
        .chain(std::iter::once(0))
        .collect()
}

pub fn delete_to_recycle_bin(path: &str) -> Result<(), AppError> {
    if !path_exists(path) {
        return Err(AppError::new(
            ErrorCode::PathNotFound,
            format!("路径不存在: {}", path),
        ));
    }

    let wide = to_wide_double_null(path);

    let mut op = SHFILEOPSTRUCTW {
        hwnd: 0,
        wFunc: FO_DELETE,
        pFrom: wide.as_ptr(),
        pTo: std::ptr::null(),
        fFlags: FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_NOERRORUI,
        fAnyOperationsAborted: 0,
        hNameMappings: std::ptr::null_mut(),
        lpszProgressTitle: std::ptr::null(),
    };

    let result = unsafe { SHFileOperationW(&mut op) };
    if result != 0 {
        return Err(AppError::new(
            ErrorCode::RecycleBinUnavailable,
            format!("回收站操作失败 (Win32 error {})", result),
        ));
    }

    Ok(())
}

pub fn open_with_default_app(path: &str) -> Result<(), AppError> {
    if !path_exists(path) {
        return Err(AppError::new(
            ErrorCode::PathNotFound,
            format!("路径不存在: {}", path),
        ));
    }

    let wide_path = to_wide(path);
    let verb = to_wide("open");

    let result = unsafe {
        ShellExecuteW(
            0,
            verb.as_ptr(),
            wide_path.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOW,
        )
    };

    if result as isize <= 32 {
        return Err(AppError::new(
            ErrorCode::DefaultAppOpenFailed,
            format!("默认应用打开失败 (ShellExecute returned {})", result),
        ));
    }

    Ok(())
}

pub fn open_terminal(path: &str) -> Result<(), AppError> {
    if !path_exists(path) {
        return Err(AppError::new(
            ErrorCode::PathNotFound,
            format!("路径不存在: {}", path),
        ));
    }

    let wide_dir = to_wide(path);

    let mut si = STARTUPINFOW {
        cb: std::mem::size_of::<STARTUPINFOW>() as u32,
        lpReserved: std::ptr::null_mut(),
        lpDesktop: std::ptr::null_mut(),
        lpTitle: std::ptr::null_mut(),
        dwX: 0,
        dwY: 0,
        dwXSize: 0,
        dwYSize: 0,
        dwXCountChars: 0,
        dwYCountChars: 0,
        dwFillAttribute: 0,
        dwFlags: 0,
        wShowWindow: 0,
        cbReserved2: 0,
        lpReserved2: std::ptr::null_mut(),
        hStdInput: std::ptr::null_mut(),
        hStdOutput: std::ptr::null_mut(),
        hStdError: std::ptr::null_mut(),
    };

    let mut pi = PROCESS_INFORMATION {
        hProcess: std::ptr::null_mut(),
        hThread: std::ptr::null_mut(),
        dwProcessId: 0,
        dwThreadId: 0,
    };

    let mut cmd_line: Vec<u16> = "cmd.exe\0".encode_utf16().collect();

    let success = unsafe {
        CreateProcessW(
            std::ptr::null(),
            cmd_line.as_mut_ptr(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            0,
            CREATE_NEW_CONSOLE,
            std::ptr::null_mut(),
            wide_dir.as_ptr(),
            &mut si as *mut _ as *mut c_void,
            &mut pi as *mut _ as *mut c_void,
        )
    };

    if success == 0 {
        return Err(AppError::new(
            ErrorCode::TerminalUnavailable,
            "无法打开终端窗口",
        ));
    }

    unsafe {
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }

    Ok(())
}

pub fn show_properties(path: &str) -> Result<(), AppError> {
    if !path_exists(path) {
        return Err(AppError::new(
            ErrorCode::PathNotFound,
            format!("路径不存在: {}", path),
        ));
    }

    let wide_path = to_wide(path);
    let verb = to_wide("properties");

    let result = unsafe {
        ShellExecuteW(
            0,
            verb.as_ptr(),
            wide_path.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOW,
        )
    };

    if result as isize <= 32 {
        return Err(AppError::new(
            ErrorCode::PropertiesOpenFailed,
            format!("属性窗口打开失败 (ShellExecute returned {})", result),
        ));
    }

    Ok(())
}

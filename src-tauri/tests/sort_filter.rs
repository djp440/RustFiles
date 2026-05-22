use std::path::PathBuf;

use rustfiles::core::types::{FilterKind, SortKey};

fn media_fixture_path() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push(".tmp");
    p.push("fixtures");
    p.push("media-dir");
    p
}

fn small_fixture_path() -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.push(".tmp");
    p.push("fixtures");
    p.push("small-dir");
    p
}

#[test]
fn sort_by_name_ascending_folders_first() {
    let path = small_fixture_path();
    let page = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        true,
    )
    .expect("small-dir should be listable");
    let entries = &page.entries;
    assert!(!entries.is_empty());
    for i in 1..entries.len() {
        let prev = &entries[i - 1];
        let curr = &entries[i];
        if prev.is_folder != curr.is_folder {
            continue;
        }
        assert!(
            prev.name.to_lowercase() <= curr.name.to_lowercase(),
            "name ascending: {} > {}",
            prev.name,
            curr.name
        );
    }
}

#[test]
fn sort_by_name_descending_folders_first() {
    let path = small_fixture_path();
    let page = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        false,
        &FilterKind::All,
        true,
    )
    .expect("small-dir should be listable");
    let entries = &page.entries;
    assert!(!entries.is_empty());
    for i in 1..entries.len() {
        let prev = &entries[i - 1];
        let curr = &entries[i];
        if prev.is_folder != curr.is_folder {
            continue;
        }
        assert!(
            prev.name.to_lowercase() >= curr.name.to_lowercase(),
            "name descending: {} < {}",
            prev.name,
            curr.name
        );
    }
}

#[test]
fn sort_by_modified_preserves_folders_first() {
    let path = small_fixture_path();
    let page = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Modified,
        true,
        &FilterKind::All,
        true,
    )
    .expect("small-dir should be listable");
    let entries = &page.entries;
    assert!(!entries.is_empty());
    let mut found_folder = false;
    for entry in entries {
        if !found_folder && entry.is_folder {
            found_folder = true;
        }
        if found_folder && !entry.is_folder {
            found_folder = false;
        }
    }
    let first_file_idx = entries.iter().position(|e| !e.is_folder).unwrap_or(entries.len());
    for i in 1..first_file_idx {
        assert!(entries[i - 1].is_folder, "folders must come first");
    }
    let folders = entries.iter().filter(|e| e.is_folder).collect::<Vec<_>>();
    if folders.len() >= 2 {
        for i in 1..folders.len() {
            assert!(
                folders[i - 1].modified <= folders[i].modified,
                "folders should be sorted by modified ascending"
            );
        }
    }
    let files = entries.iter().filter(|e| !e.is_folder).collect::<Vec<_>>();
    if files.len() >= 2 {
        for i in 1..files.len() {
            assert!(
                files[i - 1].modified <= files[i].modified,
                "files should be sorted by modified ascending"
            );
        }
    }
}

#[test]
fn sort_by_size_preserves_folders_first() {
    let path = media_fixture_path();
    let page = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Size,
        true,
        &FilterKind::All,
        true,
    )
    .expect("media-dir should be listable");
    let entries = &page.entries;
    assert!(!entries.is_empty());
    let first_file_idx = entries.iter().position(|e| !e.is_folder).unwrap_or(0);
    for i in 0..first_file_idx {
        assert!(entries[i].is_folder, "folders must come first");
    }
    if entries.len() >= 2 {
        for i in 1..entries.len() {
            let prev = &entries[i - 1];
            let curr = &entries[i];
            if prev.is_folder != curr.is_folder {
                continue;
            }
            assert!(
                prev.size <= curr.size,
                "same-type entries should be sorted by size ascending"
            );
        }
    }
}

#[test]
fn sort_by_file_type_preserves_folders_first() {
    let path = media_fixture_path();
    let page = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::FileType,
        true,
        &FilterKind::All,
        true,
    )
    .expect("media-dir should be listable");
    let entries = &page.entries;
    assert!(!entries.is_empty());
    for i in 1..entries.len() {
        let prev = &entries[i - 1];
        let curr = &entries[i];
        if prev.is_folder != curr.is_folder {
            assert!(prev.is_folder && !curr.is_folder, "folders must come first");
            continue;
        }
        let ext_a = prev.name.rsplit('.').next().unwrap_or("");
        let ext_b = curr.name.rsplit('.').next().unwrap_or("");
        assert!(
            ext_a <= ext_b,
            "file type sort: {} ({}) > {} ({})",
            prev.name, ext_a, curr.name, ext_b
        );
    }
}

#[test]
fn filter_kind_videos_returns_only_video_files() {
    let path = media_fixture_path();
    let page = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::Videos,
        false,
    )
    .expect("media-dir should be listable");
    assert_eq!(page.total_count, 4, "media-dir has 4 video files");
    for entry in &page.entries {
        assert!(!entry.is_folder, "videos filter should not return folders");
        let lower = entry.name.to_lowercase();
        assert!(
            lower.ends_with(".mp4")
                || lower.ends_with(".mov")
                || lower.ends_with(".avi")
                || lower.ends_with(".mkv"),
            "unexpected file in videos filter: {}",
            entry.name
        );
    }
}

#[test]
fn filter_kind_images_returns_only_image_files() {
    let path = media_fixture_path();
    let page = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::Images,
        false,
    )
    .expect("media-dir should be listable");
    assert_eq!(page.total_count, 6, "media-dir has 6 image files");
    for entry in &page.entries {
        assert!(!entry.is_folder, "images filter should not return folders");
    }
}

#[test]
fn filter_kind_documents_returns_only_document_files() {
    let path = media_fixture_path();
    let page = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::Documents,
        false,
    )
    .expect("media-dir should be listable");
    assert_eq!(page.total_count, 3, "media-dir has 3 document files");
    for entry in &page.entries {
        assert!(!entry.is_folder, "documents filter should not return folders");
    }
}

#[test]
fn filter_kind_folders_returns_only_folders() {
    let path = small_fixture_path();
    let page = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::Folders,
        true,
    )
    .expect("small-dir should be listable");
    assert!(page.total_count >= 2, "small-dir has at least 2 folders");
    for entry in &page.entries {
        assert!(entry.is_folder, "folders filter should only return folders");
    }
}

#[test]
fn filter_kind_files_returns_only_files() {
    let path = small_fixture_path();
    let page = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::Files,
        true,
    )
    .expect("small-dir should be listable");
    assert!(page.total_count >= 5, "small-dir has at least 5 files");
    for entry in &page.entries {
        assert!(!entry.is_folder, "files filter should only return files");
    }
}

#[test]
fn show_hidden_toggle_affects_count() {
    let path = small_fixture_path();
    let page_with = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        true,
    )
    .expect("small-dir should be listable");
    let page_without = rustfiles::core::fs::list_directory(
        &path.to_string_lossy(),
        &SortKey::Name,
        true,
        &FilterKind::All,
        false,
    )
    .expect("small-dir should be listable");
    assert!(
        page_with.total_count >= page_without.total_count + 1,
        "show_hidden=true should show at least 1 more entry (hidden file), got {} vs {}",
        page_with.total_count,
        page_without.total_count
    );
}

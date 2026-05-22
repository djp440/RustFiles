pub mod commands;
pub mod core;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::list_directory,
            commands::get_sidebar_roots,
            commands::get_drives,
            commands::start_search,
            commands::cancel_task,
            commands::create_folder,
            commands::rename_item,
            commands::delete_to_recycle_bin,
            commands::delete_permanently,
            commands::copy_items,
            commands::move_items,
            commands::create_clipboard_operation,
            commands::paste_clipboard_operation,
            commands::create_drag_operation,
            commands::drop_drag_operation,
            commands::resolve_conflict,
            commands::get_task_status,
            commands::request_thumbnails,
            commands::cancel_thumbnail_requests,
            commands::report_viewport_state,
            commands::report_interaction_state,
            commands::get_settings,
            commands::update_settings,
            commands::open_with_default_app,
            commands::open_terminal,
            commands::show_properties,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

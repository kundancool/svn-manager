pub mod commands;
pub mod config;
pub mod creds;
pub mod deploy;
pub mod error;
pub mod svn;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::detect_svn_binary,
            commands::get_config,
            commands::set_svn_path,
            commands::open_project,
            commands::forget_project,
            commands::wc_status,
            commands::wc_update,
            commands::wc_log,
            commands::wc_diff,
            commands::wc_revision_diff,
            commands::wc_commit,
            commands::wc_blame,
            commands::wc_cleanup,
            commands::wc_lock,
            commands::wc_unlock,
            commands::wc_ignore,
            commands::wc_revert,
            commands::wc_resolve,
            commands::repo_layout,
            commands::repo_browse,
            commands::switch_branch,
            commands::create_copy,
            commands::merge_url,
            commands::rollback_revision,
            commands::checkout_project,
            commands::browse_url,
            commands::save_credential,
            commands::delete_credential,
            commands::save_publish,
            commands::publish_prepare,
            commands::publish_diff,
            commands::publish_push,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

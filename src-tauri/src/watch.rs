//! Filesystem watcher on the open working copy: any change outside .svn
//! emits a `wc-fs-change` event so the UI refreshes status in realtime.

use std::path::PathBuf;
use std::sync::Mutex;

use notify::{RecursiveMode, Watcher};
use tauri::Emitter;

use crate::error::{AppError, AppResult};

#[derive(Default)]
pub struct WatchState(pub Mutex<Option<notify::RecommendedWatcher>>);

#[tauri::command]
pub fn watch_start(
    app: tauri::AppHandle,
    state: tauri::State<'_, WatchState>,
    local_path: String,
) -> AppResult<()> {
    let root = PathBuf::from(&local_path);
    if !root.is_dir() {
        return Err(AppError::Config(format!("not a directory: {local_path}")));
    }

    let handle = app.clone();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if let Ok(event) = res {
            // ignore svn metadata churn; anything else is a real change
            let relevant = event
                .paths
                .iter()
                .any(|p| !p.components().any(|c| c.as_os_str() == ".svn"));
            if relevant {
                let _ = handle.emit("wc-fs-change", ());
            }
        }
    })
    .map_err(|e| AppError::Config(format!("could not create watcher: {e}")))?;

    watcher
        .watch(&root, RecursiveMode::Recursive)
        .map_err(|e| AppError::Config(format!("could not watch {local_path}: {e}")))?;

    // replaces (and drops) any previous watcher
    *state.0.lock().unwrap() = Some(watcher);
    Ok(())
}

#[tauri::command]
pub fn watch_stop(state: tauri::State<'_, WatchState>) -> AppResult<()> {
    *state.0.lock().unwrap() = None;
    Ok(())
}

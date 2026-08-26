use std::path::PathBuf;

use serde::Serialize;
use tauri::Manager;

use crate::config::{host_of_url, AppConfig, ProjectEntry, PublishConfig, RepoKind};
use crate::creds;
use crate::svn::types::RemoteEntry;
use crate::deploy::wporg::{self, DeployPreview};
use crate::error::{AppError, AppResult};
use crate::svn::client::{Credentials, SvnClient};
use crate::svn::detect::{self, SvnBinary};
use crate::svn::types::{ItemStatus, LogEntry, StatusEntry, WcInfo};

fn config_path(app: &tauri::AppHandle) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Config(e.to_string()))?;
    Ok(dir.join("config.json"))
}

fn checkout_cache_dir(app: &tauri::AppHandle, publish: &PublishConfig) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Config(e.to_string()))?;
    let kind = match publish.kind {
        RepoKind::Plugin => "plugins",
        RepoKind::Theme => "themes",
    };
    Ok(dir.join("checkouts").join(kind).join(&publish.slug))
}

async fn get_client(app: &tauri::AppHandle) -> AppResult<SvnClient> {
    let cfg = AppConfig::load(&config_path(app)?)?;
    let bin = detect::detect_svn(cfg.svn_path.map(PathBuf::from))
        .await
        .ok_or(AppError::SvnNotFound)?;
    Ok(SvnClient::new(bin.path))
}

fn load_publish(app: &tauri::AppHandle, local_path: &str) -> AppResult<PublishConfig> {
    let cfg = AppConfig::load(&config_path(app)?)?;
    cfg.find_project(local_path)
        .and_then(|p| p.publish.clone())
        .ok_or_else(|| AppError::Config(format!("publishing not configured for {local_path}")))
}

/// Explicit credentials win; otherwise fall back to the keychain entry for
/// the repository's host.
fn creds_for(
    username: Option<String>,
    password: Option<String>,
    repo_url: &str,
) -> Option<Credentials> {
    match (username, password) {
        (Some(u), Some(p)) => Some(Credentials { username: u, password: p }),
        _ => host_of_url(repo_url).and_then(|h| creds::get(&h)),
    }
}

/// Detected wp.org identity of a working copy, when its URL points there.
#[derive(Serialize)]
pub struct WporgDetection {
    pub slug: String,
    pub kind: RepoKind,
}

#[derive(Serialize)]
pub struct OpenedProject {
    pub entry: ProjectEntry,
    /// Present when local_path is an svn working copy (of any repository).
    pub wc: Option<WcInfo>,
    /// Present when the working copy points at wp.org.
    pub wporg: Option<WporgDetection>,
}

#[tauri::command]
pub async fn detect_svn_binary(app: tauri::AppHandle) -> AppResult<Option<SvnBinary>> {
    let cfg = AppConfig::load(&config_path(&app)?)?;
    Ok(detect::detect_svn(cfg.svn_path.map(PathBuf::from)).await)
}

#[tauri::command]
pub async fn get_config(app: tauri::AppHandle) -> AppResult<AppConfig> {
    AppConfig::load(&config_path(&app)?)
}

#[tauri::command]
pub async fn set_svn_path(app: tauri::AppHandle, path: Option<String>) -> AppResult<Option<SvnBinary>> {
    let cfg_path = config_path(&app)?;
    let mut cfg = AppConfig::load(&cfg_path)?;
    cfg.svn_path = path;
    cfg.save(&cfg_path)?;
    Ok(detect::detect_svn(cfg.svn_path.map(PathBuf::from)).await)
}

/// Open a folder: inspect it, remember it in recents, report what it is.
#[tauri::command]
pub async fn open_project(app: tauri::AppHandle, local_path: String) -> AppResult<OpenedProject> {
    let path = PathBuf::from(&local_path);
    if !path.is_dir() {
        return Err(AppError::Config(format!("not a directory: {local_path}")));
    }

    let mut wc = None;
    let mut wporg = None;
    if path.join(".svn").exists() {
        let client = get_client(&app).await?;
        let info = client.info(&path).await?;
        for (host, kind) in [
            ("plugins.svn.wordpress.org", RepoKind::Plugin),
            ("themes.svn.wordpress.org", RepoKind::Theme),
        ] {
            if let Some(after) = info.repo_root.split(host).nth(1) {
                if let Some(slug) = after.split('/').find(|s| !s.is_empty()) {
                    wporg = Some(WporgDetection { slug: slug.to_string(), kind });
                }
            }
        }
        wc = Some(info);
    }

    let cfg_path = config_path(&app)?;
    let mut cfg = AppConfig::load(&cfg_path)?;
    let entry = match cfg.find_project(&local_path) {
        Some(existing) => existing.clone(),
        None => ProjectEntry {
            local_path: local_path.clone(),
            name: path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| local_path.clone()),
            publish: None,
        },
    };
    cfg.upsert_project(entry.clone());
    cfg.save(&cfg_path)?;

    Ok(OpenedProject { entry, wc, wporg })
}

#[tauri::command]
pub async fn forget_project(app: tauri::AppHandle, local_path: String) -> AppResult<AppConfig> {
    let cfg_path = config_path(&app)?;
    let mut cfg = AppConfig::load(&cfg_path)?;
    cfg.projects.retain(|p| p.local_path != local_path);
    cfg.save(&cfg_path)?;
    Ok(cfg)
}

// ---- generic working copy commands ----

#[tauri::command]
pub async fn wc_status(app: tauri::AppHandle, local_path: String) -> AppResult<Vec<StatusEntry>> {
    let client = get_client(&app).await?;
    client.status(&PathBuf::from(local_path)).await
}

#[tauri::command]
pub async fn wc_update(app: tauri::AppHandle, local_path: String) -> AppResult<u64> {
    let client = get_client(&app).await?;
    client.update(&PathBuf::from(local_path)).await
}

#[tauri::command]
pub async fn wc_log(
    app: tauri::AppHandle,
    local_path: String,
    limit: u32,
    before: Option<u64>,
    path: Option<String>,
) -> AppResult<Vec<LogEntry>> {
    let client = get_client(&app).await?;
    client.log(&PathBuf::from(local_path), limit, before, path.as_deref().map(std::path::Path::new)).await
}

#[tauri::command]
pub async fn wc_diff(
    app: tauri::AppHandle,
    local_path: String,
    file: Option<String>,
) -> AppResult<String> {
    let client = get_client(&app).await?;
    client
        .diff(&PathBuf::from(local_path), file.map(PathBuf::from).as_deref())
        .await
}

#[tauri::command]
pub async fn wc_revision_diff(
    app: tauri::AppHandle,
    local_path: String,
    revision: u64,
    path: Option<String>,
) -> AppResult<String> {
    let client = get_client(&app).await?;
    client.diff_revision(&PathBuf::from(local_path), revision, path.as_deref().map(std::path::Path::new)).await
}

/// Commit selected paths. Unversioned selections are `svn add`ed and missing
/// ones `svn delete`d first, so the commit matches what the user checked.
#[tauri::command]
pub async fn wc_commit(
    app: tauri::AppHandle,
    local_path: String,
    message: String,
    paths: Vec<String>,
    username: Option<String>,
    password: Option<String>,
) -> AppResult<Option<u64>> {
    let wc = PathBuf::from(&local_path);
    let client = get_client(&app).await?;
    let status = client.status(&wc).await?;

    let selected: Vec<&StatusEntry> = status
        .iter()
        .filter(|s| paths.iter().any(|p| p == &s.path))
        .collect();
    let to_add: Vec<PathBuf> = selected
        .iter()
        .filter(|s| s.item == ItemStatus::Unversioned)
        .map(|s| PathBuf::from(&s.path))
        .collect();
    let to_delete: Vec<PathBuf> = selected
        .iter()
        .filter(|s| s.item == ItemStatus::Missing)
        .map(|s| PathBuf::from(&s.path))
        .collect();
    client.add(&wc, &to_add).await?;
    client.delete(&wc, &to_delete).await?;

    let commit_paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    let url = client.info(&wc).await?.url;
    client
        .commit(&wc, &message, creds_for(username, password, &url).as_ref(), &commit_paths)
        .await
}

#[tauri::command]
pub async fn wc_blame(
    app: tauri::AppHandle,
    local_path: String,
    path: String,
) -> AppResult<Vec<crate::svn::types::BlameLine>> {
    let client = get_client(&app).await?;
    client.blame(&PathBuf::from(local_path), &PathBuf::from(path)).await
}

#[tauri::command]
pub async fn wc_cleanup(app: tauri::AppHandle, local_path: String) -> AppResult<()> {
    let client = get_client(&app).await?;
    client.cleanup(&PathBuf::from(local_path)).await
}

#[tauri::command]
pub async fn wc_lock(
    app: tauri::AppHandle,
    local_path: String,
    paths: Vec<String>,
    comment: Option<String>,
) -> AppResult<()> {
    let client = get_client(&app).await?;
    client
        .lock(
            &PathBuf::from(local_path),
            &paths.iter().map(PathBuf::from).collect::<Vec<_>>(),
            comment.as_deref(),
        )
        .await
}

#[tauri::command]
pub async fn wc_unlock(
    app: tauri::AppHandle,
    local_path: String,
    paths: Vec<String>,
) -> AppResult<()> {
    let client = get_client(&app).await?;
    client
        .unlock(&PathBuf::from(local_path), &paths.iter().map(PathBuf::from).collect::<Vec<_>>())
        .await
}

#[tauri::command]
pub async fn wc_ignore(app: tauri::AppHandle, local_path: String, path: String) -> AppResult<()> {
    let client = get_client(&app).await?;
    client.add_to_ignore(&PathBuf::from(local_path), &PathBuf::from(path)).await
}

#[tauri::command]
pub async fn wc_revert(
    app: tauri::AppHandle,
    local_path: String,
    paths: Vec<String>,
) -> AppResult<()> {
    let client = get_client(&app).await?;
    client
        .revert(&PathBuf::from(local_path), &paths.iter().map(PathBuf::from).collect::<Vec<_>>())
        .await
}

#[tauri::command]
pub async fn wc_resolve(
    app: tauri::AppHandle,
    local_path: String,
    path: String,
    accept: String,
) -> AppResult<()> {
    let client = get_client(&app).await?;
    client
        .resolve(&PathBuf::from(local_path), &PathBuf::from(path), &accept)
        .await
}

/// Repository shape for a working copy: root, current location, and which
/// standard-layout folders exist — the UI shows only what the repo has.
#[derive(Serialize)]
pub struct RepoLayout {
    pub repo_root: String,
    pub current_url: String,
    pub has_trunk: bool,
    pub has_branches: bool,
    pub has_tags: bool,
}

#[tauri::command]
pub async fn repo_layout(app: tauri::AppHandle, local_path: String) -> AppResult<RepoLayout> {
    let client = get_client(&app).await?;
    let info = client.info(&PathBuf::from(&local_path)).await?;
    let entries = client.list_remote(&info.repo_root, "").await.unwrap_or_default();
    let has = |name: &str| entries.iter().any(|e| e.name == name && e.kind == "dir");
    Ok(RepoLayout {
        has_trunk: has("trunk"),
        has_branches: has("branches"),
        has_tags: has("tags"),
        repo_root: info.repo_root,
        current_url: info.url,
    })
}

/// Browse the repository (relative to its root) from a working copy,
/// optionally pinned to a revision.
#[tauri::command]
pub async fn repo_browse(
    app: tauri::AppHandle,
    local_path: String,
    path: String,
    revision: Option<u64>,
) -> AppResult<Vec<RemoteEntry>> {
    let client = get_client(&app).await?;
    let info = client.info(&PathBuf::from(&local_path)).await?;
    client.list_remote_at(&info.repo_root, &path, revision).await
}

#[tauri::command]
pub async fn switch_branch(
    app: tauri::AppHandle,
    local_path: String,
    url: String,
) -> AppResult<u64> {
    let client = get_client(&app).await?;
    let wc = PathBuf::from(&local_path);
    client.switch_to(&wc, &url).await?;
    Ok(client.info(&wc).await?.revision)
}

/// Create a branch or tag: server-side copy of the WC's current URL.
#[tauri::command]
pub async fn create_copy(
    app: tauri::AppHandle,
    local_path: String,
    destination: String,
    message: String,
    username: Option<String>,
    password: Option<String>,
) -> AppResult<Option<u64>> {
    let client = get_client(&app).await?;
    let info = client.info(&PathBuf::from(&local_path)).await?;
    let dst = format!("{}/{}", info.repo_root.trim_end_matches('/'), destination.trim_matches('/'));
    let auth = creds_for(username, password, &info.repo_root);
    client.copy_remote(&info.url, &dst, &message, auth.as_ref()).await
}

#[tauri::command]
pub async fn merge_url(
    app: tauri::AppHandle,
    local_path: String,
    source_url: String,
) -> AppResult<String> {
    let client = get_client(&app).await?;
    client.merge_from(&PathBuf::from(local_path), &source_url).await
}

/// Reverse-merge a revision into the WC. Leaves the undo as local changes
/// for the user to review and commit.
#[tauri::command]
pub async fn rollback_revision(
    app: tauri::AppHandle,
    local_path: String,
    revision: u64,
) -> AppResult<String> {
    let client = get_client(&app).await?;
    client.merge_revision(&PathBuf::from(local_path), revision, true).await
}

/// Check out a repository URL into a new local folder and open it.
#[tauri::command]
pub async fn checkout_project(
    app: tauri::AppHandle,
    url: String,
    dest: String,
) -> AppResult<OpenedProject> {
    let client = get_client(&app).await?;
    let dest_path = PathBuf::from(&dest);
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    client.checkout(&url, &dest_path).await?;
    open_project(app, dest).await
}

/// Browse any repository URL without a checkout (pre-checkout browsing).
#[tauri::command]
pub async fn browse_url(
    app: tauri::AppHandle,
    url: String,
    path: String,
) -> AppResult<Vec<RemoteEntry>> {
    let client = get_client(&app).await?;
    client.list_remote(&url, &path).await
}

// ---- credentials ----

#[tauri::command]
pub async fn save_credential(
    app: tauri::AppHandle,
    host: String,
    username: String,
    password: String,
) -> AppResult<AppConfig> {
    creds::save(&host, &username, &password)?;
    let cfg_path = config_path(&app)?;
    let mut cfg = AppConfig::load(&cfg_path)?;
    cfg.upsert_credential_meta(&host, &username);
    cfg.save(&cfg_path)?;
    Ok(cfg)
}

#[tauri::command]
pub async fn delete_credential(app: tauri::AppHandle, host: String) -> AppResult<AppConfig> {
    creds::delete(&host)?;
    let cfg_path = config_path(&app)?;
    let mut cfg = AppConfig::load(&cfg_path)?;
    cfg.remove_credential_meta(&host);
    cfg.save(&cfg_path)?;
    Ok(cfg)
}

// ---- wp.org publishing ----

#[tauri::command]
pub async fn save_publish(
    app: tauri::AppHandle,
    local_path: String,
    publish: Option<PublishConfig>,
) -> AppResult<AppConfig> {
    let cfg_path = config_path(&app)?;
    let mut cfg = AppConfig::load(&cfg_path)?;
    let mut entry = cfg
        .find_project(&local_path)
        .cloned()
        .ok_or_else(|| AppError::Config(format!("unknown project: {local_path}")))?;
    entry.publish = publish;
    cfg.upsert_project(entry);
    cfg.save(&cfg_path)?;
    Ok(cfg)
}

/// True when the project's own working copy IS the wp.org repo (direct mode).
async fn is_direct_wporg(app: &tauri::AppHandle, local_path: &str) -> bool {
    let path = PathBuf::from(local_path);
    if !path.join(".svn").exists() {
        return false;
    }
    let Ok(client) = get_client(app).await else { return false };
    let Ok(info) = client.info(&path).await else { return false };
    info.repo_root.contains(".svn.wordpress.org")
}

#[tauri::command]
pub async fn publish_prepare(app: tauri::AppHandle, local_path: String) -> AppResult<DeployPreview> {
    let publish = load_publish(&app, &local_path)?;
    let client = get_client(&app).await?;
    if is_direct_wporg(&app, &local_path).await {
        let wc = PathBuf::from(&local_path);
        client.update(&wc).await?;
        let status = wporg::stage_wc(&client, &wc).await?;
        Ok(DeployPreview { sync: Default::default(), status })
    } else {
        let checkout = checkout_cache_dir(&app, &publish)?;
        wporg::prepare_deploy(
            &client,
            &publish.repo_url(),
            &checkout,
            &PathBuf::from(&local_path),
            &publish.excludes,
        )
        .await
    }
}

#[tauri::command]
pub async fn publish_diff(
    app: tauri::AppHandle,
    local_path: String,
    file: Option<String>,
) -> AppResult<String> {
    let publish = load_publish(&app, &local_path)?;
    let client = get_client(&app).await?;
    let wc = if is_direct_wporg(&app, &local_path).await {
        PathBuf::from(&local_path)
    } else {
        checkout_cache_dir(&app, &publish)?
    };
    client.diff(&wc, file.map(PathBuf::from).as_deref()).await
}

#[tauri::command]
pub async fn publish_push(
    app: tauri::AppHandle,
    local_path: String,
    message: String,
    username: Option<String>,
    password: Option<String>,
) -> AppResult<Option<u64>> {
    let publish = load_publish(&app, &local_path)?;
    let client = get_client(&app).await?;
    let wc = if is_direct_wporg(&app, &local_path).await {
        PathBuf::from(&local_path)
    } else {
        checkout_cache_dir(&app, &publish)?
    };
    wporg::push_deploy(
        &client,
        &wc,
        &message,
        creds_for(username, password, &publish.repo_url()).as_ref(),
    )
    .await
}

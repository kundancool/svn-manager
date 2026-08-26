use std::collections::HashSet;
use std::fs;
use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Serialize;
use walkdir::WalkDir;

use crate::error::{AppError, AppResult};

/// Dirs/files never deployed, regardless of user config.
pub const DEFAULT_EXCLUDES: &[&str] = &[
    ".git",
    ".svn",
    ".hg",
    "node_modules",
    ".DS_Store",
    ".gitignore",
    ".gitattributes",
    "*.zip",
];

#[derive(Debug, Clone, Default, Serialize)]
pub struct SyncReport {
    /// Repo-relative paths written into dst (new or changed content).
    pub copied: Vec<String>,
    /// Repo-relative paths removed from dst because they left src.
    pub deleted: Vec<String>,
}

fn build_globset(extra: &[String]) -> AppResult<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in DEFAULT_EXCLUDES.iter().map(|s| s.to_string()).chain(extra.iter().cloned()) {
        let glob = Glob::new(&pattern)
            .map_err(|e| AppError::Config(format!("invalid exclude pattern '{pattern}': {e}")))?;
        builder.add(glob);
    }
    builder.build().map_err(|e| AppError::Config(e.to_string()))
}

fn is_excluded(rel: &Path, excludes: &GlobSet) -> bool {
    // Match the full relative path and every component, so "node_modules"
    // excludes the tree anywhere it appears.
    if excludes.is_match(rel) {
        return true;
    }
    rel.components().any(|c| excludes.is_match(Path::new(c.as_os_str())))
}

fn relative_files(root: &Path, excludes: &GlobSet) -> AppResult<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    let walker = WalkDir::new(root).into_iter().filter_entry(|e| {
        e.path()
            .strip_prefix(root)
            .map(|rel| rel.as_os_str().is_empty() || !is_excluded(rel, excludes))
            .unwrap_or(true)
    });
    for entry in walker {
        let entry = entry.map_err(|e| AppError::Config(e.to_string()))?;
        if entry.file_type().is_file() {
            files.push(entry.path().strip_prefix(root).unwrap().to_path_buf());
        }
    }
    Ok(files)
}

fn files_identical(a: &Path, b: &Path) -> bool {
    match (fs::metadata(a), fs::metadata(b)) {
        (Ok(ma), Ok(mb)) if ma.len() == mb.len() => match (fs::read(a), fs::read(b)) {
            (Ok(ca), Ok(cb)) => ca == cb,
            _ => false,
        },
        _ => false,
    }
}

/// Mirror `src` into `dst` (an svn checkout's trunk): copy new/changed files,
/// delete files that no longer exist in src. `.svn` metadata and excludes are
/// never copied nor deleted.
pub fn sync_files(src: &Path, dst: &Path, extra_excludes: &[String]) -> AppResult<SyncReport> {
    let excludes = build_globset(extra_excludes)?;
    let src_files = relative_files(src, &excludes)?;
    let dst_files = relative_files(dst, &excludes)?;
    let src_set: HashSet<_> = src_files.iter().cloned().collect();

    let mut report = SyncReport::default();

    for rel in &src_files {
        let from = src.join(rel);
        let to = dst.join(rel);
        if to.exists() && files_identical(&from, &to) {
            continue;
        }
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&from, &to)?;
        report.copied.push(rel.to_string_lossy().into_owned());
    }

    for rel in &dst_files {
        if !src_set.contains(rel) {
            fs::remove_file(dst.join(rel))?;
            report.deleted.push(rel.to_string_lossy().into_owned());
        }
    }

    Ok(report)
}

use std::path::PathBuf;

use crate::svn::client::{Credentials, SvnClient};
use crate::svn::types::{ItemStatus, StatusEntry};

#[derive(Debug, Clone, Serialize)]
pub struct DeployPreview {
    pub sync: SyncReport,
    /// svn status of the trunk checkout after staging — what a push will commit.
    pub status: Vec<StatusEntry>,
}

/// Ensure `checkout_dir` is an up-to-date checkout of `{repo_url}/trunk`,
/// mirror `local_dir` into it, and stage adds/deletes. Nothing is committed.
pub async fn prepare_deploy(
    client: &SvnClient,
    repo_url: &str,
    checkout_dir: &Path,
    local_dir: &Path,
    extra_excludes: &[String],
) -> AppResult<DeployPreview> {
    let trunk_url = format!("{}/trunk", repo_url.trim_end_matches('/'));
    if checkout_dir.join(".svn").exists() {
        client.update(checkout_dir).await?;
    } else {
        fs::create_dir_all(checkout_dir)?;
        client.checkout(&trunk_url, checkout_dir).await?;
    }

    let sync = sync_files(local_dir, checkout_dir, extra_excludes)?;
    let status = stage_wc(client, checkout_dir).await?;
    Ok(DeployPreview { sync, status })
}

/// Stage a working copy for commit: `svn add` unversioned files, `svn delete`
/// missing ones. Returns the fresh status. Used for the hidden trunk checkout
/// (sync mode) and for a user's own wp.org checkout (direct mode).
pub async fn stage_wc(client: &SvnClient, wc: &Path) -> AppResult<Vec<StatusEntry>> {
    let status = client.status(wc).await?;
    let to_add: Vec<PathBuf> = status
        .iter()
        .filter(|s| s.item == ItemStatus::Unversioned)
        .map(|s| PathBuf::from(&s.path))
        .collect();
    let to_delete: Vec<PathBuf> = status
        .iter()
        .filter(|s| s.item == ItemStatus::Missing)
        .map(|s| PathBuf::from(&s.path))
        .collect();
    client.add(wc, &to_add).await?;
    client.delete(wc, &to_delete).await?;
    client.status(wc).await
}

/// Commit the staged trunk checkout. Returns new revision, None if nothing to commit.
pub async fn push_deploy(
    client: &SvnClient,
    checkout_dir: &Path,
    message: &str,
    credentials: Option<&Credentials>,
) -> AppResult<Option<u64>> {
    client.commit(checkout_dir, message, credentials, &[]).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    fn write(p: &Path, content: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, content).unwrap();
    }

    fn setup() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("plugin");
        let dst = dir.path().join("trunk");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();
        (dir, src, dst)
    }

    #[test]
    fn copies_new_and_changed_files() {
        let (_g, src, dst) = setup();
        write(&src.join("plugin.php"), "<?php v2");
        write(&src.join("inc/util.php"), "<?php util");
        write(&dst.join("plugin.php"), "<?php v1");

        let report = sync_files(&src, &dst, &[]).unwrap();

        assert_eq!(fs::read_to_string(dst.join("plugin.php")).unwrap(), "<?php v2");
        assert_eq!(fs::read_to_string(dst.join("inc/util.php")).unwrap(), "<?php util");
        let mut copied = report.copied.clone();
        copied.sort();
        assert_eq!(copied, vec!["inc/util.php".to_string(), "plugin.php".to_string()]);
        assert!(report.deleted.is_empty());
    }

    #[test]
    fn skips_identical_files() {
        let (_g, src, dst) = setup();
        write(&src.join("same.php"), "identical");
        write(&dst.join("same.php"), "identical");

        let report = sync_files(&src, &dst, &[]).unwrap();
        assert!(report.copied.is_empty());
    }

    #[test]
    fn deletes_files_missing_from_source() {
        let (_g, src, dst) = setup();
        write(&src.join("keep.php"), "keep");
        write(&dst.join("keep.php"), "keep");
        write(&dst.join("gone.php"), "old");
        write(&dst.join("sub/gone2.php"), "old");

        let report = sync_files(&src, &dst, &[]).unwrap();

        assert!(!dst.join("gone.php").exists());
        assert!(!dst.join("sub/gone2.php").exists());
        let mut deleted = report.deleted.clone();
        deleted.sort();
        assert_eq!(deleted, vec!["gone.php".to_string(), "sub/gone2.php".to_string()]);
    }

    #[test]
    fn respects_excludes_and_never_touches_dst_svn_dir() {
        let (_g, src, dst) = setup();
        write(&src.join("plugin.php"), "x");
        write(&src.join(".git/HEAD"), "ref");
        write(&src.join("node_modules/pkg/index.js"), "js");
        write(&src.join("build.log"), "log");
        write(&dst.join(".svn/entries"), "svn-meta");

        let report = sync_files(&src, &dst, &["*.log".to_string()]).unwrap();

        assert!(!dst.join(".git").exists());
        assert!(!dst.join("node_modules").exists());
        assert!(!dst.join("build.log").exists());
        // .svn metadata in destination untouched, not reported deleted
        assert_eq!(fs::read_to_string(dst.join(".svn/entries")).unwrap(), "svn-meta");
        assert!(report.deleted.is_empty());
        assert_eq!(report.copied, vec!["plugin.php".to_string()]);
    }

    #[test]
    fn invalid_glob_is_a_clean_error() {
        let (_g, src, dst) = setup();
        assert!(sync_files(&src, &dst, &["[".to_string()]).is_err());
    }

    use crate::svn::client::SvnClient;
    use crate::svn::types::ItemStatus;

    /// Local repo shaped like wp.org: /trunk /tags /assets committed at r1.
    async fn make_wporg_like_repo(dir: &Path) -> (String, SvnClient) {
        let bin = crate::svn::detect::detect_svn(None).await.expect("svn installed");
        let client = SvnClient::new(bin.path);
        let repo = dir.join("repo");
        let out = tokio::process::Command::new("svnadmin")
            .arg("create").arg(&repo).output().await.unwrap();
        assert!(out.status.success());
        let url = format!("file://{}", repo.display());
        let seed = dir.join("seed");
        client.checkout(&url, &seed).await.unwrap();
        for d in ["trunk", "tags", "assets"] {
            fs::create_dir(seed.join(d)).unwrap();
        }
        client
            .add(&seed, &["trunk".into(), "tags".into(), "assets".into()])
            .await
            .unwrap();
        client.commit(&seed, "layout", None, &[]).await.unwrap();
        (url, client)
    }

    #[tokio::test]
    async fn full_deploy_cycle_prepare_then_push() {
        let dir = tempfile::tempdir().unwrap();
        let (url, client) = make_wporg_like_repo(dir.path()).await;

        let plugin = dir.path().join("my-plugin");
        write(&plugin.join("my-plugin.php"), "<?php // v1");
        write(&plugin.join("readme.txt"), "=== My Plugin ===");
        write(&plugin.join(".git/HEAD"), "ref");

        let checkout = dir.path().join("cache-checkout");
        let preview = prepare_deploy(&client, &url, &checkout, &plugin, &[]).await.unwrap();

        // both files staged as added, .git excluded
        let mut items: Vec<_> = preview.status.iter().map(|s| (s.path.clone(), s.item)).collect();
        items.sort();
        assert_eq!(
            items,
            vec![
                ("my-plugin.php".to_string(), ItemStatus::Added),
                ("readme.txt".to_string(), ItemStatus::Added),
            ]
        );

        let rev = push_deploy(&client, &checkout, "Release v1", None).await.unwrap();
        assert_eq!(rev, Some(2));

        // second cycle: modify one, delete one, add one
        write(&plugin.join("my-plugin.php"), "<?php // v2");
        fs::remove_file(plugin.join("readme.txt")).unwrap();
        write(&plugin.join("new.css"), "body{}");

        let preview = prepare_deploy(&client, &url, &checkout, &plugin, &[]).await.unwrap();
        let mut items: Vec<_> = preview.status.iter().map(|s| (s.path.clone(), s.item)).collect();
        items.sort();
        assert_eq!(
            items,
            vec![
                ("my-plugin.php".to_string(), ItemStatus::Modified),
                ("new.css".to_string(), ItemStatus::Added),
                ("readme.txt".to_string(), ItemStatus::Deleted),
            ]
        );

        let rev = push_deploy(&client, &checkout, "Release v2", None).await.unwrap();
        assert_eq!(rev, Some(3));

        // nothing to deploy now: preview empty, push reports no commit
        let preview = prepare_deploy(&client, &url, &checkout, &plugin, &[]).await.unwrap();
        assert!(preview.status.is_empty());
        let rev = push_deploy(&client, &checkout, "noop", None).await.unwrap();
        assert_eq!(rev, None);
    }
}

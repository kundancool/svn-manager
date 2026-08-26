use std::path::{Path, PathBuf};

use crate::error::{AppError, AppResult};
use crate::svn::parser;
use crate::svn::types::{StatusEntry, WcInfo};

/// Extra credentials passed to svn when the cached auth is missing/rejected.
#[derive(Debug, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct SvnClient {
    bin: PathBuf,
}

fn map_command_error(stderr: &str) -> AppError {
    let auth_markers = ["E170001", "E215004", "Authentication failed", "authorization failed"];
    if auth_markers.iter().any(|m| stderr.contains(m)) {
        let realm = stderr
            .lines()
            .find(|l| l.contains("E170001") || l.contains("E215004"))
            .unwrap_or("")
            .trim()
            .to_string();
        return AppError::AuthRequired { realm };
    }
    if stderr.contains("E155007") || stderr.contains("is not a working copy") {
        return AppError::NotAWorkingCopy(stderr.trim().to_string());
    }
    AppError::SvnCommand { stderr: stderr.trim().to_string() }
}

impl SvnClient {
    pub fn new(bin: PathBuf) -> Self {
        Self { bin }
    }

    async fn run(&self, cwd: Option<&Path>, args: &[&str]) -> AppResult<String> {
        self.run_with_stdin(cwd, args, None).await
    }

    async fn run_with_stdin(
        &self,
        cwd: Option<&Path>,
        args: &[&str],
        stdin_data: Option<&str>,
    ) -> AppResult<String> {
        use std::process::Stdio;
        use tokio::io::AsyncWriteExt;

        let mut cmd = tokio::process::Command::new(&self.bin);
        cmd.args(["--non-interactive"])
            .args(args)
            // stable, parseable output regardless of user locale
            .env("LC_ALL", "C")
            .env("LANG", "C")
            .stdin(if stdin_data.is_some() { Stdio::piped() } else { Stdio::null() })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }
        let mut child = cmd.spawn()?;
        if let Some(data) = stdin_data {
            let mut stdin = child.stdin.take().expect("stdin piped");
            stdin.write_all(data.as_bytes()).await?;
            drop(stdin);
        }
        let output = child.wait_with_output().await?;
        if !output.status.success() {
            return Err(map_command_error(&String::from_utf8_lossy(&output.stderr)));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    pub async fn info(&self, path: &Path) -> AppResult<WcInfo> {
        let out = self.run(Some(path), &["info", "--xml"]).await?;
        parser::parse_info_xml(&out)
    }

    pub async fn status(&self, path: &Path) -> AppResult<Vec<StatusEntry>> {
        let out = self.run(Some(path), &["status", "--xml"]).await?;
        parser::parse_status_xml(&out)
    }

    pub async fn checkout(&self, url: &str, dest: &Path) -> AppResult<()> {
        let dest_str = dest.to_string_lossy();
        self.run(None, &["checkout", url, &dest_str]).await?;
        Ok(())
    }

    /// Returns the revision the working copy is now at.
    pub async fn update(&self, path: &Path) -> AppResult<u64> {
        self.run(Some(path), &["update", "--accept", "postpone"]).await?;
        Ok(self.info(path).await?.revision)
    }

    pub async fn add(&self, wc: &Path, paths: &[PathBuf]) -> AppResult<()> {
        if paths.is_empty() {
            return Ok(());
        }
        let mut args = vec!["add".to_string(), "--parents".to_string()];
        args.extend(paths.iter().map(|p| p.to_string_lossy().into_owned()));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(Some(wc), &refs).await?;
        Ok(())
    }

    pub async fn delete(&self, wc: &Path, paths: &[PathBuf]) -> AppResult<()> {
        if paths.is_empty() {
            return Ok(());
        }
        let mut args = vec!["delete".to_string()];
        args.extend(paths.iter().map(|p| p.to_string_lossy().into_owned()));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(Some(wc), &refs).await?;
        Ok(())
    }

    /// Commit; empty `paths` commits the whole working copy.
    /// Returns the new revision (None when svn reports nothing to commit).
    pub async fn commit(
        &self,
        wc: &Path,
        message: &str,
        credentials: Option<&Credentials>,
        paths: &[PathBuf],
    ) -> AppResult<Option<u64>> {
        let mut args: Vec<String> = vec!["commit".into(), "-m".into(), message.into()];
        if let Some(c) = credentials {
            // password over stdin so it never appears in the process list
            args.extend([
                "--username".into(),
                c.username.clone(),
                "--password-from-stdin".into(),
            ]);
        }
        args.extend(paths.iter().map(|p| p.to_string_lossy().into_owned()));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = self
            .run_with_stdin(Some(wc), &refs, credentials.map(|c| c.password.as_str()))
            .await?;
        Ok(parse_committed_revision(&out))
    }
}

impl SvnClient {
    /// Newest-first history with changed paths. `before` pages: only
    /// revisions strictly older than it are returned. `path` limits the log
    /// to a single file or directory.
    pub async fn log(
        &self,
        wc: &Path,
        limit: u32,
        before: Option<u64>,
        path: Option<&Path>,
    ) -> AppResult<Vec<crate::svn::types::LogEntry>> {
        let limit = limit.to_string();
        let range;
        let target;
        let mut args = vec!["log", "--xml", "-v", "-l", &limit];
        if let Some(rev) = before {
            if rev <= 1 {
                return Ok(Vec::new());
            }
            range = format!("-r{}:1", rev - 1);
            args.push(&range);
        }
        if let Some(p) = path {
            target = p.to_string_lossy().into_owned();
            args.push(&target);
        }
        let out = self.run(Some(wc), &args).await?;
        parser::parse_log_xml(&out)
    }

    /// Annotated file: every line with the revision and author that last
    /// changed it, paired with the file's committed content.
    pub async fn blame(
        &self,
        wc: &Path,
        path: &Path,
    ) -> AppResult<Vec<crate::svn::types::BlameLine>> {
        let p = path.to_string_lossy();
        let meta_xml = self.run(Some(wc), &["blame", "--xml", &p]).await?;
        let meta = parser::parse_blame_xml(&meta_xml)?;
        let content = self.run(Some(wc), &["cat", &p]).await?;
        let lines: Vec<&str> = content.split('\n').collect();
        Ok(meta
            .into_iter()
            .map(|(line_number, revision, author, date)| crate::svn::types::BlameLine {
                text: lines
                    .get((line_number - 1) as usize)
                    .map(|s| s.trim_end_matches('\r').to_string())
                    .unwrap_or_default(),
                line_number,
                revision,
                author,
                date,
            })
            .collect())
    }

    /// Repair a working copy left locked by an interrupted operation.
    pub async fn cleanup(&self, wc: &Path) -> AppResult<()> {
        self.run(Some(wc), &["cleanup"]).await?;
        Ok(())
    }

    /// Take repository locks on files (with an optional lock comment).
    pub async fn lock(&self, wc: &Path, paths: &[PathBuf], comment: Option<&str>) -> AppResult<()> {
        if paths.is_empty() {
            return Ok(());
        }
        let mut args = vec!["lock".to_string()];
        if let Some(c) = comment {
            args.extend(["-m".to_string(), c.to_string()]);
        }
        args.extend(paths.iter().map(|p| p.to_string_lossy().into_owned()));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(Some(wc), &refs).await?;
        Ok(())
    }

    pub async fn unlock(&self, wc: &Path, paths: &[PathBuf]) -> AppResult<()> {
        if paths.is_empty() {
            return Ok(());
        }
        let mut args = vec!["unlock".to_string()];
        args.extend(paths.iter().map(|p| p.to_string_lossy().into_owned()));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(Some(wc), &refs).await?;
        Ok(())
    }

    /// Append a file's name to its parent directory's svn:ignore property.
    pub async fn add_to_ignore(&self, wc: &Path, rel_path: &Path) -> AppResult<()> {
        let name = rel_path
            .file_name()
            .ok_or_else(|| crate::error::AppError::Config("path has no file name".into()))?
            .to_string_lossy()
            .into_owned();
        let dir = rel_path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| ".".to_string());

        // unset property reads as empty; propget errors on some setups → tolerate
        let existing = self
            .run(Some(wc), &["propget", "svn:ignore", &dir])
            .await
            .unwrap_or_default();
        let mut patterns: Vec<String> = existing
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect();
        if !patterns.iter().any(|p| p == &name) {
            patterns.push(name);
        }
        let value = patterns.join("\n");
        self.run(Some(wc), &["propset", "svn:ignore", &value, &dir]).await?;
        Ok(())
    }

    /// Changes introduced by a single revision (`svn diff -c REV`),
    /// optionally scoped to one path.
    pub async fn diff_revision(
        &self,
        wc: &Path,
        revision: u64,
        path: Option<&Path>,
    ) -> AppResult<String> {
        let change = format!("-c{revision}");
        let mut args = vec!["diff".to_string(), "--git".to_string(), change];
        if let Some(p) = path {
            args.push(p.to_string_lossy().into_owned());
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(Some(wc), &refs).await
    }

    /// Discard local changes on the given paths.
    pub async fn revert(&self, wc: &Path, paths: &[PathBuf]) -> AppResult<()> {
        if paths.is_empty() {
            return Ok(());
        }
        let mut args = vec!["revert".to_string()];
        args.extend(paths.iter().map(|p| p.to_string_lossy().into_owned()));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(Some(wc), &refs).await?;
        Ok(())
    }

    /// Resolve a conflicted path. `accept`: working | mine-full | theirs-full.
    pub async fn resolve(&self, wc: &Path, path: &Path, accept: &str) -> AppResult<()> {
        let p = path.to_string_lossy();
        self.run(Some(wc), &["resolve", "--accept", accept, &p]).await?;
        Ok(())
    }

    /// Browse a repository without a working copy: `svn ls --xml <url>/<path>`.
    pub async fn list_remote(
        &self,
        repo_url: &str,
        path: &str,
    ) -> AppResult<Vec<crate::svn::types::RemoteEntry>> {
        self.list_remote_at(repo_url, path, None).await
    }

    /// Like `list_remote`, pinned to a revision (file tree of any commit).
    pub async fn list_remote_at(
        &self,
        repo_url: &str,
        path: &str,
        revision: Option<u64>,
    ) -> AppResult<Vec<crate::svn::types::RemoteEntry>> {
        let url = if path.is_empty() {
            repo_url.trim_end_matches('/').to_string()
        } else {
            format!("{}/{}", repo_url.trim_end_matches('/'), path.trim_matches('/'))
        };
        let rev;
        let mut args = vec!["ls", "--xml"];
        if let Some(r) = revision {
            rev = format!("-r{r}");
            args.push(&rev);
        }
        args.push(&url);
        let out = self.run(None, &args).await?;
        parser::parse_list_xml(&out)
    }

    /// Server-side copy — how svn creates branches and tags.
    pub async fn copy_remote(
        &self,
        src_url: &str,
        dst_url: &str,
        message: &str,
        credentials: Option<&Credentials>,
    ) -> AppResult<Option<u64>> {
        let mut args: Vec<String> =
            vec!["copy".into(), src_url.into(), dst_url.into(), "-m".into(), message.into()];
        if let Some(c) = credentials {
            args.extend(["--username".into(), c.username.clone(), "--password-from-stdin".into()]);
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let out = self
            .run_with_stdin(None, &refs, credentials.map(|c| c.password.as_str()))
            .await?;
        Ok(parse_committed_revision(&out))
    }

    /// Point the working copy at a different branch/tag URL.
    pub async fn switch_to(&self, wc: &Path, url: &str) -> AppResult<()> {
        self.run(Some(wc), &["switch", "--accept", "postpone", url]).await?;
        Ok(())
    }

    /// Merge everything eligible from `source_url` into the working copy.
    pub async fn merge_from(&self, wc: &Path, source_url: &str) -> AppResult<String> {
        self.run(Some(wc), &["merge", "--accept", "postpone", source_url]).await
    }

    /// Merge a single revision from the WC's own URL; `reverse` undoes it (rollback).
    pub async fn merge_revision(&self, wc: &Path, revision: u64, reverse: bool) -> AppResult<String> {
        let spec = if reverse { format!("-c-{revision}") } else { format!("-c{revision}") };
        self.run(Some(wc), &["merge", "--accept", "postpone", &spec, "."]).await
    }

    /// Unified diff (`--git` format so adds/deletes/props are visible).
    /// `path` limits to one file; None diffs the whole working copy.
    pub async fn diff(&self, wc: &Path, path: Option<&Path>) -> AppResult<String> {
        let mut args = vec!["diff".to_string(), "--git".to_string()];
        if let Some(p) = path {
            args.push(p.to_string_lossy().into_owned());
        }
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.run(Some(wc), &refs).await
    }
}

fn parse_committed_revision(stdout: &str) -> Option<u64> {
    stdout.lines().rev().find_map(|line| {
        let line = line.trim();
        line.strip_prefix("Committed revision ")
            .and_then(|rest| rest.trim_end_matches('.').parse().ok())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::svn::types::ItemStatus;
    use std::path::PathBuf;

    async fn client() -> SvnClient {
        let bin = crate::svn::detect::detect_svn(None).await.expect("svn installed");
        SvnClient::new(bin.path)
    }

    /// svnadmin create + checkout, returns (repo_url, wc_path, _tempdir guard)
    async fn make_repo() -> (String, PathBuf, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        let out = tokio::process::Command::new("svnadmin")
            .arg("create")
            .arg(&repo)
            .output()
            .await
            .unwrap();
        assert!(out.status.success());
        let url = format!("file://{}", repo.display());
        let wc = dir.path().join("wc");
        let c = client().await;
        c.checkout(&url, &wc).await.unwrap();
        (url, wc, dir)
    }

    #[tokio::test]
    async fn info_reports_repo_url() {
        let (url, wc, _guard) = make_repo().await;
        let c = client().await;
        let info = c.info(&wc).await.unwrap();
        assert_eq!(info.url, url);
        assert_eq!(info.revision, 0);
    }

    #[tokio::test]
    async fn info_on_plain_dir_is_not_a_working_copy() {
        let dir = tempfile::tempdir().unwrap();
        let c = client().await;
        match c.info(dir.path()).await {
            Err(crate::error::AppError::NotAWorkingCopy(_)) => {}
            other => panic!("expected NotAWorkingCopy, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn add_commit_status_roundtrip() {
        let (_url, wc, _guard) = make_repo().await;
        let c = client().await;

        std::fs::write(wc.join("a.txt"), "one\n").unwrap();
        let st = c.status(&wc).await.unwrap();
        assert_eq!(st.len(), 1);
        assert_eq!(st[0].item, ItemStatus::Unversioned);

        c.add(&wc, &[PathBuf::from("a.txt")]).await.unwrap();
        let st = c.status(&wc).await.unwrap();
        assert_eq!(st[0].item, ItemStatus::Added);

        let rev = c.commit(&wc, "add a.txt", None, &[]).await.unwrap();
        assert_eq!(rev, Some(1));

        let st = c.status(&wc).await.unwrap();
        assert!(st.is_empty());

        c.delete(&wc, &[PathBuf::from("a.txt")]).await.unwrap();
        let rev = c.commit(&wc, "rm a.txt", None, &[]).await.unwrap();
        assert_eq!(rev, Some(2));
    }

    #[tokio::test]
    async fn update_pulls_changes_from_second_wc() {
        let (url, wc1, _guard) = make_repo().await;
        let c = client().await;
        let wc2 = _guard.path().join("wc2");
        c.checkout(&url, &wc2).await.unwrap();

        std::fs::write(wc1.join("b.txt"), "hi\n").unwrap();
        c.add(&wc1, &[PathBuf::from("b.txt")]).await.unwrap();
        c.commit(&wc1, "add b", None, &[]).await.unwrap();

        let rev = c.update(&wc2).await.unwrap();
        assert_eq!(rev, 1);
        assert!(wc2.join("b.txt").exists());
    }

    #[tokio::test]
    async fn diff_shows_content_changes() {
        let (_url, wc, _guard) = make_repo().await;
        let c = client().await;
        std::fs::write(wc.join("d.txt"), "one\n").unwrap();
        c.add(&wc, &[PathBuf::from("d.txt")]).await.unwrap();
        c.commit(&wc, "add d", None, &[]).await.unwrap();

        std::fs::write(wc.join("d.txt"), "two\n").unwrap();
        let diff = c.diff(&wc, Some(Path::new("d.txt"))).await.unwrap();
        assert!(diff.contains("-one"));
        assert!(diff.contains("+two"));

        // whole-wc diff works too
        let diff_all = c.diff(&wc, None).await.unwrap();
        assert!(diff_all.contains("d.txt"));
    }

    #[tokio::test]
    async fn log_and_revision_diff() {
        let (_url, wc, _guard) = make_repo().await;
        let c = client().await;
        std::fs::write(wc.join("l.txt"), "first\n").unwrap();
        c.add(&wc, &[PathBuf::from("l.txt")]).await.unwrap();
        c.commit(&wc, "first commit", None, &[]).await.unwrap();
        std::fs::write(wc.join("l.txt"), "second\n").unwrap();
        c.commit(&wc, "second commit", None, &[]).await.unwrap();
        c.update(&wc).await.unwrap();

        let entries = c.log(&wc, 10, None, None).await.unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].revision, 2);
        assert_eq!(entries[0].message, "second commit");
        assert_eq!(entries[0].paths[0].action, "M");

        // paging: strictly older than r2 → just r1
        let older = c.log(&wc, 10, Some(2), None).await.unwrap();
        assert_eq!(older.len(), 1);
        assert_eq!(older[0].revision, 1);

        // nothing older than r1
        let none = c.log(&wc, 10, Some(1), None).await.unwrap();
        assert!(none.is_empty());

        let diff = c.diff_revision(&wc, 2, None).await.unwrap();
        assert!(diff.contains("-first"));
        assert!(diff.contains("+second"));

        // path-scoped revision diff: unrelated path yields nothing
        let scoped = c.diff_revision(&wc, 2, Some(Path::new("l.txt"))).await.unwrap();
        assert!(scoped.contains("+second"));
        std::fs::write(wc.join("other.txt"), "x\n").unwrap();
        c.add(&wc, &[PathBuf::from("other.txt")]).await.unwrap();
        c.commit(&wc, "other", None, &[]).await.unwrap();
        let unrelated = c.diff_revision(&wc, 3, Some(Path::new("l.txt"))).await.unwrap();
        assert!(unrelated.trim().is_empty());
    }

    #[tokio::test]
    async fn remote_list_respects_revision() {
        let (url, wc, _guard) = make_repo().await;
        let c = client().await;
        std::fs::write(wc.join("first.txt"), "1\n").unwrap();
        c.add(&wc, &[PathBuf::from("first.txt")]).await.unwrap();
        c.commit(&wc, "r1", None, &[]).await.unwrap();
        std::fs::write(wc.join("second.txt"), "2\n").unwrap();
        c.add(&wc, &[PathBuf::from("second.txt")]).await.unwrap();
        c.commit(&wc, "r2", None, &[]).await.unwrap();

        let at_r1 = c.list_remote_at(&url, "", Some(1)).await.unwrap();
        assert_eq!(at_r1.len(), 1);
        assert_eq!(at_r1[0].name, "first.txt");
        let at_r2 = c.list_remote_at(&url, "", Some(2)).await.unwrap();
        assert_eq!(at_r2.len(), 2);
    }

    #[tokio::test]
    async fn revert_restores_modified_and_selected_files() {
        let (_url, wc, _guard) = make_repo().await;
        let c = client().await;
        std::fs::write(wc.join("r.txt"), "orig\n").unwrap();
        c.add(&wc, &[PathBuf::from("r.txt")]).await.unwrap();
        c.commit(&wc, "add r", None, &[]).await.unwrap();

        std::fs::write(wc.join("r.txt"), "changed\n").unwrap();
        c.revert(&wc, &[PathBuf::from("r.txt")]).await.unwrap();
        assert_eq!(std::fs::read_to_string(wc.join("r.txt")).unwrap(), "orig\n");
        assert!(c.status(&wc).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn resolve_conflict_with_theirs() {
        let (url, wc1, _guard) = make_repo().await;
        let c = client().await;
        let wc2 = _guard.path().join("wc2");
        c.checkout(&url, &wc2).await.unwrap();

        std::fs::write(wc1.join("c.txt"), "base\n").unwrap();
        c.add(&wc1, &[PathBuf::from("c.txt")]).await.unwrap();
        c.commit(&wc1, "base", None, &[]).await.unwrap();
        c.update(&wc2).await.unwrap();

        std::fs::write(wc1.join("c.txt"), "theirs\n").unwrap();
        c.commit(&wc1, "theirs change", None, &[]).await.unwrap();
        std::fs::write(wc2.join("c.txt"), "mine\n").unwrap();
        c.update(&wc2).await.unwrap(); // --accept postpone → conflict

        let st = c.status(&wc2).await.unwrap();
        assert!(st.iter().any(|s| s.item == ItemStatus::Conflicted));

        c.resolve(&wc2, &PathBuf::from("c.txt"), "theirs-full").await.unwrap();
        assert_eq!(std::fs::read_to_string(wc2.join("c.txt")).unwrap(), "theirs\n");
        assert!(c.status(&wc2).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn remote_list_branch_switch_and_rollback() {
        let (url, seed, _guard) = make_repo().await;
        let c = client().await;
        // standard layout
        for d in ["trunk", "branches", "tags"] {
            std::fs::create_dir(seed.join(d)).unwrap();
        }
        std::fs::write(seed.join("trunk/f.txt"), "v1\n").unwrap();
        c.add(&seed, &["trunk".into(), "branches".into(), "tags".into()]).await.unwrap();
        c.commit(&seed, "layout", None, &[]).await.unwrap(); // r1

        // repo browser: list root
        let entries = c.list_remote(&url, "").await.unwrap();
        let names: Vec<_> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["branches", "tags", "trunk"]);
        assert!(entries.iter().all(|e| e.kind == "dir"));

        // create branch remotely
        c.copy_remote(
            &format!("{url}/trunk"),
            &format!("{url}/branches/feature-x"),
            "branch feature-x",
            None,
        )
        .await
        .unwrap(); // r2
        let branches = c.list_remote(&url, "branches").await.unwrap();
        assert_eq!(branches[0].name, "feature-x");

        // switch a trunk checkout to the branch
        let wc = _guard.path().join("wc-switch");
        c.checkout(&format!("{url}/trunk"), &wc).await.unwrap();
        c.switch_to(&wc, &format!("{url}/branches/feature-x")).await.unwrap();
        let info = c.info(&wc).await.unwrap();
        assert!(info.url.ends_with("/branches/feature-x"));

        // rollback: commit v2 on branch then reverse-merge it
        std::fs::write(wc.join("f.txt"), "v2\n").unwrap();
        let rev = c.commit(&wc, "v2", None, &[]).await.unwrap().unwrap(); // r3
        c.update(&wc).await.unwrap();
        c.merge_revision(&wc, rev, true).await.unwrap();
        assert_eq!(std::fs::read_to_string(wc.join("f.txt")).unwrap(), "v1\n");
        let st = c.status(&wc).await.unwrap();
        assert!(st.iter().any(|s| s.item == ItemStatus::Modified));
    }

    #[tokio::test]
    async fn merge_pulls_branch_change_into_trunk_wc() {
        let (url, seed, _guard) = make_repo().await;
        let c = client().await;
        std::fs::create_dir(seed.join("trunk")).unwrap();
        std::fs::create_dir(seed.join("branches")).unwrap();
        std::fs::write(seed.join("trunk/m.txt"), "base\n").unwrap();
        c.add(&seed, &["trunk".into(), "branches".into()]).await.unwrap();
        c.commit(&seed, "layout", None, &[]).await.unwrap();
        c.copy_remote(&format!("{url}/trunk"), &format!("{url}/branches/b1"), "b1", None)
            .await
            .unwrap();

        let bwc = _guard.path().join("bwc");
        c.checkout(&format!("{url}/branches/b1"), &bwc).await.unwrap();
        std::fs::write(bwc.join("m.txt"), "branch work\n").unwrap();
        c.commit(&bwc, "branch work", None, &[]).await.unwrap();

        let twc = _guard.path().join("twc");
        c.checkout(&format!("{url}/trunk"), &twc).await.unwrap();
        c.merge_from(&twc, &format!("{url}/branches/b1")).await.unwrap();
        assert_eq!(std::fs::read_to_string(twc.join("m.txt")).unwrap(), "branch work\n");
    }

    #[tokio::test]
    async fn blame_annotates_lines_with_revisions() {
        let (_url, wc, _guard) = make_repo().await;
        let c = client().await;
        std::fs::write(wc.join("b.txt"), "alpha\nbeta\n").unwrap();
        c.add(&wc, &[PathBuf::from("b.txt")]).await.unwrap();
        c.commit(&wc, "r1 lines", None, &[]).await.unwrap();
        std::fs::write(wc.join("b.txt"), "alpha\nbeta\ngamma\n").unwrap();
        c.commit(&wc, "r2 adds gamma", None, &[]).await.unwrap();
        c.update(&wc).await.unwrap();

        let lines = c.blame(&wc, Path::new("b.txt")).await.unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].text, "alpha");
        assert_eq!(lines[0].revision, Some(1));
        assert_eq!(lines[2].text, "gamma");
        assert_eq!(lines[2].revision, Some(2));
        assert_eq!(lines[2].author.as_deref(), Some("kundan"));
        assert_eq!(lines[2].line_number, 3);
    }

    #[tokio::test]
    async fn cleanup_runs_on_working_copy() {
        let (_url, wc, _guard) = make_repo().await;
        let c = client().await;
        c.cleanup(&wc).await.unwrap();
    }

    #[tokio::test]
    async fn lock_and_unlock_file() {
        let (_url, wc, _guard) = make_repo().await;
        let c = client().await;
        std::fs::write(wc.join("l.txt"), "x\n").unwrap();
        c.add(&wc, &[PathBuf::from("l.txt")]).await.unwrap();
        c.commit(&wc, "add", None, &[]).await.unwrap();

        c.lock(&wc, &[PathBuf::from("l.txt")], None).await.unwrap();
        let st = c.status(&wc).await.unwrap();
        // locked-but-unmodified file appears in status with a lock token
        assert!(st.iter().any(|s| s.path == "l.txt" && s.has_lock));

        c.unlock(&wc, &[PathBuf::from("l.txt")]).await.unwrap();
        let st = c.status(&wc).await.unwrap();
        assert!(!st.iter().any(|s| s.path == "l.txt" && s.has_lock));
    }

    #[tokio::test]
    async fn add_to_ignore_hides_unversioned_file() {
        let (_url, wc, _guard) = make_repo().await;
        let c = client().await;
        std::fs::write(wc.join("debug.log"), "x\n").unwrap();
        std::fs::write(wc.join("keep.txt"), "x\n").unwrap();

        c.add_to_ignore(&wc, Path::new("debug.log")).await.unwrap();
        let st = c.status(&wc).await.unwrap();
        assert!(!st.iter().any(|s| s.path == "debug.log"));
        // the dir gets a property modification; keep.txt still unversioned
        assert!(st.iter().any(|s| s.path == "keep.txt"));

        // appending keeps existing patterns
        std::fs::write(wc.join("second.log"), "x\n").unwrap();
        c.add_to_ignore(&wc, Path::new("second.log")).await.unwrap();
        let st = c.status(&wc).await.unwrap();
        assert!(!st.iter().any(|s| s.path == "second.log"));
        assert!(!st.iter().any(|s| s.path == "debug.log"));
    }

    #[tokio::test]
    async fn log_can_target_a_single_path() {
        let (_url, wc, _guard) = make_repo().await;
        let c = client().await;
        std::fs::write(wc.join("one.txt"), "1\n").unwrap();
        c.add(&wc, &[PathBuf::from("one.txt")]).await.unwrap();
        c.commit(&wc, "one", None, &[]).await.unwrap();
        std::fs::write(wc.join("two.txt"), "2\n").unwrap();
        c.add(&wc, &[PathBuf::from("two.txt")]).await.unwrap();
        c.commit(&wc, "two", None, &[]).await.unwrap();
        c.update(&wc).await.unwrap();

        let all = c.log(&wc, 10, None, None).await.unwrap();
        assert_eq!(all.len(), 2);
        let only_one = c.log(&wc, 10, None, Some(Path::new("one.txt"))).await.unwrap();
        assert_eq!(only_one.len(), 1);
        assert_eq!(only_one[0].message, "one");
    }

    #[test]
    fn maps_auth_error_from_stderr() {
        let stderr = "svn: E170001: Authentication failed";
        match map_command_error(stderr) {
            crate::error::AppError::AuthRequired { .. } => {}
            other => panic!("expected AuthRequired, got {other:?}"),
        }
        let stderr = "svn: E215004: No more credentials or we tried too many times.";
        assert!(matches!(
            map_command_error(stderr),
            crate::error::AppError::AuthRequired { .. }
        ));
        assert!(matches!(
            map_command_error("svn: E155007: not a working copy"),
            crate::error::AppError::NotAWorkingCopy(_)
        ));
        assert!(matches!(
            map_command_error("some random failure"),
            crate::error::AppError::SvnCommand { .. }
        ));
    }
}

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RepoKind {
    Plugin,
    Theme,
}

/// Optional wp.org publishing setup attached to a project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishConfig {
    pub slug: String,
    pub kind: RepoKind,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub excludes: Vec<String>,
}

impl PublishConfig {
    pub fn repo_url(&self) -> String {
        let host = match self.kind {
            RepoKind::Plugin => "plugins.svn.wordpress.org",
            RepoKind::Theme => "themes.svn.wordpress.org",
        };
        format!("https://{host}/{}", self.slug)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub local_path: String,
    /// Display name, defaults to the folder name.
    pub name: String,
    #[serde(default)]
    pub publish: Option<PublishConfig>,
}

/// Listing metadata for a saved login. The password itself lives in the
/// OS keychain (see creds.rs) — never in this file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMeta {
    pub host: String,
    pub username: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub svn_path: Option<String>,
    /// Most recently used first.
    #[serde(default)]
    pub projects: Vec<ProjectEntry>,
    #[serde(default)]
    pub credentials: Vec<CredentialMeta>,
}

/// Host part of an svn repository URL (http/https/svn/svn+ssh). None for
/// file:// and anything unparseable — those need no credentials.
pub fn host_of_url(url: &str) -> Option<String> {
    let rest = url.split("://").nth(1)?;
    if url.starts_with("file://") {
        return None;
    }
    let authority = rest.split('/').next()?;
    let host = authority.rsplit('@').next()?.split(':').next()?;
    if host.is_empty() || host.contains(' ') {
        None
    } else {
        Some(host.to_string())
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> AppResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = std::fs::read_to_string(path)?;
        serde_json::from_str(&raw).map_err(|e| AppError::Config(format!("bad config: {e}")))
    }

    pub fn save(&self, path: &Path) -> AppResult<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(self)
            .map_err(|e| AppError::Config(e.to_string()))?;
        std::fs::write(path, raw)?;
        Ok(())
    }

    /// Insert or replace (matched by local_path) and move to front of the list.
    pub fn upsert_project(&mut self, project: ProjectEntry) {
        self.projects.retain(|p| p.local_path != project.local_path);
        self.projects.insert(0, project);
    }

    pub fn find_project(&self, local_path: &str) -> Option<&ProjectEntry> {
        self.projects.iter().find(|p| p.local_path == local_path)
    }

    pub fn upsert_credential_meta(&mut self, host: &str, username: &str) {
        self.credentials.retain(|c| c.host != host);
        self.credentials.push(CredentialMeta { host: host.into(), username: username.into() });
    }

    pub fn remove_credential_meta(&mut self, host: &str) {
        self.credentials.retain(|c| c.host != host);
    }

    pub fn credential_for_host(&self, host: &str) -> Option<&CredentialMeta> {
        self.credentials.iter().find(|c| c.host == host)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, name: &str, publish: Option<PublishConfig>) -> ProjectEntry {
        ProjectEntry { local_path: path.into(), name: name.into(), publish }
    }

    #[test]
    fn roundtrips_config_to_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");

        // missing file loads as default
        let mut cfg = AppConfig::load(&path).unwrap();
        assert!(cfg.projects.is_empty());
        assert!(cfg.svn_path.is_none());

        cfg.svn_path = Some("/opt/homebrew/bin/svn".into());
        cfg.upsert_project(entry(
            "/home/me/my-plugin",
            "my-plugin",
            Some(PublishConfig {
                slug: "my-plugin".into(),
                kind: RepoKind::Plugin,
                username: "kundan".into(),
                excludes: vec!["*.zip".into()],
            }),
        ));
        cfg.upsert_project(entry("/home/me/generic-wc", "generic-wc", None));
        cfg.save(&path).unwrap();

        let loaded = AppConfig::load(&path).unwrap();
        assert_eq!(loaded.projects.len(), 2);
        assert_eq!(loaded.projects[0].name, "generic-wc");
        assert!(loaded.projects[0].publish.is_none());
        assert_eq!(loaded.projects[1].publish.as_ref().unwrap().slug, "my-plugin");
        assert_eq!(loaded.svn_path.as_deref(), Some("/opt/homebrew/bin/svn"));
    }

    #[test]
    fn upsert_replaces_existing_project_and_moves_it_first() {
        let mut cfg = AppConfig::default();
        cfg.upsert_project(entry("/p/a", "a", None));
        cfg.upsert_project(entry("/p/b", "b", None));
        cfg.upsert_project(entry("/p/a", "a-renamed", None));
        assert_eq!(cfg.projects.len(), 2);
        assert_eq!(cfg.projects[0].name, "a-renamed");
        assert_eq!(cfg.find_project("/p/b").unwrap().name, "b");
        assert!(cfg.find_project("/p/zzz").is_none());
    }

    #[test]
    fn credential_meta_roundtrips_and_upserts_by_host() {
        let mut cfg = AppConfig::default();
        cfg.upsert_credential_meta("svn.example.com", "alice");
        cfg.upsert_credential_meta("plugins.svn.wordpress.org", "bob");
        cfg.upsert_credential_meta("svn.example.com", "carol"); // replaces alice
        assert_eq!(cfg.credentials.len(), 2);
        assert_eq!(
            cfg.credential_for_host("svn.example.com").map(|c| c.username.as_str()),
            Some("carol")
        );
        cfg.remove_credential_meta("svn.example.com");
        assert!(cfg.credential_for_host("svn.example.com").is_none());
        assert_eq!(cfg.credentials.len(), 1);
    }

    #[test]
    fn host_extraction_from_repo_urls() {
        assert_eq!(host_of_url("https://plugins.svn.wordpress.org/my-plugin"), Some("plugins.svn.wordpress.org".into()));
        assert_eq!(host_of_url("svn://svn.example.com:3690/repo/trunk"), Some("svn.example.com".into()));
        assert_eq!(host_of_url("file:///tmp/repo"), None);
        assert_eq!(host_of_url("not a url"), None);
    }

    #[test]
    fn corrupt_config_is_a_clean_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.json");
        std::fs::write(&path, "{not json").unwrap();
        assert!(AppConfig::load(&path).is_err());
    }

    #[test]
    fn repo_url_by_kind() {
        let p = PublishConfig {
            slug: "my-plugin".into(),
            kind: RepoKind::Plugin,
            username: String::new(),
            excludes: vec![],
        };
        assert_eq!(p.repo_url(), "https://plugins.svn.wordpress.org/my-plugin");
        let t = PublishConfig { kind: RepoKind::Theme, ..p };
        assert_eq!(t.repo_url(), "https://themes.svn.wordpress.org/my-plugin");
    }
}

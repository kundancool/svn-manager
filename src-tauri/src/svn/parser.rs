use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::svn::types::{ItemStatus, LogEntry, LogPath, RemoteEntry, StatusEntry, WcInfo};

// Raw shapes matching `svn --xml` output; public types are flattened from these.

#[derive(Deserialize)]
struct StatusXml {
    #[serde(rename = "target", default)]
    targets: Vec<TargetXml>,
}

#[derive(Deserialize)]
struct TargetXml {
    #[serde(rename = "entry", default)]
    entries: Vec<EntryXml>,
}

#[derive(Deserialize)]
struct EntryXml {
    #[serde(rename = "@path")]
    path: String,
    #[serde(rename = "wc-status")]
    wc_status: WcStatusXml,
}

#[derive(Deserialize)]
struct WcStatusXml {
    #[serde(rename = "@item")]
    item: ItemStatus,
    #[serde(rename = "@props")]
    props: ItemStatus,
    #[serde(rename = "@revision")]
    revision: Option<String>,
    commit: Option<CommitXml>,
    lock: Option<LockXml>,
}

#[derive(Deserialize)]
struct LockXml {
    #[allow(dead_code)]
    token: Option<String>,
}

#[derive(Deserialize)]
struct CommitXml {
    #[serde(rename = "@revision")]
    revision: u64,
    author: Option<String>,
    date: Option<String>,
}

pub fn parse_status_xml(xml: &str) -> AppResult<Vec<StatusEntry>> {
    let parsed: StatusXml =
        quick_xml::de::from_str(xml).map_err(|e| AppError::Parse(e.to_string()))?;
    Ok(parsed
        .targets
        .into_iter()
        .flat_map(|t| t.entries)
        .map(|e| StatusEntry {
            path: e.path,
            item: e.wc_status.item,
            props: e.wc_status.props,
            revision: e.wc_status.revision.and_then(|r| r.parse().ok()),
            last_author: e.wc_status.commit.as_ref().and_then(|c| c.author.clone()),
            last_commit_revision: e.wc_status.commit.as_ref().map(|c| c.revision),
            has_lock: e.wc_status.lock.is_some(),
            last_commit_date: e.wc_status.commit.and_then(|c| c.date),
        })
        .collect())
}

#[derive(Deserialize)]
struct InfoXml {
    entry: InfoEntryXml,
}

#[derive(Deserialize)]
struct InfoEntryXml {
    #[serde(rename = "@kind")]
    kind: String,
    #[serde(rename = "@revision")]
    revision: u64,
    url: String,
    #[serde(rename = "relative-url")]
    relative_url: Option<String>,
    repository: RepositoryXml,
    #[serde(rename = "wc-info")]
    wc_info: Option<WcInfoXml>,
}

#[derive(Deserialize)]
struct RepositoryXml {
    root: String,
}

#[derive(Deserialize)]
struct WcInfoXml {
    #[serde(rename = "wcroot-abspath")]
    wcroot_abspath: Option<String>,
}

pub fn parse_info_xml(xml: &str) -> AppResult<WcInfo> {
    let parsed: InfoXml =
        quick_xml::de::from_str(xml).map_err(|e| AppError::Parse(e.to_string()))?;
    let e = parsed.entry;
    Ok(WcInfo {
        url: e.url,
        repo_root: e.repository.root,
        revision: e.revision,
        kind: e.kind,
        wc_root: e.wc_info.and_then(|w| w.wcroot_abspath),
        relative_url: e.relative_url,
    })
}

#[derive(Deserialize)]
struct LogXml {
    #[serde(rename = "logentry", default)]
    entries: Vec<LogEntryXml>,
}

#[derive(Deserialize)]
struct LogEntryXml {
    #[serde(rename = "@revision")]
    revision: u64,
    author: Option<String>,
    date: String,
    #[serde(default)]
    paths: Option<LogPathsXml>,
    msg: Option<String>,
}

#[derive(Deserialize)]
struct LogPathsXml {
    #[serde(rename = "path", default)]
    paths: Vec<LogPathXml>,
}

#[derive(Deserialize)]
struct LogPathXml {
    #[serde(rename = "@action")]
    action: String,
    #[serde(rename = "@kind")]
    kind: String,
    #[serde(rename = "$text")]
    path: String,
}

#[derive(Deserialize)]
struct BlameXml {
    target: BlameTargetXml,
}

#[derive(Deserialize)]
struct BlameTargetXml {
    #[serde(rename = "entry", default)]
    entries: Vec<BlameEntryXml>,
}

#[derive(Deserialize)]
struct BlameEntryXml {
    #[serde(rename = "@line-number")]
    line_number: u64,
    commit: Option<CommitXml>,
}

/// (line_number, revision, author, date) for one blamed line.
pub type BlameMeta = (u64, Option<u64>, Option<String>, Option<String>);

/// Per-line annotation metadata from `svn blame --xml` (no line text —
/// callers pair it with the file content).
pub fn parse_blame_xml(xml: &str) -> AppResult<Vec<BlameMeta>> {
    let parsed: BlameXml =
        quick_xml::de::from_str(xml).map_err(|e| AppError::Parse(e.to_string()))?;
    Ok(parsed
        .target
        .entries
        .into_iter()
        .map(|e| {
            (
                e.line_number,
                e.commit.as_ref().map(|c| c.revision),
                e.commit.as_ref().and_then(|c| c.author.clone()),
                e.commit.and_then(|c| c.date),
            )
        })
        .collect())
}

pub fn parse_log_xml(xml: &str) -> AppResult<Vec<LogEntry>> {
    let parsed: LogXml =
        quick_xml::de::from_str(xml).map_err(|e| AppError::Parse(e.to_string()))?;
    Ok(parsed
        .entries
        .into_iter()
        .map(|e| LogEntry {
            revision: e.revision,
            author: e.author,
            date: e.date,
            message: e.msg.unwrap_or_default(),
            paths: e
                .paths
                .map(|p| {
                    p.paths
                        .into_iter()
                        .map(|p| LogPath { path: p.path, action: p.action, kind: p.kind })
                        .collect()
                })
                .unwrap_or_default(),
        })
        .collect())
}

#[derive(Deserialize)]
struct ListsXml {
    #[serde(rename = "list", default)]
    lists: Vec<ListXml>,
}

#[derive(Deserialize)]
struct ListXml {
    #[serde(rename = "entry", default)]
    entries: Vec<ListEntryXml>,
}

#[derive(Deserialize)]
struct ListEntryXml {
    #[serde(rename = "@kind")]
    kind: String,
    name: String,
    size: Option<u64>,
    commit: Option<CommitXml>,
}

pub fn parse_list_xml(xml: &str) -> AppResult<Vec<RemoteEntry>> {
    let parsed: ListsXml =
        quick_xml::de::from_str(xml).map_err(|e| AppError::Parse(e.to_string()))?;
    Ok(parsed
        .lists
        .into_iter()
        .flat_map(|l| l.entries)
        .map(|e| RemoteEntry {
            name: e.name,
            kind: e.kind,
            size: e.size,
            revision: e.commit.as_ref().map(|c| c.revision),
            author: e.commit.as_ref().and_then(|c| c.author.clone()),
            date: e.commit.and_then(|c| c.date),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const STATUS_XML: &str = include_str!("../../tests/fixtures/status.xml");
    const INFO_XML: &str = include_str!("../../tests/fixtures/info.xml");

    #[test]
    fn parses_status_entries_with_mixed_states() {
        let entries = parse_status_xml(STATUS_XML).unwrap();
        assert_eq!(entries.len(), 4);

        let added = &entries[0];
        assert_eq!(added.path, "trunk/added.css");
        assert_eq!(added.item, ItemStatus::Added);
        assert_eq!(added.revision, None); // revision="-1" for added files

        let unversioned = &entries[1];
        assert_eq!(unversioned.item, ItemStatus::Unversioned);

        let deleted = &entries[2];
        assert_eq!(deleted.item, ItemStatus::Deleted);
        assert_eq!(deleted.revision, Some(1));
        assert_eq!(deleted.last_author.as_deref(), Some("kundan"));

        let modified = &entries[3];
        assert_eq!(modified.path, "trunk/readme.txt");
        assert_eq!(modified.item, ItemStatus::Modified);
    }

    #[test]
    fn parses_unknown_status_item_without_error() {
        let xml = STATUS_XML.replace("item=\"modified\"", "item=\"some-future-state\"");
        let entries = parse_status_xml(&xml).unwrap();
        assert_eq!(entries[3].item, ItemStatus::Unknown);
    }

    #[test]
    fn parses_empty_status() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<status>
<target
   path=".">
</target>
</status>
"#;
        let entries = parse_status_xml(xml).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn parses_wc_info() {
        let info = parse_info_xml(INFO_XML).unwrap();
        assert!(info.url.starts_with("file:///"));
        assert_eq!(info.url, info.repo_root);
        assert_eq!(info.revision, 1);
        assert_eq!(info.kind, "dir");
        assert!(info.wc_root.as_deref().unwrap().ends_with("/wc"));
    }

    #[test]
    fn info_parse_fails_cleanly_on_garbage() {
        assert!(parse_info_xml("not xml at all").is_err());
    }

    const LOG_XML: &str = include_str!("../../tests/fixtures/log.xml");

    #[test]
    fn parses_log_entries_newest_first() {
        let entries = parse_log_xml(LOG_XML).unwrap();
        assert_eq!(entries.len(), 3);

        let newest = &entries[0];
        assert_eq!(newest.revision, 3);
        assert_eq!(newest.author.as_deref(), Some("kundan"));
        assert_eq!(newest.message, "remove plugin, add extra");
        assert_eq!(newest.date, "2026-08-26T10:12:08.139848Z");
        assert_eq!(newest.paths.len(), 2);
        assert_eq!(newest.paths[0].action, "A");
        assert_eq!(newest.paths[0].path, "/trunk/extra.txt");
        assert_eq!(newest.paths[1].action, "D");

        assert_eq!(entries[2].revision, 1);
        assert_eq!(entries[2].paths.len(), 4);
    }

    #[test]
    fn parses_log_without_verbose_paths() {
        // svn log --xml without -v has no <paths>
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<log>
<logentry revision="7">
<author>a</author>
<date>2026-01-01T00:00:00.000000Z</date>
<msg>hi</msg>
</logentry>
</log>"#;
        let entries = parse_log_xml(xml).unwrap();
        assert_eq!(entries[0].revision, 7);
        assert!(entries[0].paths.is_empty());
    }
}

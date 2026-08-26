use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ItemStatus {
    Added,
    Conflicted,
    Deleted,
    External,
    Ignored,
    Incomplete,
    Merged,
    Missing,
    Modified,
    None,
    Normal,
    Obstructed,
    Replaced,
    Unversioned,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize)]
pub struct StatusEntry {
    pub path: String,
    pub item: ItemStatus,
    pub props: ItemStatus,
    pub revision: Option<u64>,
    pub last_author: Option<String>,
    pub last_commit_revision: Option<u64>,
    pub last_commit_date: Option<String>,
    /// true when the working copy holds a lock token for this path
    pub has_lock: bool,
}

/// One annotated line from `svn blame`.
#[derive(Debug, Clone, Serialize)]
pub struct BlameLine {
    pub line_number: u64,
    /// None for lines not yet committed
    pub revision: Option<u64>,
    pub author: Option<String>,
    pub date: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogPath {
    pub path: String,
    /// A / M / D / R as reported by svn
    pub action: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub revision: u64,
    pub author: Option<String>,
    pub date: String,
    pub message: String,
    pub paths: Vec<LogPath>,
}

/// One entry from `svn ls --xml` — remote repository browsing.
#[derive(Debug, Clone, Serialize)]
pub struct RemoteEntry {
    pub name: String,
    /// "dir" | "file"
    pub kind: String,
    pub size: Option<u64>,
    pub revision: Option<u64>,
    pub author: Option<String>,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WcInfo {
    pub url: String,
    pub repo_root: String,
    pub revision: u64,
    pub kind: String,
    pub wc_root: Option<String>,
    pub relative_url: Option<String>,
}

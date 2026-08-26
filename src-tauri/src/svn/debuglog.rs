//! In-memory ring buffer of every svn invocation — powers the in-app debug
//! console. Passwords never appear here: they travel via stdin, and entries
//! only note that stdin data was supplied.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

const CAPACITY: usize = 200;
const EXCERPT: usize = 4000;

#[derive(Debug, Clone, Serialize)]
pub struct CommandLogEntry {
    /// unix millis
    pub at: u64,
    /// rendered command line (binary + args)
    pub command: String,
    pub cwd: Option<String>,
    pub had_stdin: bool,
    pub exit_code: Option<i32>,
    pub duration_ms: u64,
    pub stdout: String,
    pub stderr: String,
    pub ok: bool,
}

static LOGS: Mutex<VecDeque<CommandLogEntry>> = Mutex::new(VecDeque::new());

fn truncate(s: &str) -> String {
    if s.len() <= EXCERPT {
        s.to_string()
    } else {
        let mut end = EXCERPT;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}… [{} bytes total]", &s[..end], s.len())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn record(
    command: String,
    cwd: Option<String>,
    had_stdin: bool,
    exit_code: Option<i32>,
    duration_ms: u64,
    stdout: &str,
    stderr: &str,
    ok: bool,
) {
    let entry = CommandLogEntry {
        at: SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0),
        command,
        cwd,
        had_stdin,
        exit_code,
        duration_ms,
        stdout: truncate(stdout),
        stderr: truncate(stderr),
        ok,
    };
    let mut logs = LOGS.lock().unwrap();
    if logs.len() >= CAPACITY {
        logs.pop_front();
    }
    logs.push_back(entry);
}

pub fn entries() -> Vec<CommandLogEntry> {
    LOGS.lock().unwrap().iter().cloned().collect()
}

pub fn clear() {
    LOGS.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    // one test — the buffer is a process-wide global, parallel tests would race
    #[test]
    fn records_caps_and_truncates() {
        clear();
        for i in 0..(CAPACITY + 10) {
            record(format!("svn status #{i}"), None, false, Some(0), 5, "out", "", true);
        }
        // other tests may log concurrently — assert tolerant properties only
        let all = entries();
        assert_eq!(all.len(), CAPACITY);
        let has = |n: usize| all.iter().any(|e| e.command == format!("svn status #{n}"));
        assert!(!has(0), "oldest entries should be dropped");
        assert!(has(CAPACITY + 9), "newest entry should be kept");

        clear();
        let big = "x".repeat(EXCERPT * 2);
        record("svn cat big".into(), None, false, Some(0), 5, &big, "", true);
        let e = entries().into_iter().find(|e| e.command == "svn cat big").unwrap();
        assert!(e.stdout.len() < EXCERPT + 100);
        assert!(e.stdout.contains("bytes total"));
        clear();
    }
}

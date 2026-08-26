use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SvnBinary {
    pub path: PathBuf,
    pub version: String,
}

fn parse_version_output(output: &str) -> Option<String> {
    let v = output.trim();
    if v.is_empty() { None } else { Some(v.to_string()) }
}

/// Run `<bin> --version --quiet`; Some(SvnBinary) if it behaves like svn.
pub async fn probe(bin: &Path) -> Option<SvnBinary> {
    let output = tokio::process::Command::new(bin)
        .args(["--version", "--quiet"])
        .output()
        .await
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let version = parse_version_output(&String::from_utf8_lossy(&output.stdout))?;
    Some(SvnBinary { path: bin.to_path_buf(), version })
}

fn candidates() -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    // PATH entries first, expanded to absolute paths so the result is displayable.
    let exe = if cfg!(target_os = "windows") { "svn.exe" } else { "svn" };
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(exe);
            if candidate.is_file() {
                paths.push(candidate);
            }
        }
    }
    #[cfg(target_os = "macos")]
    paths.extend([
        PathBuf::from("/opt/homebrew/bin/svn"),
        PathBuf::from("/usr/local/bin/svn"),
        PathBuf::from("/usr/bin/svn"),
    ]);
    #[cfg(target_os = "linux")]
    paths.extend([
        PathBuf::from("/usr/bin/svn"),
        PathBuf::from("/usr/local/bin/svn"),
        PathBuf::from("/snap/bin/svn"),
    ]);
    #[cfg(target_os = "windows")]
    paths.extend([
        PathBuf::from(r"C:\Program Files\TortoiseSVN\bin\svn.exe"),
        PathBuf::from(r"C:\Program Files\SlikSvn\bin\svn.exe"),
        PathBuf::from(r"C:\Program Files (x86)\Subversion\bin\svn.exe"),
    ]);
    paths
}

/// Find a usable svn binary. `override_path` (user setting) wins when it probes OK.
pub async fn detect_svn(override_path: Option<PathBuf>) -> Option<SvnBinary> {
    if let Some(p) = override_path {
        if let Some(found) = probe(&p).await {
            return Some(found);
        }
    }
    for candidate in candidates() {
        if let Some(found) = probe(&candidate).await {
            return Some(found);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_version_output() {
        assert_eq!(parse_version_output("1.14.5\n"), Some("1.14.5".to_string()));
        assert_eq!(parse_version_output("  1.10.0 "), Some("1.10.0".to_string()));
        assert_eq!(parse_version_output(""), None);
        assert_eq!(parse_version_output("   \n"), None);
    }

    #[tokio::test]
    async fn probe_rejects_nonexistent_binary() {
        let found = probe(std::path::Path::new("/nonexistent/svn-binary")).await;
        assert!(found.is_none());
    }

    // Relies on subversion being installed (true on dev machines and CI).
    #[tokio::test]
    async fn detect_finds_system_svn() {
        let found = detect_svn(None).await.expect("svn should be installed");
        assert!(!found.version.is_empty());
        assert!(found.path.exists());
    }

    #[tokio::test]
    async fn explicit_override_wins() {
        let system = detect_svn(None).await.unwrap();
        let found = detect_svn(Some(system.path.clone())).await.unwrap();
        assert_eq!(found.path, system.path);
    }
}

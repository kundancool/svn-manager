# Changelog

All notable changes to SVN Manager are documented here. Format follows
[Keep a Changelog](https://keepachangelog.com); versions follow semver.

## [Unreleased]

## [0.3.0] - 2026-08-26

### Added
- Realtime working copy watching: file changes appear in Unstaged within
  ~2 seconds (native filesystem watcher, .svn churn ignored, throttled
  status refresh). Manual Refresh remains as fallback.

## [0.2.0] - 2026-08-26

### Added
- Debug console: bug icon in the status bar opens a CLI-style log of every
  svn command with exit codes, timing, and output (passwords never logged —
  they pass to svn over stdin).
- Auth dialogs explain rejected credentials instead of silently reopening.

### Changed
- Dependencies to latest (Vite 8, GitHub Actions v7, tauri-action v1);
  TypeScript stays on 5.x until vue-tsc supports the TypeScript 7 layout.
- Tag releases publish directly instead of creating hidden drafts.

## [0.1.0] - 2026-08-26

First release.

### Added
- **Changes** — Fork-style staged/unstaged working copy view, expandable
  directory tree with file-type icons, per-file diffs, commit box under
  the diff pane, revert, conflict resolution (keep mine / take theirs /
  mark resolved), right-click context menu (diff, blame, file history,
  lock/unlock, add to svn:ignore, revert).
- **History** — commit table with search and per-file filtering; detail tabs
  per commit: Commit (metadata + message), Changes (per-path revision
  diffs), File Tree (lazy repository tree at that revision); one-click
  rollback via reverse merge.
- **Branches & Tags** — listed from the repository (shown only when the repo
  has them): switch, create branch/tag, merge into the working copy.
- **Repository browser** and checkout-from-URL.
- **Blame** with per-revision tinting; working copy cleanup.
- **Publish to WordPress.org** — optional per-project deploy workflow:
  stage local files against wp.org trunk, review, publish in one commit.
- **Credential manager** — logins in the OS keychain (macOS Keychain,
  Windows Credential Manager, Linux secret-service), used automatically;
  passwords passed to svn over stdin, never the command line.
- Desktop chrome: overlay title bar, resizable split panes, status bar;
  light/dark/system themes with macOS liquid-glass vibrancy; diff themes
  (Monokai default, App, GitHub, Dracula, Solarized) with wrap toggle.
- svn CLI detection with guided install; settings and per-project dialogs.

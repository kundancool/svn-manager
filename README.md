# SVN Manager

A fast, open source, cross-platform Subversion client — built with Tauri (Rust) and Vue 3. Small binary, native speed, a clean desktop UI with light, dark, and system themes (with liquid-glass vibrancy on macOS).

A free alternative to SmartSVN, focused on the workflows people actually use.

## Features

- **Changes** — working copy status with A/M/D/C badges, per-file colored diffs, select files, commit, revert, one-click conflict resolution (keep mine / take theirs / mark resolved).
- **History** — revision log with author/date/message, changed paths, full revision diffs, paging, and **rollback** (reverse-merge any revision, review, commit).
- **Branches & Tags** — listed straight from the repository (shown only when the repo has them): switch, create branches and tags, merge a branch into your working copy.
- **Repository browser** — navigate the remote repo tree without touching your checkout.
- **Check out** any repository URL into a new folder; recent projects one click away.
- **Credential manager** — logins stored in the OS keychain (macOS Keychain / Windows Credential Manager / Linux secret-service), used automatically; passwords go to svn over stdin, never the command line and never into config files.
- **Publish to WordPress.org** — optional per-project deploy workflow for plugin/theme authors: stage your local files against the wp.org `trunk/`, review every change, publish with one commit.

The UI shows only what your repository has — no branches folder, no branches section. Same consistent layout everywhere: sidebar, content, resizable panes, status bar.

## Requirements

The `svn` command line client. SVN Manager detects it on first run and shows install instructions if missing:

- macOS: `brew install subversion`
- Windows: TortoiseSVN (with "command line client tools") or SlikSVN
- Linux: `sudo apt install subversion`

## Development

Stack: [Tauri v2](https://tauri.app) (Rust) + Vue 3 + TypeScript + Pinia + Tailwind CSS v4.

```bash
npm install
npm run tauri dev     # run the app
cd src-tauri && cargo test    # Rust tests (need svn + svnadmin installed)
npm run check         # vue-tsc type checking
npm run tauri build   # production bundles (dmg / msi / AppImage)
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for project layout and guidelines.

## License

[MIT](LICENSE)

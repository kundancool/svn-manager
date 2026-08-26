# Contributing to SVN Manager

Thanks for helping! This project aims to be the best open source Subversion GUI.

## Development setup

Prerequisites: [Rust](https://rustup.rs), Node.js 22+, and Subversion (`svn` + `svnadmin` — the test suite creates real repositories).

```bash
npm install
npm run tauri dev          # run the app with hot reload
```

## Checks to run before a PR

```bash
cd src-tauri && cargo test          # Rust tests (34+, run against real svn repos)
cd src-tauri && cargo clippy        # lints
npm run check                       # vue-tsc type checking
npm run build                       # frontend build
```

## Project layout

- `src-tauri/src/svn/` — svn CLI wrapper: detection, `--xml` parsers, typed async client
- `src-tauri/src/deploy/` — WordPress.org sync/publish engine
- `src-tauri/src/creds.rs` — OS keychain credential storage
- `src-tauri/src/commands.rs` — Tauri command layer (thin; logic lives in the modules above)
- `src/components/` — Vue 3 SPA: `ui/` reusable primitives, `views/`, `dialogs/`
- `src/stores/app.ts` — Pinia app state
- `src/style.css` — design tokens (all colors are semantic tokens; components never hardcode colors)

## Guidelines

- **Tests first** for Rust logic: parsers get fixture tests (capture real `svn --xml` output), client methods get integration tests against a `svnadmin create` repo in a tempdir.
- **UI consistency**: use the shared component classes (`.btn`, `.field`, `.microlabel`) and semantic color tokens (`text-ink`, `bg-surface`, `border-edge`, `text-ok/mod/del/warn`). Both themes must work — never hardcode a color.
- **Security**: passwords go to the OS keychain or over stdin to svn — never into config files, logs, or command-line arguments.
- Keep commits focused; describe the why in the commit body when it isn't obvious.

## Reporting bugs

Use the bug report issue template. Include your OS, `svn --version`, and the exact error text.

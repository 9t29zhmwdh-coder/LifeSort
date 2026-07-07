# Changelog, LifeSort

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [0.1.1] - 2026-07-07

### Fixed
- App crashed on every launch: `main.rs` called `tokio::runtime::Handle::current()` from inside Tauri's synchronous `setup()` closure, which has no active Tokio context; switched to `tauri::async_runtime::block_on()`
- Missing `sqlx` workspace dependency and missing Tauri icon set (caused a `generate_context!` panic)
- CI excluded the `lifesort-tauri` crate from check/clippy/test, so these issues went undetected; CI now covers the full workspace plus a new frontend typecheck/build job
- CSS `@import` ordering issue
- LICENSE copyright line formatting

### Added
- Full English/German UI with a language toggle (English default, German switchable)
- Onboarding sections in README: how the app runs, in-practice summary, uninstall/cleanup steps
- Real EN/DE screenshots of the running app

## [0.1.0] - 2026-06-12

### Added
- Recursive file scanner for arbitrary source directories
- AI classifier via local Ollama model (photos, documents, media recognition)
- Duplicate detection using hash comparison
- Folder structure generator: Photos/People/Places/Events, Documents/Invoices/Contracts/Taxes, Downloads/Installers/Archives, Media/Videos/Audio
- One-click execute with full undo support (SQLite undo log)
- Tauri v2 desktop shell with React/TypeScript frontend

# Architecture — LifeSort

## Overview

LifeSort is a Rust workspace with a Tauri v2 desktop shell and a React/TypeScript frontend.

```
LifeSort/
├── crates/
│   ├── ls-core/          # Core library: scanner, classifier, organizer
│   │   ├── src/
│   │   │   ├── scanner/  # Recursive file system traversal
│   │   │   ├── classifier/ # AI classification via Ollama
│   │   │   ├── organizer/  # Folder structure generator & executor
│   │   │   ├── dedup/    # Duplicate detection (hash-based)
│   │   │   ├── models/   # Shared data types
│   │   │   └── db/       # SQLite storage, undo log
│   │   └── Cargo.toml
│   └── ls-cli/           # Headless CLI interface
│       ├── src/main.rs
│       └── Cargo.toml
├── src-tauri/            # Tauri v2 application shell
│   ├── src/
│   │   ├── main.rs
│   │   ├── state.rs
│   │   └── commands/     # IPC commands exposed to frontend
│   └── Cargo.toml
├── frontend/             # React + TypeScript UI
│   ├── src/
│   │   ├── App.tsx
│   │   ├── stores/       # State management
│   │   └── components/   # UI components
│   └── package.json
└── Cargo.toml            # Workspace root
```

## Data Flow

1. **Scanner** traverses the target directory recursively
2. **Classifier** sends file metadata + content samples to local Ollama model
3. **Organizer** generates a proposed folder structure (Photos/People/Places/Events, Documents/Invoices/Contracts/Taxes, Downloads/Installers/Archives, Media/Videos/Audio)
4. User reviews the plan in the **UI**, confirms or adjusts
5. **Executor** moves files, writes undo log to SQLite
6. **Undo** restores original state from log

## AI Integration

- Uses Ollama HTTP API (localhost:11434)
- Model: configurable (default: llama3)
- No internet connection required

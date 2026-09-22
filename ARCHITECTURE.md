# Architecture: LifeSort

## Overview

LifeSort is a Rust workspace with a Tauri v2 desktop shell and a React/TypeScript frontend.

```
LifeSort/
├── crates/
│   ├── ls-core/          # Core library: scanner, classifier, organizer
│   │   ├── src/
│   │   │   ├── scanner/    # Recursive traversal, skips app packages and the Photos library
│   │   │   ├── imageprep.rs # Photos to a 1024 px JPEG for the model, HEIC via sips on macOS
│   │   │   ├── classifier/ # Rules first, then Ollama; keyword rules for documents
│   │   │   ├── ai/         # Ollama client, prompts, status and MLX fallback
│   │   │   ├── organizer/  # Move proposals, execution without overwriting, undo
│   │   │   ├── dedup/      # Duplicate detection (size, then BLAKE3)
│   │   │   ├── models/     # Shared data types
│   │   │   └── db/         # SQLite: move journal and settings
│   │   ├── examples/       # bench_vision, bench_text: the model measurements
│   │   └── Cargo.toml
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

1. **Scanner** walks the chosen folder. Hidden folders, app packages and every `.photoslibrary` stay out, because moving files out of them breaks the owning app.
2. **Classifier** decides per file:
   - screenshots by EXIF marker, file name or exact screen size, without asking a model;
   - other photos through the vision model, sent as a complete JPEG of at most 1024 px;
   - PDFs with a text layer and plain text through the text model, with keyword rules as fallback;
   - everything else by type.
3. **Organizer** proposes a target per file in the UI language. Same names get `(2)`, `(3)` already in the preview.
4. The user reviews and confirms in the **UI**.
5. **Executor** moves files, never over an existing file, and writes each move to the journal in SQLite.
6. **Undo** moves a file back, also after a restart, and refuses when the original place is taken.

Scan results live in memory for the session. The database holds only the move journal and the settings.

## AI Integration

- Ollama HTTP API, `localhost:11434` by default, URL and both models configurable
- Before a run the app checks that Ollama answers and both models are installed, and says so when not
- Models on Ollama's MLX engine reject structured output; they are asked without it and the JSON is read from the answer
- No internet connection required

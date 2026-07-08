# Roadmap — LifeSort

## v0.1.0 — Initial Release (2026-06-12)
- [x] Recursive file scanner
- [x] AI classifier via Ollama (photos, documents, media)
- [x] Folder structure generator (Photos/People/Places/Events, Documents/Invoices/Contracts/Taxes, Downloads/Installers/Archives, Media/Videos/Audio)
- [x] Duplicate detection (hash-based)
- [x] One-click execute with undo
- [x] Tauri v2 desktop shell

## v0.2.0 — Planned
- [ ] Custom folder rule editor (drag & drop)
- [ ] Batch tagging & rename templates
- [ ] Preview mode (dry-run with diff view)
- [ ] Watch mode: auto-sort incoming files

## v0.3.0 — Planned
- [ ] EXIF-aware photo sorting (date, GPS)
- [ ] Duplicate photo comparison (visual)
- [ ] Plugin system for custom classifiers

## v1.0.0 — Target
- [ ] Windows & Linux support
- [ ] Full test coverage

## Under Consideration
- [ ] Optional intake from [CleanFlow](https://github.com/9t29zhmwdh-coder/CleanFlow): CleanFlow handles one-off cleanup (junk, duplicates, trash), LifeSort handles ongoing archival sorting. A CLI pipeline could feed files CleanFlow keeps but doesn't sort straight into LifeSort's classifier instead of leaving them in place. Not scoped yet.

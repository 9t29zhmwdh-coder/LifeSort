# Roadmap: LifeSort

## v0.1.0, Initial Release (2026-06-12)
- [x] Recursive file scanner
- [x] AI classifier via Ollama (photos, documents, media)
- [x] Folder structure generator (Photos/People/Places/Events, Documents/Invoices/Contracts/Taxes, Downloads/Installers/Archives, Media/Videos/Audio)
- [x] Duplicate detection (hash-based)
- [x] One-click execute with undo
- [x] Tauri v2 desktop shell

## v0.2.0, Planned
- [ ] Custom folder rule editor (drag & drop)
- [ ] Batch tagging & rename templates
- [ ] Preview mode (dry-run with diff view)
- [ ] Watch mode: auto-sort incoming files

## v0.3.0, Planned
- [ ] EXIF-aware photo sorting (date, GPS)
- [ ] Duplicate photo comparison (visual)

## v1.3.0, Released
- [x] Moves never overwrite; undo survives a restart
- [x] HEIC photos on macOS, complete images to the vision model
- [x] iPhone screenshots recognised by pixel size, EXIF marker and name
- [x] Photos library and app packages excluded from every scan
- [x] Duplicates go to the Trash after confirmation
- [x] Model recommendations measured per Mac memory size

## v1.4.0, Released
- [x] Apple Photos mode: videos, screenshots, memes and photos of documents collected in albums, nothing deleted

## Next
- [ ] Burst frames: verify the group on a library that contains bursts
- [ ] OCR for scanned PDFs (today they are classified as unknown)
- [ ] Office documents (.docx, .xlsx)

## v1.0.0, Target
- [ ] Windows & Linux support
- [ ] Full test coverage

## Dual-Licensing Readiness

Assessed 2026-07-11: Community-only, not a Dual-Licensing candidate. LifeSort is a single-user, offline-by-design personal file organizer (photos/documents/downloads), the same shape as CleanFlow in this portfolio. No team, multi-tenant or enterprise dimension exists anywhere on the roadmap; a plugin system was the only feature with any paid-extension potential; it was never implemented and was dropped from the roadmap in 1.3.0. Revisit only if a genuine team/business use case emerges.

## Under Consideration
- [ ] Optional intake from [CleanFlow](https://github.com/9t29zhmwdh-coder/CleanFlow): CleanFlow handles one-off cleanup (junk, duplicates, trash), LifeSort handles ongoing archival sorting. A CLI pipeline could feed files CleanFlow keeps but doesn't sort straight into LifeSort's classifier instead of leaving them in place. Not scoped yet.

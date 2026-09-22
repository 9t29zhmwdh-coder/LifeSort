# Privacy Policy : LifeSort

LifeSort runs **fully offline** on your local machine.

## What data is processed

- File names, paths, and metadata (size, date, type) : for classification and sorting
- File content (photos, PDFs, documents) : analyzed locally for AI classification

## What data leaves your machine

**Nothing.** All AI inference runs via [Ollama](https://ollama.ai) locally.
No file content, metadata, or usage data is ever transmitted to external servers.

## Apple Photos library (macOS)

Only after you allow it in the macOS prompt, LifeSort reads the Photos library: type, size, dimensions, duration, file name and the screenshot and burst markers of each item. For the model it requests small previews; originals stored only in iCloud are not downloaded. The only change it makes to the library is creating albums named "LifeSort: …" and adding items to them. Nothing is deleted or moved. What was read stays in memory and is gone when the window closes. Access can be revoked in System Settings > Privacy & Security > Photos.

## Storage

- A local SQLite database stores the move journal (for undo) and the settings
- Scan and classification results are kept in memory only and are gone when the window closes
- All data stays in your user account (`~/Library/Application Support/ch.raystudio.lifesort` on macOS)

## Third-party services

None. LifeSort does not use any cloud services, analytics, or telemetry, and loads no fonts or scripts from the internet. Its only network connection goes to the Ollama address in the settings, `localhost` by default.

## Changes

This policy may be updated with new releases. Check the CHANGELOG for details.

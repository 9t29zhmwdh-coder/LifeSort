# Changelog, LifeSort

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [1.0.9] - 2026-07-29

### Removed

- `keyring` as a dependency. It was declared in the workspace and pulled into both `ls-core` and the Tauri app, but no file outside those three manifests ever referenced it. LifeSort talks to a local Ollama instance and holds no API key that would need a credential store.
- It could not have worked as one either. `keyring` 3 ships no default features, and the declaration named none, so no platform backend was compiled in. Anyone wiring it up in that state would have got a store that silently persists nothing. Removing it is therefore better than the bump to 4 it was scheduled for: an unused dependency that cannot do its job still enlarges the dependency surface and the SBOM, and still has to be reviewed on every advisory.

---

## [1.0.8] - 2026-07-29

### Changed

- `base64` 0.22 to 0.23, `dirs` 5 to 6, `infer` 0.16 to 0.19 and `lopdf` 0.42 to 0.44. All four cross a major boundary but need no change in this codebase: the parts of their APIs used here did not move. They are grouped because that is exactly what they have in common, and separating them further would suggest a difference that is not there.

---

## [1.0.7] - 2026-07-29

### Changed

- Lock file updates for `serde_json`, `tokio`, `uuid`, `thiserror`, `anyhow` and `regex`. These are the compatible part of a grouped update that also carried eight breaking bumps. Splitting them out means the routine half can land on its own, and each breaking dependency gets reviewed for what it actually changes rather than waved through inside a pull request that reads as routine.

---

## [1.0.6] - 2026-07-29

### Changed

Dependency and workflow updates merged since 1.0.5:

- chore(ci): bump the actions group across 1 directory with 3 updates
- chore(deps): bump the npm group across 1 directory with 4 updates

---

## [1.0.5] - 2026-07-28

### Fixed

- The CodeQL job requested `packages: read`, `actions: read` and `contents: read` at job level, repeating grants the workflow level already provides. OpenSSF Scorecard counts that as excessive token permissions and scores `Token-Permissions` at 0 out of 10 for it. The job now requests only `security-events: write`, which is the one grant that genuinely exceeds the workflow default.

## [1.0.4] - 2026-07-28

### Changed

- CodeQL moved from GitHub's default setup to an advanced setup with a committed `.github/workflows/codeql.yml`. The default setup skips pull requests that touch no code of a given language, so a dependency pull request changing only a lock file reported `skipping` on the required `Analyze (...)` checks forever and could never be merged. The workflow runs on every pull request regardless of what changed. It also uses the `security-extended` query suite, which the default setup does not allow choosing. Required checks are unchanged: verified on `BugRadar` that all eight, the generic `CodeQL` check included, turn green under this setup.
- Dependabot now groups only minor and patch updates per ecosystem; majors arrive as individual pull requests. The previous grouping put React 18 to 19, Tailwind 3 to 4 and similar breaking changes into one pull request together with urgently needed security patches, which made the whole batch unreviewable and unmergeable. Actions stay grouped wholesale. Follows `engineering-standards` v0.11.0.

## [1.0.3] - 2026-07-28

### Security

- `postcss` updated to 8.5.24, closing a high-severity path traversal in the source map auto-loading via `sourceMappingURL` that affects all versions up to and including 8.5.17.

Applied as a normal pull request rather than by merging Dependabot's, because Dependabot pull requests cannot currently pass this repository's required checks: CodeQL runs through GitHub's default setup, which does not trigger on a pull request that only touches a lock file, so its checks report `skipping` and never turn green. Bypassing a required check is not an option per `standards/ci-cd.md` section 7, so the fix takes the route that runs the full pipeline.

## [1.0.2] - 2026-07-28

### Added

- `.github/dependabot.yml`, covering GitHub Actions, the Cargo workspace and the frontend npm packages, with grouped weekly updates. The file was missing, and without it there are no version updates at all: security alerts only fire for disclosed vulnerabilities. Follows `engineering-standards` v0.10.0.

### Fixed

- `frontend/package.json` carried version 0.2.8 while the workspace and `tauri.conf.json` were on 1.0.1, the tagged version. All manifests now agree, so the next bump can touch every file that carries a version, as `release-process.md` section 2 requires.
- `actions/checkout` was pinned to two different SHAs across the workflows. All now use v7.0.1 with the full version in the comment.

## [1.0.1] - 2026-07-20

### Changed

- OpenSSF Scorecard workflow and badge.
- `copilot-instructions.md` for consistent AI-assisted contributions.
- Coverage reporting in CI (cargo-tarpaulin).
- Split the README's security/CI badges onto their own line, separate from the platform/tech/AI badges (they were rendering as a single merged line).

## [1.0.0] - 2026-07-18

First stable release: a real, packaged, installable distribution exists
for macOS, Windows, and Linux (DMG, EXE installer, deb/rpm, AppImage),
the prerequisite for a 1.0 release per this portfolio's own SemVer
discipline.

## [0.2.9] - 2026-07-17

### Changed

- README/README.de: marked Ollama as "(optional, for AI-assisted
  sorting)": core scan/hash/dedup/move already work without it.

## [0.2.8] - 2026-07-13

### Added

- Documented the EN/DE language toggle in README.md/README.de.md; it was already implemented and working but not mentioned.

## [0.2.7] - 2026-07-12

### Fixed

- Removed em-dashes/en-dashes across ARCHITECTURE.md, CONTRIBUTING.md, and several Rust source comments/string literals (Swiss German orthography rule).
- Removed stale scaffold-tool bookkeeping files SKELETON.md and TEMPLATE_NOTES.md.

## [0.2.6] - 2026-07-12

### Security

- Rewrote SECURITY.md to the portfolio's current standard (GitHub Security Advisory reporting, 48h response target, Latest-only supported version; the previous version incorrectly said "0.1.x" was supported).
- Documented a Dependabot-flagged advisory (glib, transitive via Tauri's Linux GTK bindings) as an accepted, time-boxed exception.

## [0.2.5] - 2026-07-12

### Added

- Release workflow (`.github/workflows/release.yml`): builds and attaches macOS (DMG), Windows (NSIS installer), and Linux (AppImage) bundles to a GitHub Release on every tag push. Previously, no release ever had an installer attached.
- README/README.de.md: Download section linking to the latest release's installers.

### Security

- Bumped `vite` (v5 to v8) and `@vitejs/plugin-react` (v4 to v6) together to resolve a Dependabot-flagged advisory (esbuild dev-server request/response exposure). Dev-server only, does not affect the built application. Also added `esbuild` as an explicit dev dependency: Vite 8 no longer bundles it by default, and this project's `vite.config.ts` explicitly requests `minify: 'esbuild'`.

### Fixed

- All GitHub Actions in `ci.yml` pinned to a commit SHA, matching the portfolio's Action Pinning standard.

## [0.2.4] - 2026-07-11

### Fixed

- SemVer correction: v0.1.1 added a genuine new feature (full English/German UI with a language toggle) but was versioned as a patch. Renumbered v0.1.1 through v0.1.4 to v0.2.0 through v0.2.3 (same commits, tags and releases recreated at identical SHAs), per the portfolio's SemVer discipline (patch = fix, minor = feature, major = finished product).
- Removed an eszett (ß) from TEMPLATE_NOTES.md; the project uses Swiss German orthography (ss, not ß).
- Removed em-dashes from TEMPLATE_NOTES.md's file list.

## [0.2.3] - 2026-07-11

### Added

- Documented Dual-Licensing assessment (Community-only) in ROADMAP.md.

### Fixed

- Removed em-dashes from ROADMAP.md and SECURITY.md.

## [0.2.2] - 2026-07-11

### Fixed

- Updated actions/setup-node to its latest major version in CI, since GitHub is deprecating the Node.js 20 runtime and the previous version was being forced onto Node 24 and crashing during post-run cleanup.

## [0.2.1] - 2026-07-10

### Changed

- Moved the "New here? -> beginners guide" callout in README.md above Features (previously only appeared near Requirements)

### Added

- Added the "New here?" beginner guide callout to README.de.md (was missing)

## [0.2.0] - 2026-07-07

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

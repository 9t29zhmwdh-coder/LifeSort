# Changelog, LifeSort

All notable changes to this project are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [1.2.5] - 2026-08-27

### Changed

- **Eigenes App-Icon.** Bis hierher trug jedes Werkzeug des Portfolios dasselbe RayStudio-Logo, byteweise identisch, was im Dock und in der Titelleiste nicht auseinanderzuhalten war. LifeSort bekommt jetzt sein eigenes Zeichen im bestehenden Hausstil: runder Rahmen mit Goldkante, tiefdunkler Grund, die Initialen in einer Didone-Serife, der goldene Strahl mit Funkeln darueber.

  Zwei Fassungen, wie es Apple und Microsoft ebenfalls halten: ab 128 Punkten die feine mit dem Schriftzug, darunter eine ohne. Gesperrte Versalien werden bei 32 Punkten zu einem grauen Streifen und nehmen den Initialen nur den Platz weg, den sie dort brauchen.

  Die Farbwerte stammen aus `RegistrarCheck.png`, nicht aus einer Schaetzung: Grund `#010d22`, Gold von `#a7782f` ueber `#e2c47e` nach `#ca9f4d`. Die SVG-Quellen liegen unter `src-tauri/icons/source/`, damit sich das Zeichen spaeter aendern laesst, ohne es nachbauen zu muessen.

---

## [1.2.4] - 2026-08-27

### Fixed

- **The whole interface had lost its spacing.** A global `* { box-sizing: border-box; margin: 0; padding: 0 }` sat in `index.css` outside any cascade layer. Tailwind 4 puts its utilities into real layers, and an unlayered rule beats a layered one regardless of specificity, so every `p-*`, `px-*`, `py-*`, `gap-*` and `space-y-*` in the application was overridden: 85 uses across 6 files. The product name ran straight into its subtitle, the navigation items had no separation, and the scan button rendered as a bare coloured strip against the window edge.

  Under Tailwind 3 the same CSS was harmless, because those utilities were not layered. The regression therefore arrived with the Tailwind 4 upgrade and was invisible to both the typecheck and the production build. Wrapping the reset in `@layer base` restores the layout the markup has described all along; no markup was touched.

  Found while fixing the same defect in LogLens. A sweep across the other Tauri projects found it here and in StateForge; ClarityDesk, BugRadar, DeviceHealth, LifePlanner, MailLoom and CleanFlow are unaffected.

### Security

- `h2` 0.4.15 to 0.4.19, closing RUSTSEC-2026-0258, unbounded empty DATA frames. The advisory was published on 2026-08-17, after this project's last pipeline run, so it surfaced on the next build rather than through any change here. `h2` arrives transitively through `reqwest` and `hyper`; only that one package moved, everything else in the lockfile is untouched.

---

## [1.2.3] - 2026-08-05

### Changed

- TypeScript 5.9.3 to 7. No source change was needed. The production build runs `tsc` ahead of vite, so the typecheck has to pass for anything to be produced at all, and the output comes out with the same content hashes as before.

---

## [1.2.2] - 2026-08-05

### Changed

- The `glib` advisory GHSA-wrw7-89jp-8q8g is now recorded in `SECURITY.md` and its unreachable versions ignored in `dependabot.yml`, matching the six sibling repositories. It had been dismissed as tolerable risk here, which left it invisible while Dependabot kept attempting an update that cannot succeed: `tauri` 2.11.5 requires `gtk ^0.18`, `gtk` 0.18.2 requires `glib ^0.18`, and cargo rejects 0.20.0 outright. The dismissal has been withdrawn so the entry and the Security tab agree.

---

## [1.2.1] - 2026-08-05

### Added

- A smoke test in CI: the application is built, started, and checked to still be running five seconds later. Until now the pipeline only ever established that the code compiles. A program that builds cleanly and dies on launch would have passed every check and been discovered by whoever downloaded it.
- It runs on Linux and macOS. The Linux job needs `xvfb`, since a GTK window closes immediately without an X server, and that would produce a failure the runner invents rather than one the code has.
- The test also fails on a panic in the output even when the process survives, because a background task that dies quietly leaves the window open and useless.

---

## [1.2.0] - 2026-08-03

### Changed

- `infer` 0.19 to 0.22. The detected MIME type decides which folder a file is sorted into, so this one is checked by tests rather than by a green build: PNG, JPEG, PDF and ZIP still resolve to the same types through magic bytes, and files without a signature still fall back to the extension.
- `recharts` 2 to 3, a real major that this repository actually uses. The dashboard pie chart was rendered in a real DOM under both versions with the same data and produced the same four segments and two labels. The bundle dropped from 596 kB to 553 kB.
- The cargo group: `async-trait` 0.1.89 to 0.1.91, `clap` 4.6.1 to 4.6.5, `tauri-plugin-dialog` 2.7.1 to 2.7.2.
- `vite` 8.1.5 to 8.2.0, `@vitejs/plugin-react` 6.0.4 to 6.0.5 and `postcss` 8.5.24 to 8.5.25.
- `github/codeql-action` 4.37.3 to 4.37.4 and `actions/attest` 4.2.0 to 4.2.1, merged separately and carried by this version.

### Added

- Two tests over MIME detection: four magic-byte signatures through `infer`, and the extension fallback for files without one. A version that reports a different type, or none, would sort files into the wrong folder without any crash or build failure.

---

## [1.1.1] - 2026-08-03

### Fixed

- Corrects a claim in the 1.1.0 entry. It said that leaving `rounded` in place would have halved every corner radius, because version 4 shifted the scale. That is wrong. Measured directly under Tailwind 4.3.3: `rounded` is still 0.25rem and is kept as an alias. The scale did shift, but under the name `rounded-sm`, which now means 0.25rem where it meant 0.125rem before. The dangerous case is source that already used `rounded-sm`, and this repository never did, so the rename changed nothing visually. The migration itself was correct; the reason given for it was not.

---

## [1.1.0] - 2026-08-03

### Changed

- Tailwind CSS 3 to 4. `tailwind.config.ts` is gone; the custom colours and the mono stack are theme variables in the stylesheet now, and PostCSS uses `@tailwindcss/postcss`.
- Three utility renames went through five components: `rounded` to `rounded-sm` six times, `outline-none` to `outline-hidden` three times, `flex-shrink-0` to `shrink-0` five times. The first one matters most, because version 4 shifted the radius scale by one step and the old name now means half as much.
- autoprefixer is no longer a dependency. Built once with it and once without, the stylesheet comes out with the same content hash, so it was contributing nothing.

---

## [1.0.17] - 2026-08-02

### Changed

- React 18 to 19, together with `react-dom` and both type packages. Dependabot had split these across separate pull requests and neither could be merged alone: `@types/react-dom` 18 requires `@types/react` 18, so raising either one left npm unable to resolve the peer dependency. All four move together here.
- No code changes were needed, checked against the list of things React 19 removes rather than assumed: `createRoot` is already in use, and there are no string refs, no `propTypes`, no argument-less `useRef`, no `forwardRef`, no `defaultProps` and no callback refs. Typecheck and production build both clean.

---

## [1.0.16] - 2026-08-02

### Changed

- `notify-debouncer-full` 0.6.0 to 0.7.0, merged since 1.0.15 and carried by this version. The watcher here re-reads on any debounced event rather than filtering by kind, so the Windows defect found in LogLens does not apply.

---

## [1.0.15] - 2026-07-31

### Fixed

- CI checked only macOS while the release builds for macOS, Linux and Windows. The AppImage and the Windows installer went out without ever having been compile-checked, so a fault appearing only on those platforms would have surfaced in a user's download rather than in a pull request. `check` now runs as a matrix over all three. The Linux runner installs the same GTK and WebKit packages the release workflow installs, since Tauri does not build without them.
- The `solo-main-protection` ruleset now requires `Check (ubuntu-latest)`, `(macos-latest)` and `(windows-latest)` instead of the old single `Check`. Renaming a job without moving the required context leaves every later pull request permanently unmergeable while looking green.

---

## [1.0.14] - 2026-07-30

### Changed

- The README opens with what the tool is for instead of what it is built with. Both language versions previously began "AI-powered file organizer", the same phrase CleanFlow used, so a reader looking at both could not tell which one to pick. The repository description is rewritten for the same reason.
- The opening now names the case this tool is **not** for and links to CleanFlow by name. The exclusion is the sentence that saves the wrong reader ten minutes, and between two tools that sound alike it is worth more than any feature list.

---

## [1.0.13] - 2026-07-29

### Security

- The release workflow no longer grants `contents: write` for its whole run. The permission moves to the one job that publishes the release, and everything else runs with `contents: read`. OpenSSF Scorecard scores the Token-Permissions check 0 out of 10 whenever any workflow holds a top-level write permission, regardless of how little of the run needs it, so this single line was what held the check at zero.
### Added

- `frontend/src/vite-env.d.ts`, referencing `vite/client`. Vite has always declared modules for `*.css` and the other asset types it handles, but nothing in this project pulled that declaration in. TypeScript 5 accepts the untyped side-effect import of `index.css` regardless, so the gap stayed invisible; TypeScript 7 rejects it with `TS2882`. The file belongs to Vite's own project scaffold and was simply missing, so this closes an existing hole rather than preparing for a specific upgrade.

---

## [1.0.12] - 2026-07-29

### Changed

- `sqlx` 0.8 to 0.9. This is the last of the eight breaking updates split out of the grouped cargo bump.

### Added

- A runtime test for the database layer. Every call site stayed type-correct across this bump, but a database library can change how migrations are applied or how SQLite values map back into Rust without any of that showing up at compile time. The test opens a fresh database through `db::open`, checks that sqlx recorded applied migrations, and opens it a second time to confirm reopening is a no-op rather than an error, which is what every start after the first one does. It passes against both 0.8 and 0.9, so it measures the behaviour rather than the version.

---

## [1.0.11] - 2026-07-29

### Changed

- `notify` 6 to 8, together with `notify-debouncer-full` 0.3 to 0.6. The two move as a pair: the debouncer wraps a watcher from `notify`, so a version of it that matches `notify` 8 is required and cannot be chosen independently.
- The watcher struct names `RecommendedCache` instead of `FileIdMap`. The cache type `new_debouncer` returns is chosen per platform, so naming one of them concretely compiles only on the platform where that choice happens to match. It built on macOS and failed on the Linux coverage runner with `expected Debouncer<INotifyWatcher, FileIdMap>, found Debouncer<INotifyWatcher, NoCache>`.
- The file watcher registers its path directly on the debouncer. `Debouncer::watcher()` is deprecated in 0.6 and no longer returns a watcher that `watch` can be called on, so `debouncer.watcher().watch(path, ...)` becomes `debouncer.watch(path, ...)`. This is the only source change among the eight breaking updates split out of the grouped cargo bump.

---

## [1.0.10] - 2026-07-29

### Changed

- `reqwest` 0.12 to 0.13, with `rustls` named explicitly in the feature list.

### Security

- The TLS backend changes here, not only the certificate roots. This crate named no TLS feature and lived on the defaults, which meant native-tls under 0.12; in 0.13 `default-tls` points at `rustls`. Naming `rustls` in the manifest makes that switch visible instead of letting it happen silently through a changed default.
- TLS now trusts the operating system's certificate store: the `rustls` feature in 0.13 pulls in `rustls-platform-verifier`. A machine that trusts an internal certificate authority works without extra configuration, and the trust decision moves onto the machine the tool runs on.
- The rustls crypto provider is `aws-lc-rs`, which is what the `rustls` feature selects in 0.13.

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

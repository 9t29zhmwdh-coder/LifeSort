# Skeleton — LifeSort

This file documents the repository structure and CI expectations for contributors.

## Repository Layout

```
LifeSort/
├── crates/ls-core/       # Core library
├── crates/ls-cli/        # CLI interface
├── src-tauri/            # Tauri shell
├── frontend/             # React/TypeScript UI
├── .github/
│   ├── ISSUE_TEMPLATE/
│   │   ├── bug_report.md
│   │   └── feature_request.md
│   └── PULL_REQUEST_TEMPLATE.md
├── ARCHITECTURE.md
├── CHANGELOG.md
├── CODE_OF_CONDUCT.md
├── CONTRIBUTING.md
├── PRIVACY.md
├── ROADMAP.md
├── SECURITY.md
└── SKELETON.md
```

## CI Expectations

- `cargo check --workspace` — must pass
- `cargo test --workspace` — must pass
- `cargo clippy --workspace -- -D warnings` — must pass
- `cargo fmt --all` — must be applied before PR

## Branch Strategy

- `main` — stable, tagged releases
- `dev` — integration branch
- Feature branches: `feat/<name>`

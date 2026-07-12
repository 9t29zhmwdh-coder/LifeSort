# Security Policy

## Reporting a Vulnerability

**Do NOT open a public GitHub issue for security vulnerabilities.**

Instead, report it via [GitHub Security Advisory](https://github.com/9t29zhmwdh-coder/LifeSort/security/advisories/new) or contact the maintainer via the GitHub profile.

Include:
- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

A response within **48 hours** is the target, and I will work to resolve the issue promptly.

## Scope

This project runs fully locally; no data is sent to external servers. Network access (if any) is limited to local network resources only.

## Known Accepted Exceptions

- **glib (RUSTSEC, medium): unsoundness in `Iterator`/`DoubleEndedIterator` impls for `glib::VariantStrIter`**, present as a transitive dependency of Tauri's Linux tray/menu integration via `gtk 0.18.2`/`atk 0.18.2`. `gtk 0.18.2` pins `glib` to `^0.18`; the fixed `glib 0.20.0` requires a `gtk`/Tauri major-version bump, not an isolated patch. This crate is only linked on Linux builds and the unsound pattern is not reachable from this application's own code. Accepted as of 2026-07-12; revisit when Tauri's own dependency tree moves past `gtk 0.18`.

## Supported Versions

| Version | Supported |
|---------|-----------|
| Latest  | ✅ Yes    |
| Older   | ❌ No     |

Security fixes are only applied to the latest release.

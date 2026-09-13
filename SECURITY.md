# Security Policy

## Supported versions

Only the latest GitHub release is supported with security fixes.

| Version | Supported |
|---------|-----------|
| latest release | ✅ |
| older releases | ❌ |

## Reporting a vulnerability

Please **do not** open a public issue for security problems. Instead, use
[GitHub Security Advisories](https://github.com/JustRelaXX/Keepr/security/advisories/new)
(private report) so we can fix the issue before disclosure.

Include: affected version, platform, steps to reproduce, and impact. We aim to
acknowledge reports within 7 days.

## Scope notes

Keepr is local-first: it makes no network requests and stores data in a local
SQLite database. The main risk areas are the backup/restore importer (untrusted
`.keepr` files are validated and re-imported, never executed) and Tauri IPC
boundaries (least-privilege capabilities in `src-tauri/capabilities/`).

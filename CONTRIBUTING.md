# Contributing to Keepr

Thanks for your interest! Keepr is a small, careful codebase: Rust owns data
and scheduling, Svelte owns presentation.

## Ground rules

- Business logic and persistence live in `crates/keepr-core/` and
  `src-tauri/`. Do not reimplement them in the frontend.
- Every feature must persist across restarts and handle errors without
  panics. No mock persistence, no placeholder buttons.
- UI strings go to `resources/ru.json` **and** `resources/en.json` (both are
  required — a test enforces matching keys).
- Keep it cozy: minimal, playful, no corporate task-manager aesthetics.

## Workflow

1. Fork and create a feature branch.
2. Make your change with tests:
   - Rust domain logic → `crates/keepr-core/tests/behavior.rs`;
   - resources/locales → covered by `src/lib/presentation.test.ts`;
   - user-critical flows → extend `tests/desktop.mjs` where reasonable.
3. Run the full check matrix before opening a PR:

```sh
cargo test -p keepr-core
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
npm run check
npm test
npm run tauri build -- --debug --no-bundle
npm run test:e2e
```

4. Open a PR using the template. CI (`Desktop checks`) must be green on
   Ubuntu and Windows.

## Releases

Releases are cut from tags (`v*`) by `.github/workflows/release.yml` and are
maintainer-only. Do not bump versions in feature PRs.

## Questions

Open a [Discussion](https://github.com/JustRelaXX/Keepr/discussions) for
ideas; open an Issue for bugs with steps to reproduce.

# AGENTS.md

## Project Summary

This repository contains `alfred-universal-bookmarks`, a Rust-based Alfred workflow for searching, opening, copying, deleting, and auto-updating browser bookmarks on macOS.

- Binary name: `ub`
- Main entrypoint: `src/main.rs`
- Target platform: macOS
- Runtime context: Alfred 5 workflow driven by `info.plist`
- Package manager / build tool: Cargo

## Command Rule

This repo inherits the global command rule from `~/.codex/RTK.md`.

- Always prefix shell commands with `rtk`
- Examples:
  - `rtk cargo check`
  - `rtk cargo clippy -- -D warnings`
  - `rtk cargo build`
  - `rtk just check`

## Repository Layout

- `src/main.rs`: CLI dispatcher for `search`, `update`, `delete`, `version`
- `src/extractor.rs`: reads bookmark files from Safari and Chromium-family browsers
- `src/cache.rs`: `.cache/*.rkyv` cache read/write and invalidation behavior
- `src/deleter.rs`: bookmark deletion for Safari plist and Chromium JSON
- `src/updater.rs`: GitHub Releases update check and workflow download/import
- `src/types.rs`: shared data types and supported browser list
- `info.plist`: Alfred workflow definition and wiring
- `build.rs`: injects `GIT_COMMIT` at build time
- `justfile`: common local tasks and release flow

## How The App Works

### Search flow

`ub search <keyword>`:

1. Calls `extractor::extract_bookmarks()`
2. Loads browser bookmarks from supported providers
3. Uses `.cache/<provider>.rkyv` when source bookmark files have not changed
4. Matches keyword against bookmark name first, then URL
5. Deduplicates by URL
6. Emits Alfred JSON items to stdout

### Delete flow

`ub delete "<source>\t<url>"`:

1. Resolves provider path from `BOOKMARK_PROVIDERS` or `{BROWSER}_BOOKMARKS_PATH`
2. Deletes from Safari plist or Chromium JSON
3. Invalidates the matching `.cache/*.rkyv` file

Deletion is destructive. Be careful when changing traversal or match logic.

### Update flow

`ub update`:

1. Checks GitHub Releases for `we11adam/alfred-universal-bookmarks`
2. Uses a 7-day frequency gate in Alfred cache dir
3. Chooses the best `.alfredworkflow` asset by binary architecture
4. Downloads to `/tmp/UniversalBookmarks-latest.alfredworkflow`
5. Opens the workflow file with macOS `open`

## Supported Browsers

The canonical provider list lives in `src/types.rs` as `BOOKMARK_PROVIDERS`.

- Safari uses plist parsing
- All other current providers use Chromium-style JSON bookmarks
- Custom paths are supported via `{BROWSER_NAME}_BOOKMARKS_PATH`

When adding a provider:

1. Update `BOOKMARK_PROVIDERS`
2. Confirm it matches Safari or Chromium data shape
3. Verify custom env var naming still works
4. Check delete support and cache file naming

## Development Workflow

Useful commands:

- `rtk cargo check`
- `rtk cargo clippy -- -D warnings`
- `rtk cargo build`
- `rtk cargo build --release`
- `rtk just check`
- `rtk just dev`
- `rtk just rls`

Notes:

- There are currently no Rust tests in the repo
- `just check` runs `cargo check` and clippy, but not tests
- `just dev` and `just rls` copy the built binary to repo root as `./ub`

## Validation Expectations

For code changes, prefer this validation order:

1. `rtk cargo check`
2. `rtk cargo clippy -- -D warnings`
3. If behavior changed, rebuild `./ub` with `rtk just dev` or `rtk just rls`
4. If the change touches Alfred integration, verify `info.plist` assumptions and, when possible, do a manual Alfred smoke test on macOS

Manual smoke checks that matter:

- Search returns merged results across browsers
- Duplicate URLs only appear once
- `Cmd+Enter` delete still deletes the intended bookmark
- `Option+Enter` copy behavior remains intact
- Auto-update changes do not break cache-dir resolution or asset selection

## File-Specific Guidance

### `src/main.rs`

- Keep stdout reserved for Alfred JSON or explicit CLI output
- `eprintln!` is used for debug/logging; avoid polluting stdout during `search`
- Search result ordering is intentional: name matches before URL matches

### `src/extractor.rs`

- Keep provider-specific parsing isolated
- Missing bookmark files are normal and should not be treated as fatal
- Path formatting uses `PATH_SPLIT`; keep display formatting stable unless updating Alfred UX intentionally

### `src/cache.rs`

- Cache validity is based on source-file mtime vs cache-file mtime
- Cache files live in repo-local `.cache/` during local runs
- Do not introduce cache writes that break Alfred execution in read-limited contexts

### `src/deleter.rs`

- Any deletion bug can remove real user bookmarks
- Prefer conservative matching and explicit failure over broad deletion logic
- Cache invalidation after deletion is part of correctness, not an optimization

### `src/updater.rs`

- Avoid adding heavy dependencies for update logic without strong reason
- The file intentionally uses macOS-native tools like `curl`, `lipo`, `open`, and `PlistBuddy`
- Preserve the “best effort, never panic, don’t block search flow” behavior

### `info.plist`

- Treat as production workflow wiring, not incidental config
- Changes here affect Alfred keyword, modifiers, variable passing, and action routing
- If CLI args or action names change, `info.plist` likely must change too

## Release Notes

The release flow is defined in `justfile`.

- Version must be updated in both `Cargo.toml` and `info.plist`
- `build.rs` injects the short git commit hash into `ub version`
- Releases are tag-driven with `v<version>`

Do not change versioning or release asset assumptions casually; `src/updater.rs` depends on GitHub release metadata and `.alfredworkflow` assets.

## Practical Rules For Future Agents

- Read the touched module end-to-end before editing
- Prefer small, behavior-preserving changes
- Keep macOS and Alfred runtime assumptions explicit
- Do not treat missing browsers or missing bookmark files as errors
- Be extra careful with deletion, cache invalidation, and stdout formatting
- If you change user-visible behavior, update both `README.md` and `README_zh-CN.md`

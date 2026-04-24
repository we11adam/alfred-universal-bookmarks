# Alfred Universal Bookmarks

[![Release Alfred Workflow](https://github.com/we11adam/alfred-universal-bookmarks/actions/workflows/release.yml/badge.svg)](https://github.com/we11adam/alfred-universal-bookmarks/actions/workflows/release.yml)

English | [简体中文](README_zh-CN.md)

A blazing-fast Alfred Workflow written in Rust that allows you to search bookmarks across all your installed browsers simultaneously.

## Features

- ⚡ **Blazing Fast**: Powered by Rust with rkyv-based caching for near-instant searching across thousands of bookmarks.
- 🌐 **Multi-Browser Support**: Search bookmarks from Safari, Chrome, Brave, Arc, Edge, and more.
- 🦄 **Unified Results**: See all your bookmarks in one place, with their source and folder path clearly displayed.
- 🧹 **Deduplication**: Automatically hides duplicate URLs across different browsers.
- ✏️ **Delete Bookmarks**: Press `Cmd/Command + Enter` to delete the selected bookmark directly.
- 🔄 **Auto-Update**: Automatically checks for updates and keeps your workflow up to date.
- 📎 **Copy to Clipboard**: Use `Alt/Option + Enter` to copy the URL directly to your clipboard.
-  **Universal Binary**: Supports both Intel-based and Apple Silicon Macs.

## Supported Browsers

Universal Bookmarks currently supports:

- **Safari**
- **Google Chrome**
- **Microsoft Edge**
- **Brave & Brave Beta**
- **Arc**
- **Vivaldi**
- **Opera**
- **Sidekick**
- **Chromium**
- **Thorium**
- **Dia**
- **Comet**
- **Helium**

### Custom Bookmarks Path

If your bookmarks file is located in a non-standard directory (e.g., using a specific browser profile), you can override the default search path using environment variables in the workflow configuration.

Set an environment variable following the pattern `{BROWSER_NAME}_BOOKMARKS_PATH`. The browser name must be in uppercase, with any spaces replaced by underscores. If you provide a relative path, it will be automatically resolved relative to your `$HOME` directory.

**Examples:**
- `SAFARI_BOOKMARKS_PATH=/custom/absolute/path/to/Bookmarks.plist`
- `GOOGLE_CHROME_BOOKMARKS_PATH=Library/Application Support/Google/Chrome/Profile 1/Bookmarks`
- `BRAVE_BETA_BOOKMARKS_PATH=/Volumes/ExternalData/Brave/Bookmarks`

## Installation
Go to the [releases](https://github.com/we11adam/alfred-universal-bookmarks/releases) page, click on the UniversalBookmarks.alfredworkflow file to download it. Then, double-click the file to import it into Alfred.

### Prerequisites

- [Alfred 5](https://www.alfredapp.com/) with Powerpack.

### Setting up Alfred

1. Open the project folder in Finder.
2. The `info.plist` is already configured. You can import the workflow into Alfred by double-clicking the folder or the `info.plist` file (Alfred should recognize it).
3. Ensure the `ub` binary is in the workflow directory.

## Usage

1. Open Alfred.
2. Type `ub` followed by your search query.

Search behavior:
- Case-insensitive name and URL substring matching.
- Pinyin matching for Chinese bookmark names: type full pinyin (e.g., `gongzuotianbao`), partial pinyin (`zuotian`), initials (`gztb`), or mixed (`gztianbao`) to match `工作填报`.
- Matching priority: name → pinyin → URL.

3. Press `Enter` to open the bookmark in your default browser.
4. Press `Alt/Option + Enter` to copy the URL to your clipboard.
5. Press `Cmd/Command + Enter` to delete the selected bookmark.

Command mode:
- Type `ub :` to browse workflow commands.
- Type `ub :update` to run an immediate update check and, if a newer release exists, download and open the workflow installer.
- Type `ub :about` to show the current workflow version and embedded git commit.

Developer (running locally):
- Run unit tests with `rtk cargo test` or run `rtk just check` (the check recipe now runs tests including pinyin cases).
- Development note: When running from a git checkout (a `.git` directory exists in the current working directory), the auto-update check is disabled to avoid downloading and installing releases during local development.


## License

MIT

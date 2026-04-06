# Alfred Universal Bookmarks

[![Release Alfred Workflow](https://github.com/we11adam/alfred-universal-bookmarks/actions/workflows/release.yml/badge.svg)](https://github.com/we11adam/alfred-universal-bookmarks/actions/workflows/release.yml)

[English](README.md) | 简体中文

一个使用 Rust 编写、速度极快的 Alfred 工作流，让你能够同时搜索所有已安装浏览器中的书签。

## 特性

- ⚡ **极速**: 使用 Rust 编写，基于 rkyv 缓存技术，支持在数千个书签中进行近乎瞬间的搜索。
- 🌐 **多浏览器支持**: 支持搜索来自 Safari、Chrome、Brave、Arc、Edge 等浏览器的书签。
- 🦄 **统一结果**: 将所有书签集中在一处，并清晰地显示它们的来源和文件夹路径。
- 🧹 **去重**: 自动隐藏跨浏览器的重复 URL。
- ✏️ **删除书签**: 按下 `Cmd/Command + Enter` 可直接删除选中的书签。
- 🔄 **自动更新**: 自动检查更新并保持工作流为最新版本。
- 📎 **复制到剪贴板**: 使用 `Alt/Option + Enter` 快捷键可将书签 URL 直接复制到剪贴板。
-  **通用二进制**: 原生支持基于 Intel 和 Apple Silicon 的 Mac 设备。

## 支持的浏览器

Universal Bookmarks 目前支持以下浏览器：

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

### 自定义书签路径

如果你的书签文件保存在非默认目录下（例如：使用了不一样的浏览器配置文件），你可以通过在工作流配置中设置环境变量来覆盖默认的搜索路径。

设置环境变量的格式为 `{BROWSER_NAME}_BOOKMARKS_PATH`。浏览器名称必须大写，并将其中的空格替换为下划线。如果你提供的是相对路径，程序会自动将其与你的 `$HOME` 目录拼接解析。

**示例：**
- `SAFARI_BOOKMARKS_PATH=/custom/absolute/path/to/Bookmarks.plist`
- `GOOGLE_CHROME_BOOKMARKS_PATH=Library/Application Support/Google/Chrome/Profile 1/Bookmarks`
- `BRAVE_BETA_BOOKMARKS_PATH=/Volumes/ExternalData/Brave/Bookmarks`

## 安装

前往 [releases](https://github.com/we11adam/alfred-universal-bookmarks/releases) 页面，点击下载 `UniversalBookmarks.alfredworkflow` 文件。然后双击该文件即可将其导入到 Alfred 中。

### 环境要求

- 带有 Powerpack 激活的 [Alfred 5](https://www.alfredapp.com/)。

### 设置 Alfred（通过源码）

1. 在 Finder 中打开本项目文件夹。
2. 项目中的 `info.plist` 已经配置完毕。你可以通过双击整个文件夹或 `info.plist` 文件本身将工作流导入 Alfred (Alfred 能够自动识别)。
3. 确保工作流目录下存在编译好的 `ub` 二进制文件。

## 使用方法

1. 唤出 Alfred。
2. 输入关键字 `ub` 并加上你要搜索的内容。
3. 按下 `Enter` 在默认浏览器中打开该书签。
4. 按下 `Alt/Option + Enter` 将 URL 复制到剪贴板。
5. 按下 `Cmd/Command + Enter` 删除选中的书签。

## 开源协议

MIT

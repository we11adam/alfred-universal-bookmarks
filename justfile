# Set default shell to bash
set shell := ["bash", "-c"]

# Show all available commands
default:
    @just --list

# Format syntax
fmt:
    cargo fmt --all

# Make clippy happy
fix:
    cargo clippy --fix

# Run lints and tests to ensure code quality before release
check:
    cargo check
    cargo clippy -- -D warnings
    # cargo test # uncomment if tests are added

dev:
    cargo build
    mv target/debug/ub .

# Release a new version (e.g. `just release 1.0.1`), updates config, commits, and tags
release NEW_VERSION: check
    #!/usr/bin/env bash
    set -euo pipefail

    if [ -z "{{NEW_VERSION}}" ]; then
        echo "❌ Error: Version parameter cannot be empty. Usage: just release <version>"
        exit 1
    fi

    if [ -n "$(git tag -l "v{{NEW_VERSION}}")" ]; then
        echo "❌ Error: Tag v{{NEW_VERSION}} already exists! Please specify a new version."
        exit 1
    fi

    # 1. Check if git working tree is clean
    if ! git diff --quiet || ! git diff --cached --quiet; then
        echo "❌ Error: Git working tree is not clean. Please commit or stash your changes first."
        exit 1
    fi

    echo "🚀 Starting release for version: v{{NEW_VERSION}}..."

    # 2. Update version in Cargo.toml (using macOS compatible sed)
    # Note: 'cargo set-version {{NEW_VERSION}}' from cargo-edit is also popular, but sed requires zero dependencies.
    sed -i '' -e 's/^version = ".*"/version = "{{NEW_VERSION}}"/' Cargo.toml

    # 3. Update Cargo.lock to sync the new version
    cargo update -p alfred-universal-bookmarks || true

    echo "📦 Committing version bump..."
    git add Cargo.toml Cargo.lock
    git commit -m "chore: bump version to v{{NEW_VERSION}}"

    echo "🏷️ Creating git tag..."
    git tag -a "v{{NEW_VERSION}}" -m "Release v{{NEW_VERSION}}"

    echo "✅ Release ready!"
    echo "Run the following command to push to remote and trigger GitHub Actions:"
    echo "  git push origin master && git push origin master --tags"

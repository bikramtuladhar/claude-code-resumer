# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/bikramtuladhar/claude-code-resumer/compare/v0.1.4...v0.2.0) (2026-01-24)


### Features

* pass Claude Code CLI options through cs ([eb45b48](https://github.com/bikramtuladhar/claude-code-resumer/commit/eb45b48cd2825b28d8072ba6c9d9e64c2d7f9b02))

## [0.1.4](https://github.com/bikramtuladhar/claude-code-resumer/compare/v0.1.3...v0.1.4) (2026-01-17)


### Bug Fixes

* update Cargo.toml version in manual release workflow ([#13](https://github.com/bikramtuladhar/claude-code-resumer/issues/13)) ([fdf412f](https://github.com/bikramtuladhar/claude-code-resumer/commit/fdf412fdecfe0b3a3a2236b6a234fe6a4c8796b5))

## [0.1.2](https://github.com/bikramtuladhar/claude-code-resumer/compare/v0.1.1...v0.1.2) (2026-01-17)


### Bug Fixes

* simplify homebrew structure and update to v0.1.1 ([455bd23](https://github.com/bikramtuladhar/claude-code-resumer/commit/455bd23d027ff7d3e7738d489b220708e3c8104b))
* simplify homebrew structure and update to v0.1.1 ([75cab2a](https://github.com/bikramtuladhar/claude-code-resumer/commit/75cab2a96d8d9167a9905b31c04fc140dd93fe16))

## 0.1.0 (2026-01-17)


### Features

* add release automation with Release Please ([6771fbf](https://github.com/bikramtuladhar/claude-code-resumer/commit/6771fbf24a0e159c66aefb16fd1fd0eac56d39e4))

## [Unreleased]

### Added
- Deterministic session UUIDs from `folder+branch` using RFC 4122 UUID v5
- Session persistence with `~/.cs/sessions` database for instant resume
- `--force` / `-f` flag to force create new session (ignores database)
- `--reset` flag to remove stale session from database and create new
- `--list` / `-l` flag to list all tracked sessions
- `--clear` flag to clear entire session database
- `--dry-run` / `-n` flag to show session info without launching Claude
- `CS_NAMESPACE` environment variable for custom UUID namespaces
- Git tag version injection at build time
- Cross-platform support: macOS (Intel & Apple Silicon), Linux (x64 & ARM64)
- Homebrew formula for easy installation
- GitHub Actions workflow for automated release builds
- Unit tests for core functionality
- Release Please integration for automated releases from conventional commits
- Auto-generated changelogs from commit messages
- Automatic Homebrew formula SHA256 hash updates after releases
- Non-interactive mode (`-y`) for release script
- Reusable GitHub Actions build workflow

[Unreleased]: https://github.com/bikramtuladhar/claude-code-resumer/commits/main

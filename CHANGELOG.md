# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0](https://github.com/bikramtuladhar/claude-code-resumer/compare/v0.1.0...v0.2.0) (2026-01-17)


### Features

* add Formula symlink for direct homebrew tap support ([491af17](https://github.com/bikramtuladhar/claude-code-resumer/commit/491af1795719fe3f8dfa99788d094686305b798c))
* add self-update command and Claude CLI detection ([9121f75](https://github.com/bikramtuladhar/claude-code-resumer/commit/9121f75936f5eaa3edd2daf0acd01cc68f02e836))

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

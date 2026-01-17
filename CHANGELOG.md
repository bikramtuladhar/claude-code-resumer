# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.2.0] - 2026-01-17

### Added
- GitHub CLI integration for release management

## [1.1.0] - 2025-01-16

### Fixed
- Fixed release workflow: changed `dtolnay/rust-action` to correct `dtolnay/rust-toolchain`

### Added
- CI workflow that runs tests on all pull requests before merge
- Cargo caching in CI for faster builds

## [1.0.0] - 2025-01-16

### Added
- Deterministic session UUIDs from `folder+branch` using RFC 4122 UUID v5
- Session persistence with `~/.cs/sessions` database for instant resume
- `--force` / `-f` flag to force create new session (ignores database)
- `--reset` flag to remove stale session from database and create new
- `--list` / `-l` flag to list all tracked sessions
- `--clear` flag to clear entire session database
- `--dry-run` / `-n` flag to show session info without launching Claude
- `CS_NAMESPACE` environment variable for custom UUID namespaces
- Cross-platform support: macOS (Intel & Apple Silicon), Linux (x64 & ARM64)
- Homebrew formula for easy installation
- GitHub Actions workflow for automated release builds
- Unit tests for core functionality

### Technical Details
- Native Rust binary (~330KB optimized)
- Uses SHA-1 based UUID v5 generation
- Session database is a simple newline-delimited text file
- Replaces current process when launching Claude (Unix)

[Unreleased]: https://github.com/bikramtuladhar/claude-code-resumer/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/bikramtuladhar/claude-code-resumer/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/bikramtuladhar/claude-code-resumer/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/bikramtuladhar/claude-code-resumer/releases/tag/v1.0.0

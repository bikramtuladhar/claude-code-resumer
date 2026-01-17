# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[Unreleased]: https://github.com/bikramtuladhar/claude-code-resumer/commits/main

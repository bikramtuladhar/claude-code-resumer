# cs - Claude Code Session Manager

A lightweight CLI tool that creates deterministic Claude Code sessions based on your current folder and git branch. Same folder + same branch = same session, always.

## Quick Start

```bash
# One-line install (works on macOS, Linux, FreeBSD, Android Termux, iOS iSH)
curl -fsSL https://raw.githubusercontent.com/bikramtuladhar/claude-code-resumer/main/install.sh | bash

# Use it
cd ~/projects/my-app
cs                    # Creates/resumes session for my-app+main
git checkout feature  # Switch branch
cs                    # Creates/resumes session for my-app+feature
```

## Why?

Claude Code's `--resume` flag opens an interactive picker if the session doesn't exist, and `--session-id` requires manually tracking UUIDs. There's no built-in way to automatically create/resume sessions with predictable names based on your project context.

**cs** solves this by generating a deterministic UUID v5 from `folder_name+branch_name`:
- Running `cs` in `my-app` on branch `main` always opens the same session
- Switching to `feature/auth` branch opens a different session
- Coming back to `main` resumes exactly where you left off

## Features

- **Deterministic sessions** - Same folder + branch = same UUID (RFC 4122 UUID v5)
- **Instant resume** - Local session database for O(1) lookups
- **Zero configuration** - Just run `cs` in any git repository
- **Session management** - Force create, reset stale sessions, list/clear database
- **Custom namespaces** - Isolate session pools via `CS_NAMESPACE` env var
- **Cross-platform** - macOS, Linux, FreeBSD, Android (Termux), iOS (iSH)
- **Fast** - Native Rust binary, ~330KB, instant startup

## Installation

### Automatic Installation (Recommended)

The installer automatically detects your platform and installs the correct binary:

```bash
curl -fsSL https://raw.githubusercontent.com/bikramtuladhar/claude-code-resumer/main/install.sh | bash
```

**Supported platforms:**
- macOS (Intel & Apple Silicon)
- Linux (x64 & ARM64)
- FreeBSD (x64)
- Android Termux (ARM64, ARM32, x64)
- iOS iSH (i686)

### Manual Download

#### macOS

```bash
# Apple Silicon (M1/M2/M3)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-macos-arm64 -o cs
chmod +x cs && sudo mv cs /usr/local/bin/

# Intel
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-macos-intel -o cs
chmod +x cs && sudo mv cs /usr/local/bin/
```

#### Linux

```bash
# x64 (glibc)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-linux-x64 -o cs
chmod +x cs && sudo mv cs /usr/local/bin/

# ARM64 (glibc)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-linux-arm64 -o cs
chmod +x cs && sudo mv cs /usr/local/bin/

# x64 (musl/Alpine)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-linux-x64-musl -o cs
chmod +x cs && sudo mv cs /usr/local/bin/

# i686 (musl/Alpine)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-linux-i686-musl -o cs
chmod +x cs && sudo mv cs /usr/local/bin/
```

#### FreeBSD

```bash
# x64
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-freebsd-x64 -o cs
chmod +x cs && sudo mv cs /usr/local/bin/
```

#### Android (Termux)

```bash
# ARM64 (most Android devices)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-android-arm64 -o cs
chmod +x cs && mv cs $PREFIX/bin/

# ARM32 (older devices)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-android-arm32 -o cs
chmod +x cs && mv cs $PREFIX/bin/

# x64 (emulators)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-android-x64 -o cs
chmod +x cs && mv cs $PREFIX/bin/
```

#### iOS (iSH)

iSH is an x86 Linux emulator for iOS. Use the musl-linked i686 binary:

```bash
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-linux-i686-musl -o cs
chmod +x cs && mv cs /usr/local/bin/
```

### Homebrew (macOS/Linux)

```bash
brew tap bikramtuladhar/tap
brew install cs
```

### Build from Source

```bash
git clone https://github.com/bikramtuladhar/claude-code-resumer.git
cd claude-code-resumer
cargo build --release
cp target/release/cs /usr/local/bin/
```

## Usage

```bash
cd ~/projects/my-app
cs
# ┌─────────────────────────────────────────────
# │ Session: my-app+main
# │ UUID:    a1b2c3d4-e5f6-5789-abcd-ef0123456789
# │ Status:  new
# └─────────────────────────────────────────────
# Creating session...

# Switch branches = different session
git checkout feature/auth
cs
# │ Session: my-app+feature/auth
# │ Status:  new

# Come back to main = same session as before
git checkout main
cs
# │ Session: my-app+main
# │ Status:  exists
# Resuming session...
```

## Commands

| Command | Short | Description |
|---------|-------|-------------|
| `cs` | | Start/resume session for current folder+branch |
| `cs --force` | `-f` | Force create new session (ignores database) |
| `cs --reset` | | Remove current session from DB, then create new |
| `cs --list` | `-l` | List all sessions in database |
| `cs --clear` | | Clear entire session database |
| `cs --dry-run` | `-n` | Show session info without launching Claude |
| `cs --help` | `-h` | Show help message |
| `cs --version` | `-v` | Show version |

## How It Works

```
                            ┌─────────────────────────────────────┐
                            │           cs [flags]                │
                            └─────────────────┬───────────────────┘
                                              │
                            ┌─────────────────▼───────────────────┐
                            │  Generate UUID v5 from folder+branch │
                            │  my-app+main → a1b2c3d4-...         │
                            └─────────────────┬───────────────────┘
                                              │
                  ┌───────────────────────────┼───────────────────────────┐
                  │                           │                           │
           --force or --reset?          Check ~/.cs/sessions         --list/--clear?
                  │                           │                           │
                  ▼                           ▼                           ▼
        ┌─────────────────┐         ┌─────────────────┐         ┌─────────────────┐
        │  Skip DB check  │         │   UUID exists?  │         │  Manage DB and  │
        │  (--reset also  │         │                 │         │  exit           │
        │  removes entry) │         └────────┬────────┘         └─────────────────┘
        └────────┬────────┘                  │
                 │              ┌────────────┴────────────┐
                 │              │                         │
                 │            Yes                        No
                 │              │                         │
                 │              ▼                         ▼
                 │    ┌─────────────────┐      ┌─────────────────┐
                 │    │  claude -r UUID │      │ Save to DB      │
                 │    │  (resume)       │      │ claude --session│
                 │    └─────────────────┘      │ -id UUID        │
                 │                             │ (create)        │
                 │                             └─────────────────┘
                 │                                      ▲
                 └──────────────────────────────────────┘
```

**Normal flow:**
1. Get current folder name and git branch
2. Generate deterministic UUID v5 from `folder+branch`
3. Check local session database (`~/.cs/sessions`)
4. If exists → resume with `claude -r <uuid>`
5. If new → create with `claude --session-id <uuid>` and save to DB

**Force/Reset flow:**
- `--force` skips the DB check entirely, always creates
- `--reset` removes any existing DB entry first, then creates

## Platform-Specific Notes

### Android (Termux)

1. Install [Termux](https://termux.dev/) from F-Droid (not Play Store)
2. Install dependencies: `pkg install git curl`
3. Run the install script or download the Android binary manually

### iOS (iSH)

1. Install [iSH](https://ish.app/) from the App Store
2. Install dependencies: `apk add git curl bash`
3. Run the install script or download the i686-musl binary manually

### FreeBSD

Ensure you have `curl` installed: `pkg install curl`

## Troubleshooting

### "No conversation found" error

This happens when a session was saved to the local DB but Claude never actually created a conversation (e.g., you exited without sending a message).

```bash
cs --reset   # Clears stale entry and creates fresh session
```

### Need to start fresh on the current branch

```bash
cs --force   # Ignores DB, doesn't modify it
# or
cs --reset   # Removes from DB, then creates new
```

### View/manage tracked sessions

```bash
cs --list    # List all sessions in database
cs --clear   # Clear entire session database
```

### Binary not found after installation

Make sure the installation directory is in your PATH:

```bash
# For ~/.local/bin (common on Linux)
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# For Termux
# $PREFIX/bin should already be in PATH

# For iSH
echo 'export PATH="/usr/local/bin:$PATH"' >> ~/.profile
source ~/.profile
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CS_NAMESPACE` | Custom UUID v5 namespace for isolated session pools | DNS namespace (RFC 4122) |

**Example:** Keep work and personal sessions separate:

```bash
# In your work shell profile
export CS_NAMESPACE="11111111-1111-1111-1111-111111111111"

# In your personal shell profile
export CS_NAMESPACE="22222222-2222-2222-2222-222222222222"

# Same folder+branch will now produce different UUIDs
```

### Files

| Path | Description |
|------|-------------|
| `~/.cs/sessions` | Session database (one UUID per line) |

## Requirements

- [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) installed and in PATH
- Git repository (for branch detection)

## Available Binaries

| Platform | Architecture | Binary Name |
|----------|--------------|-------------|
| macOS | Apple Silicon (M1/M2/M3) | `cs-macos-arm64` |
| macOS | Intel | `cs-macos-intel` |
| Linux | x64 (glibc) | `cs-linux-x64` |
| Linux | ARM64 (glibc) | `cs-linux-arm64` |
| Linux | x64 (musl) | `cs-linux-x64-musl` |
| Linux | i686 (musl) | `cs-linux-i686-musl` |
| FreeBSD | x64 | `cs-freebsd-x64` |
| Android | ARM64 | `cs-android-arm64` |
| Android | ARM32 | `cs-android-arm32` |
| Android | x64 | `cs-android-x64` |

## Development

```bash
cargo test           # Run tests
cargo build          # Build debug
cargo build --release  # Build release
./target/release/cs --dry-run  # Test locally
```

## License

MIT

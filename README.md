# cs - Claude Code Session Manager

A lightweight CLI tool that creates deterministic Claude Code sessions based on your current folder and git branch. Same folder + same branch = same session, always.

## Quick Start

```bash
# Install (macOS Apple Silicon)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-macos-arm64 -o cs
chmod +x cs && sudo mv cs /usr/local/bin/

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
- **Cross-platform** - macOS (Intel & Apple Silicon) and Linux
- **Fast** - Native Rust binary, ~330KB, instant startup

## Installation

### Direct Download

```bash
# macOS (Apple Silicon)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-macos-arm64 -o cs
chmod +x cs && sudo mv cs /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-macos-intel -o cs
chmod +x cs && sudo mv cs /usr/local/bin/

# Linux (x64)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-linux-x64 -o cs
chmod +x cs && sudo mv cs /usr/local/bin/

# Linux (ARM64)
curl -L https://github.com/bikramtuladhar/claude-code-resumer/releases/latest/download/cs-linux-arm64 -o cs
chmod +x cs && sudo mv cs /usr/local/bin/
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

## Development

```bash
cargo test           # Run tests
cargo build          # Build debug
cargo build --release  # Build release
./target/release/cs --dry-run  # Test locally
```

## License

MIT

# cs - Claude Code Session Manager

A lightweight CLI tool that creates deterministic Claude Code sessions based on your current folder and git branch.

## Why?

- **Consistency**: Same folder + branch = same session, always
- **No manual naming**: Sessions auto-named from your project context
- **Branch switching**: Each branch gets its own session automatically
- **Resume anywhere**: Come back days later, same session continues

## Installation

### Direct Download

Download the latest binary from [Releases](https://github.com/YOUR_USERNAME/claude-session-manager/releases):

```bash
# macOS (Apple Silicon)
curl -L https://github.com/YOUR_USERNAME/claude-session-manager/releases/latest/download/cs-macos-arm64 -o cs
chmod +x cs
sudo mv cs /usr/local/bin/

# macOS (Intel)
curl -L https://github.com/YOUR_USERNAME/claude-session-manager/releases/latest/download/cs-macos-intel -o cs
chmod +x cs
sudo mv cs /usr/local/bin/

# Linux (x64)
curl -L https://github.com/YOUR_USERNAME/claude-session-manager/releases/latest/download/cs-linux-x64 -o cs
chmod +x cs
sudo mv cs /usr/local/bin/
```

### Build from Source

```bash
git clone https://github.com/YOUR_USERNAME/claude-session-manager.git
cd claude-session-manager
cargo build --release
cp target/release/cs /usr/local/bin/
```

## Usage

```bash
# In any git repository
cd ~/projects/my-app
cs   # Opens/creates session: my-app+main

# Switch branches = different session
git checkout feature/auth
cs   # Opens/creates session: my-app+feature/auth

# Come back to main = same session as before
git checkout main
cs   # Resumes session: my-app+main
```

### Options

```
cs              Start/resume session for current folder+branch
cs --dry-run    Show session info without launching claude
cs --help       Show help message
cs --version    Show version
```

## How It Works

1. Gets current folder name and git branch
2. Generates deterministic UUID v5 from `folder+branch`
3. Calls `claude --session-id <uuid>`

The UUID is generated using RFC 4122 UUID v5 (SHA-1 based), so the same input always produces the same UUID.

## Requirements

- [Claude Code CLI](https://claude.ai/code) installed and in PATH
- Git repository

## License

MIT

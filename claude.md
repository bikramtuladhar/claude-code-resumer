# Claude Session Manager (`cs`)

A lightweight CLI tool that creates deterministic Claude Code sessions based on your current folder and git branch. Same folder + same branch = same session, always.

## Problem

Claude Code's `--resume` flag opens an interactive picker if the session doesn't exist, and `--session-id` requires a UUID. There's no built-in way to automatically create/resume sessions with predictable names based on your project context.

## Solution

`cs` generates a deterministic UUID v5 from `folder_name+branch_name`, so:
- Running `cs` in `my-app` on branch `main` always opens the same session
- Switching to `feature/auth` branch opens a different session
- Coming back to `main` resumes exactly where you left off

## Features

- **Deterministic sessions**: Same folder + branch = same UUID (RFC 4122 UUID v5)
- **Zero configuration**: Just run `cs` in any git repository
- **Cross-platform**: macOS (Intel & Apple Silicon) and Linux
- **Fast**: Native binary, instant startup
- **Simple**: Single binary, no dependencies

---

## Implementation Plan

### Phase 1: Project Setup

```bash
# Create project structure
mkdir -p claude-session-manager
cd claude-session-manager

# Initialize
cargo init --name cs
```

**Directory Structure:**
```
claude-session-manager/
├── Cargo.toml
├── src/
│   └── main.rs
├── README.md
├── LICENSE
├── .github/
│   └── workflows/
│       └── release.yml
└── .gitignore
```

### Phase 2: Core Implementation

#### 2.1 Cargo.toml

```toml
[package]
name = "cs"
version = "1.0.0"
edition = "2021"
description = "Claude Code Session Manager - deterministic sessions based on folder+branch"
repository = "https://github.com/YOUR_USERNAME/claude-session-manager"
license = "MIT"
keywords = ["claude", "cli", "session", "developer-tools"]
categories = ["command-line-utilities", "development-tools"]

[dependencies]
sha1 = "0.10"

[profile.release]
strip = true
lto = true
opt-level = "z"
codegen-units = 1
```

#### 2.2 src/main.rs

```rust
use sha1::{Digest, Sha1};
use std::env;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Command, exit};

/// UUID v5 namespace (DNS namespace from RFC 4122)
const DNS_NAMESPACE: [u8; 16] = [
    0x6b, 0xa7, 0xb8, 0x10,
    0x9d, 0xad, 0x11, 0xd1,
    0x80, 0xb4, 0x00, 0xc0,
    0x4f, 0xd4, 0x30, 0xc8,
];

/// Generate a deterministic UUID v5 from a name
fn generate_uuid5(name: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(&DNS_NAMESPACE);
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();

    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x50; // Version 5
    bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant 10xx

    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
        u16::from_be_bytes([bytes[4], bytes[5]]),
        u16::from_be_bytes([bytes[6], bytes[7]]),
        u16::from_be_bytes([bytes[8], bytes[9]]),
        u64::from_be_bytes([0, 0, bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]])
    )
}

/// Get current git branch name
fn get_git_branch() -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|_| "Failed to execute git command")?;

    if !output.status.success() {
        return Err("Not a git repository or no branch found".to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get current folder name
fn get_folder_name() -> Result<String, String> {
    env::current_dir()
        .map_err(|_| "Failed to get current directory")?
        .file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| "Failed to get folder name".to_string())
}

fn print_help() {
    eprintln!("cs - Claude Code Session Manager");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    cs              Start/resume session for current folder+branch");
    eprintln!("    cs --dry-run    Show session info without launching claude");
    eprintln!("    cs --help       Show this help message");
    eprintln!("    cs --version    Show version");
    eprintln!();
    eprintln!("Session name format: <folder>+<branch>");
    eprintln!("Example: my-project+feature/auth -> UUID: 4b513bfa-8c71-512b-...");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // Handle flags
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                print_help();
                return;
            }
            "--version" | "-v" => {
                println!("cs {}", env!("CARGO_PKG_VERSION"));
                return;
            }
            "--dry-run" | "-n" => {
                // Continue but don't launch claude
            }
            arg => {
                eprintln!("Unknown argument: {}", arg);
                eprintln!("Run 'cs --help' for usage");
                exit(1);
            }
        }
    }

    // Get folder name
    let folder_name = match get_folder_name() {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    // Get git branch
    let branch_name = match get_git_branch() {
        Ok(name) => name,
        Err(e) => {
            eprintln!("Error: {}", e);
            exit(1);
        }
    };

    // Create session name and UUID
    let session_name = format!("{}+{}", folder_name, branch_name);
    let session_uuid = generate_uuid5(&session_name);

    // Print info
    println!("┌─────────────────────────────────────────────");
    println!("│ Session: {}", session_name);
    println!("│ UUID:    {}", session_uuid);
    println!("└─────────────────────────────────────────────");
    println!();

    // Check for dry-run
    if args.len() > 1 && (args[1] == "--dry-run" || args[1] == "-n") {
        return;
    }

    // Launch claude - use exec to replace current process
    let err = Command::new("claude")
        .args(["--session-id", &session_uuid])
        .exec();
    
    // If we get here, exec failed
    eprintln!("Error launching claude: {}", err);
    exit(1);
}
```

### Phase 3: GitHub Actions for Cross-Platform Builds

#### 3.1 .github/workflows/release.yml

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

permissions:
  contents: write

jobs:
  build:
    name: Build ${{ matrix.target }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          # macOS
          - target: x86_64-apple-darwin
            os: macos-latest
            name: cs-macos-intel
          - target: aarch64-apple-darwin
            os: macos-latest
            name: cs-macos-arm64
          # Linux
          - target: x86_64-unknown-linux-gnu
            os: ubuntu-latest
            name: cs-linux-x64
          - target: aarch64-unknown-linux-gnu
            os: ubuntu-latest
            name: cs-linux-arm64

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install cross-compilation tools (Linux ARM64)
        if: matrix.target == 'aarch64-unknown-linux-gnu'
        run: |
          sudo apt-get update
          sudo apt-get install -y gcc-aarch64-linux-gnu

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Rename binary
        run: |
          cp target/${{ matrix.target }}/release/cs ${{ matrix.name }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.name }}
          path: ${{ matrix.name }}

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: artifacts/*/*
          generate_release_notes: true
```

### Phase 4: Documentation

#### 4.1 README.md

```markdown
# cs - Claude Code Session Manager

A lightweight CLI tool that creates deterministic Claude Code sessions based on your current folder and git branch.

## Why?

- **Consistency**: Same folder + branch = same session, always
- **No manual naming**: Sessions auto-named from your project context  
- **Branch switching**: Each branch gets its own session automatically
- **Resume anywhere**: Come back days later, same session continues

## Installation

### Homebrew (macOS)
```bash
brew install YOUR_USERNAME/tap/cs
```

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
```

#### 4.2 .gitignore

```
/target
Cargo.lock
.DS_Store
```

#### 4.3 LICENSE

```
MIT License

Copyright (c) 2025

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

---

## Build & Test Locally

```bash
# Clone/create project
cd claude-session-manager

# Build debug
cargo build

# Build release
cargo build --release

# Test
./target/release/cs --dry-run

# Install locally
cp target/release/cs /usr/local/bin/
```

## Release Checklist

1. [ ] Update version in `Cargo.toml`
2. [ ] Commit changes
3. [ ] Create and push tag:
   ```bash
   git tag -a v1.0.0 -m "Release v1.0.0"
   git push origin v1.0.0
   ```
4. [ ] GitHub Actions builds and creates release automatically
5. [ ] Update Homebrew tap (optional)

---

## Optional: Homebrew Tap

Create a separate repo `homebrew-tap` with:

**Formula/cs.rb:**
```ruby
class Cs < Formula
  desc "Claude Code Session Manager"
  homepage "https://github.com/YOUR_USERNAME/claude-session-manager"
  version "1.0.0"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/YOUR_USERNAME/claude-session-manager/releases/download/v#{version}/cs-macos-arm64"
      sha256 "SHA256_HERE"
    end
    on_intel do
      url "https://github.com/YOUR_USERNAME/claude-session-manager/releases/download/v#{version}/cs-macos-intel"
      sha256 "SHA256_HERE"
    end
  end

  on_linux do
    url "https://github.com/YOUR_USERNAME/claude-session-manager/releases/download/v#{version}/cs-linux-x64"
    sha256 "SHA256_HERE"
  end

  def install
    bin.install "cs-macos-arm64" => "cs" if Hardware::CPU.arm?
    bin.install "cs-macos-intel" => "cs" if Hardware::CPU.intel? && OS.mac?
    bin.install "cs-linux-x64" => "cs" if OS.linux?
  end

  test do
    assert_match "cs", shell_output("#{bin}/cs --version")
  end
end
```

---

## Claude Code Command to Build This

Copy this into Claude Code to scaffold the project:

```
Create a new Rust CLI project called "claude-session-manager" with:
1. A binary named "cs" that generates deterministic UUID v5 from folder+branch
2. Calls claude --session-id with the generated UUID
3. Supports --help, --version, and --dry-run flags
4. GitHub Actions workflow for cross-platform releases (macOS Intel/ARM, Linux x64/ARM)
5. README with installation and usage instructions
```

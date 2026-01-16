# Homebrew Tap for cs

This directory contains the Homebrew formula for `cs` (Claude Code Session Manager).

## Setup Your Own Tap

1. Create a new GitHub repository named `homebrew-tap`
2. Copy the `Formula/` directory to that repository
3. Update `bikramtuladhar` in the formula to your actual GitHub username
4. After releasing v1.0.0, update the SHA256 hashes (see below)

## Installation (after setup)

```bash
brew tap bikramtuladhar/tap
brew install cs
```

## Updating SHA256 Hashes After Release

After pushing a tag and GitHub Actions creates the release:

```bash
# Download and hash each binary
curl -sL https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v1.0.0/cs-macos-arm64 | shasum -a 256
curl -sL https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v1.0.0/cs-macos-intel | shasum -a 256
curl -sL https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v1.0.0/cs-linux-x64 | shasum -a 256
curl -sL https://github.com/bikramtuladhar/claude-code-resumer/releases/download/v1.0.0/cs-linux-arm64 | shasum -a 256
```

Then update the `sha256` values in `Formula/cs.rb`.

## Automated Updates

You can also add a GitHub Action to your `homebrew-tap` repo that automatically updates the formula when a new release is published.

#!/bin/bash
# DEPRECATED: This script is no longer needed!
#
# The GitHub Actions build workflow now automatically updates the
# homebrew formula with SHA256 hashes after each release.
#
# This script is kept for manual use cases only.
#
# Usage: ./scripts/update-homebrew.sh <version> <github-username>
# Example: ./scripts/update-homebrew.sh 1.0.0 bikramtuladhar

set -e

VERSION="${1:-1.0.0}"
USERNAME="${2:-bikramtuladhar}"
FORMULA="homebrew/Formula/cs.rb"

echo "Updating formula for v$VERSION from $USERNAME..."

# Download and get SHA256 for each platform
echo "Fetching SHA256 for macOS ARM64..."
SHA_MACOS_ARM64=$(curl -sL "https://github.com/$USERNAME/claude-code-resumer/releases/download/v$VERSION/cs-macos-arm64" | shasum -a 256 | cut -d' ' -f1)

echo "Fetching SHA256 for macOS Intel..."
SHA_MACOS_INTEL=$(curl -sL "https://github.com/$USERNAME/claude-code-resumer/releases/download/v$VERSION/cs-macos-intel" | shasum -a 256 | cut -d' ' -f1)

echo "Fetching SHA256 for Linux x64..."
SHA_LINUX_X64=$(curl -sL "https://github.com/$USERNAME/claude-code-resumer/releases/download/v$VERSION/cs-linux-x64" | shasum -a 256 | cut -d' ' -f1)

echo "Fetching SHA256 for Linux ARM64..."
SHA_LINUX_ARM64=$(curl -sL "https://github.com/$USERNAME/claude-code-resumer/releases/download/v$VERSION/cs-linux-arm64" | shasum -a 256 | cut -d' ' -f1)

echo ""
echo "SHA256 hashes:"
echo "  macOS ARM64:  $SHA_MACOS_ARM64"
echo "  macOS Intel:  $SHA_MACOS_INTEL"
echo "  Linux x64:    $SHA_LINUX_X64"
echo "  Linux ARM64:  $SHA_LINUX_ARM64"

# Update the formula
sed -i.bak "s/SHA256_MACOS_ARM64/$SHA_MACOS_ARM64/g" "$FORMULA"
sed -i.bak "s/SHA256_MACOS_INTEL/$SHA_MACOS_INTEL/g" "$FORMULA"
sed -i.bak "s/SHA256_LINUX_X64/$SHA_LINUX_X64/g" "$FORMULA"
sed -i.bak "s/SHA256_LINUX_ARM64/$SHA_LINUX_ARM64/g" "$FORMULA"
sed -i.bak "s/bikramtuladhar/$USERNAME/g" "$FORMULA"
rm -f "$FORMULA.bak"

echo ""
echo "Formula updated: $FORMULA"
echo ""
echo "Next steps:"
echo "1. Copy homebrew/Formula/cs.rb to your homebrew-tap repo"
echo "2. Commit and push the changes"
echo "3. Users can then install with: brew tap $USERNAME/tap && brew install cs"

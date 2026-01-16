#!/bin/bash
# Release script for cs - Claude Code Session Manager
# Usage: ./scripts/release.sh [patch|minor|major]
# Default: patch

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the increment type (default: patch)
INCREMENT_TYPE="${1:-patch}"

if [[ ! "$INCREMENT_TYPE" =~ ^(patch|minor|major)$ ]]; then
    echo -e "${RED}Error: Invalid increment type '$INCREMENT_TYPE'${NC}"
    echo "Usage: $0 [patch|minor|major]"
    exit 1
fi

# Get the latest tag or default to v0.0.0
LATEST_TAG=$(git tag -l 'v*' --sort=-v:refname | head -n1)
if [ -z "$LATEST_TAG" ]; then
    LATEST_TAG="v0.0.0"
    echo -e "${YELLOW}No existing tags found, starting from v0.0.0${NC}"
else
    echo -e "Latest tag: ${GREEN}$LATEST_TAG${NC}"
fi

# Parse version numbers
VERSION="${LATEST_TAG#v}"
IFS='.' read -r MAJOR MINOR PATCH <<< "$VERSION"

# Increment version based on type
case $INCREMENT_TYPE in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
esac

NEW_VERSION="$MAJOR.$MINOR.$PATCH"
NEW_TAG="v$NEW_VERSION"

echo -e "New version: ${GREEN}$NEW_TAG${NC} (${INCREMENT_TYPE})"
echo ""

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${YELLOW}Warning: You have uncommitted changes${NC}"
    git status --short
    echo ""
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

# Update version in Cargo.toml
echo "Updating Cargo.toml..."
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm -f Cargo.toml.bak

# Update version in homebrew formula if it exists
if [ -f "homebrew/Formula/cs.rb" ]; then
    echo "Updating homebrew formula..."
    sed -i.bak "s/^  version \".*\"/  version \"$NEW_VERSION\"/" homebrew/Formula/cs.rb
    rm -f homebrew/Formula/cs.rb.bak
fi

# Show changes
echo ""
echo "Changes to be committed:"
git diff --stat

echo ""
read -p "Create release commit and tag $NEW_TAG? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Rolling back changes..."
    git checkout -- Cargo.toml
    [ -f "homebrew/Formula/cs.rb" ] && git checkout -- homebrew/Formula/cs.rb
    echo "Aborted."
    exit 1
fi

# Create commit
git add Cargo.toml
[ -f "homebrew/Formula/cs.rb" ] && git add homebrew/Formula/cs.rb
git commit -m "Release $NEW_TAG"

# Create tag
git tag -a "$NEW_TAG" -m "Release $NEW_TAG"

echo ""
echo -e "${GREEN}Created release $NEW_TAG${NC}"
echo ""
echo "Next steps:"
echo "  1. Push the commit and tag:"
echo "     git push origin main --tags"
echo ""
echo "  2. GitHub Actions will build binaries and create the release"
echo ""
echo "  3. After release, update homebrew SHA256 hashes:"
echo "     ./scripts/update-homebrew.sh $NEW_VERSION bikramtuladhar"

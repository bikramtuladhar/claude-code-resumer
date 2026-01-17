#!/bin/bash
# Release script for cs - Claude Code Session Manager
# Usage: ./scripts/release.sh [options] [patch|minor|major]
#
# Options:
#   -y, --yes      Skip all confirmations (non-interactive mode)
#   -p, --push     Push to origin after creating release
#   -n, --dry-run  Show what would happen without making changes
#   -h, --help     Show this help message
#
# Default: patch

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

CHANGELOG="CHANGELOG.md"
REPO_URL="https://github.com/bikramtuladhar/claude-code-resumer"

# Default values
AUTO_YES=false
PUSH_AFTER=false
DRY_RUN=false
INCREMENT_TYPE="patch"

# Parse flags
while [[ $# -gt 0 ]]; do
    case $1 in
        -y|--yes)
            AUTO_YES=true
            shift
            ;;
        -p|--push)
            PUSH_AFTER=true
            shift
            ;;
        -n|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [options] [patch|minor|major]"
            echo ""
            echo "Options:"
            echo "  -y, --yes      Skip all confirmations (non-interactive mode)"
            echo "  -p, --push     Push to origin after creating release"
            echo "  -n, --dry-run  Show what would happen without making changes"
            echo "  -h, --help     Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 patch              # Interactive patch release"
            echo "  $0 -y minor           # Non-interactive minor release"
            echo "  $0 -y --push patch    # Non-interactive release with auto-push"
            echo "  $0 --dry-run major    # Preview major release changes"
            exit 0
            ;;
        patch|minor|major)
            INCREMENT_TYPE="$1"
            shift
            ;;
        *)
            echo -e "${RED}Error: Unknown option '$1'${NC}"
            echo "Usage: $0 [options] [patch|minor|major]"
            echo "Use -h or --help for more information."
            exit 1
            ;;
    esac
done

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
TODAY=$(date +%Y-%m-%d)

echo -e "New version: ${GREEN}$NEW_TAG${NC} (${INCREMENT_TYPE})"
echo ""

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${YELLOW}Warning: You have uncommitted changes${NC}"
    git status --short
    echo ""
    if [ "$AUTO_YES" = false ]; then
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Aborted."
            exit 1
        fi
    else
        echo -e "${BLUE}Auto-continuing with uncommitted changes (-y flag)${NC}"
    fi
fi

# Check if changelog has [Unreleased] section with content
if [ -f "$CHANGELOG" ]; then
    UNRELEASED_CONTENT=$(sed -n '/## \[Unreleased\]/,/## \[/p' "$CHANGELOG" | tail -n +2 | head -n -1 | grep -v '^$' || true)
    if [ -z "$UNRELEASED_CONTENT" ]; then
        echo -e "${YELLOW}Warning: No changes documented in [Unreleased] section${NC}"
        echo ""
        echo -e "${BLUE}Please add your changes to CHANGELOG.md under [Unreleased] section first.${NC}"
        echo "Example:"
        echo "  ## [Unreleased]"
        echo "  "
        echo "  ### Added"
        echo "  - New feature description"
        echo "  "
        echo "  ### Fixed"
        echo "  - Bug fix description"
        echo ""
        if [ "$AUTO_YES" = false ]; then
            read -p "Continue without changelog updates? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                echo "Aborted. Please update CHANGELOG.md first."
                exit 1
            fi
        else
            echo -e "${BLUE}Auto-continuing without changelog updates (-y flag)${NC}"
        fi
    fi
fi

# Dry run exit point
if [ "$DRY_RUN" = true ]; then
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}DRY RUN - No changes made${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "Would update:"
    echo "  - Cargo.toml version to $NEW_VERSION"
    [ -f "homebrew/Formula/cs.rb" ] && echo "  - homebrew/Formula/cs.rb version to $NEW_VERSION"
    [ -f "$CHANGELOG" ] && echo "  - CHANGELOG.md [Unreleased] -> [$NEW_VERSION] - $TODAY"
    echo ""
    echo "Would create:"
    echo "  - Commit: Release $NEW_TAG"
    echo "  - Tag: $NEW_TAG"
    [ "$PUSH_AFTER" = true ] && echo "  - Push to origin main --tags"
    exit 0
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

# Update CHANGELOG.md
if [ -f "$CHANGELOG" ]; then
    echo "Updating CHANGELOG.md..."

    # Replace [Unreleased] header with new version, add new [Unreleased] section
    sed -i.bak "s/## \[Unreleased\]/## [Unreleased]\n\n## [$NEW_VERSION] - $TODAY/" "$CHANGELOG"

    # Update the links at the bottom
    # Add new unreleased link and new version link
    if grep -q "\[Unreleased\]:" "$CHANGELOG"; then
        sed -i.bak "s|\[Unreleased\]:.*|[Unreleased]: $REPO_URL/compare/v$NEW_VERSION...HEAD\n[$NEW_VERSION]: $REPO_URL/compare/$LATEST_TAG...v$NEW_VERSION|" "$CHANGELOG"
    fi

    rm -f "$CHANGELOG.bak"
fi

# Show changes
echo ""
echo "Changes to be committed:"
git diff --stat
echo ""
git diff Cargo.toml
echo ""

if [ "$AUTO_YES" = false ]; then
    read -p "Create release commit and tag $NEW_TAG? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Rolling back changes..."
        git checkout -- Cargo.toml
        [ -f "homebrew/Formula/cs.rb" ] && git checkout -- homebrew/Formula/cs.rb
        [ -f "$CHANGELOG" ] && git checkout -- "$CHANGELOG"
        echo "Aborted."
        exit 1
    fi
else
    echo -e "${BLUE}Auto-confirming release commit and tag (-y flag)${NC}"
fi

# Create commit
git add Cargo.toml
[ -f "homebrew/Formula/cs.rb" ] && git add homebrew/Formula/cs.rb
[ -f "$CHANGELOG" ] && git add "$CHANGELOG"
git commit -m "Release $NEW_TAG"

# Create tag
git tag -a "$NEW_TAG" -m "Release $NEW_TAG"

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Created release $NEW_TAG${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Handle auto-push if requested
if [ "$PUSH_AFTER" = true ]; then
    echo "Pushing to origin..."
    git push origin main --tags
    echo ""
    echo -e "${GREEN}Pushed release to origin!${NC}"
    echo ""
    echo "Next steps:"
    echo ""
    echo "  1. GitHub Actions will build binaries and create the release"
    echo ""
    echo "  2. After release, update homebrew SHA256 hashes:"
    echo -e "     ${BLUE}./scripts/update-homebrew.sh $NEW_VERSION bikramtuladhar${NC}"
else
    echo "Next steps:"
    echo ""
    echo "  1. Push the commit and tag:"
    echo -e "     ${BLUE}git push origin main --tags${NC}"
    echo ""
    echo "  2. GitHub Actions will build binaries and create the release"
    echo ""
    echo "  3. After release, update homebrew SHA256 hashes:"
    echo -e "     ${BLUE}./scripts/update-homebrew.sh $NEW_VERSION bikramtuladhar${NC}"
fi

#!/bin/bash
# Manual Release Helper Script
#
# With release-please configured, you typically don't need this script.
# Just push commits with conventional format (feat:, fix:, etc.) and
# release-please will create a release PR automatically.
#
# This script is for manual releases when you need to:
# - Release without waiting for release-please
# - Release a specific version number
#
# Usage: ./scripts/release.sh [options] [version]
#
# Options:
#   -y, --yes      Skip all confirmations (non-interactive mode)
#   -n, --dry-run  Show what would happen without making changes
#   -h, --help     Show this help message
#
# Examples:
#   ./scripts/release.sh v1.2.0          # Interactive release
#   ./scripts/release.sh -y v1.2.0       # Non-interactive release
#   ./scripts/release.sh --dry-run v2.0.0  # Preview only

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
AUTO_YES=false
DRY_RUN=false
VERSION=""

# Parse flags
while [[ $# -gt 0 ]]; do
    case $1 in
        -y|--yes)
            AUTO_YES=true
            shift
            ;;
        -n|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            echo "Manual Release Helper Script"
            echo ""
            echo "With release-please configured, you typically don't need this script."
            echo "Just push commits with conventional format and release-please creates PRs."
            echo ""
            echo "Usage: $0 [options] <version>"
            echo ""
            echo "Options:"
            echo "  -y, --yes      Skip all confirmations (non-interactive mode)"
            echo "  -n, --dry-run  Show what would happen without making changes"
            echo "  -h, --help     Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0 v1.2.0              # Interactive release"
            echo "  $0 -y v1.2.0           # Non-interactive release"
            echo "  $0 --dry-run v2.0.0    # Preview only"
            echo ""
            echo "Recommended workflow:"
            echo "  1. Push commits with conventional format: feat:, fix:, etc."
            echo "  2. Release-please creates a 'Release PR' automatically"
            echo "  3. Merge the PR to trigger a release"
            exit 0
            ;;
        v*)
            VERSION="$1"
            shift
            ;;
        *)
            echo -e "${RED}Error: Unknown option '$1'${NC}"
            echo "Usage: $0 [options] <version>"
            echo "Use -h or --help for more information."
            exit 1
            ;;
    esac
done

# Validate version
if [ -z "$VERSION" ]; then
    echo -e "${RED}Error: Version is required${NC}"
    echo ""
    echo "Usage: $0 [options] <version>"
    echo "Example: $0 v1.2.0"
    exit 1
fi

if [[ ! "$VERSION" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo -e "${RED}Error: Invalid version format '$VERSION'${NC}"
    echo "Version must be in format: v1.2.3"
    exit 1
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}Manual Release: $VERSION${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check if tag already exists
if git rev-parse "$VERSION" >/dev/null 2>&1; then
    echo -e "${YELLOW}Warning: Tag $VERSION already exists${NC}"
    if [ "$AUTO_YES" = false ]; then
        read -p "Continue anyway? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "Aborted."
            exit 1
        fi
    fi
fi

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

# Dry run exit point
if [ "$DRY_RUN" = true ]; then
    echo ""
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}DRY RUN - No changes made${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo "Would trigger GitHub Actions 'Manual Release' workflow with:"
    echo "  - version: $VERSION"
    echo ""
    echo "This will:"
    echo "  1. Create git tag $VERSION (if not exists)"
    echo "  2. Create GitHub release with auto-generated notes"
    echo "  3. Build binaries for all platforms"
    echo "  4. Upload binaries to the release"
    echo "  5. Update homebrew formula with SHA256 hashes"
    exit 0
fi

# Confirm
if [ "$AUTO_YES" = false ]; then
    echo "This will trigger GitHub Actions to:"
    echo "  1. Create tag and release $VERSION"
    echo "  2. Build binaries for all platforms"
    echo "  3. Update homebrew formula"
    echo ""
    read -p "Proceed? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

# Trigger the GitHub workflow
echo ""
echo "Triggering GitHub Actions workflow..."
gh workflow run release.yml -f version="$VERSION"

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Release workflow triggered!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "Next steps:"
echo ""
echo "  1. Watch the workflow progress:"
echo -e "     ${BLUE}gh run watch${NC}"
echo ""
echo "  2. Or view in browser:"
echo -e "     ${BLUE}gh run list --workflow=release.yml${NC}"
echo ""

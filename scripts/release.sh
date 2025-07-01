#!/bin/bash

# Release script for Lambdust
# Usage: ./scripts/release.sh <version>
# Example: ./scripts/release.sh 0.1.1

set -e

if [ $# -eq 0 ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.1"
    exit 1
fi

VERSION=$1

echo "🚀 Preparing release v$VERSION"

# Check if we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo "❌ Please switch to main branch before releasing"
    exit 1
fi

# Check if working directory is clean
if [ -n "$(git status --porcelain)" ]; then
    echo "❌ Working directory is not clean. Please commit or stash changes."
    exit 1
fi

# Update version in Cargo.toml
echo "📝 Updating version in Cargo.toml"
sed -i.bak "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Run tests
echo "🧪 Running tests"
cargo test --all-features

# Check formatting
echo "🎨 Checking code formatting"
cargo fmt --all -- --check

# Run clippy
echo "📎 Running clippy"
cargo clippy --all-features -- -D warnings

# Build documentation
echo "📚 Building documentation"
cargo doc --no-deps --all-features

# Update CHANGELOG.md (manual step reminder)
echo "📋 Please update CHANGELOG.md with the changes for v$VERSION"
echo "Press enter when ready to continue..."
read -r

# Commit version bump
echo "💾 Committing version bump"
git add Cargo.toml CHANGELOG.md
git commit -m "Bump version to $VERSION"

# Create tag
echo "🏷️  Creating tag v$VERSION"
git tag -a "v$VERSION" -m "Release v$VERSION"

# Push changes and tag
echo "📤 Pushing changes and tag to origin"
git push origin main
git push origin "v$VERSION"

echo "✅ Release v$VERSION has been prepared and pushed!"
echo "🚀 GitHub Actions will automatically publish to crates.io when the tag is detected."
echo "📦 Check the Actions tab on GitHub for deployment status."
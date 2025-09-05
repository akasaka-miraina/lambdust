#!/bin/bash
#
# Setup development git hooks for Lambdust project
#

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "🔧 Setting up git hooks for Lambdust development..."

# Install pre-push hook
if [ -f "$SCRIPT_DIR/pre-push" ]; then
    cp "$SCRIPT_DIR/pre-push" "$HOOKS_DIR/pre-push"
    chmod +x "$HOOKS_DIR/pre-push"
    echo "✅ Pre-push hook installed."
else
    echo "❌ Pre-push script not found at $SCRIPT_DIR/pre-push"
    exit 1
fi

echo "🎉 Git hooks setup complete!"
echo ""
echo "The pre-push hook will now run:"
echo "  - cargo fmt --check"
echo "  - cargo clippy --lib -- -D warnings"  
echo "  - cargo check"
echo ""
echo "This ensures code quality before pushing to remote."
echo "To bypass hooks (emergency only): git push --no-verify"
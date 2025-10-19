#!/bin/bash
# Script to install git hooks

echo "🔗 Installing git hooks..."

# Create hooks directory if it doesn't exist
mkdir -p .git/hooks

# Copy pre-commit hook
cp hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

echo "✅ Git hooks installed successfully!"
echo ""
echo "Pre-commit hook will now:"
echo "  ✓ Check code formatting (cargo fmt)"
echo "  ✓ Run clippy linter"
echo ""
echo "To skip hooks temporarily, use: git commit --no-verify"

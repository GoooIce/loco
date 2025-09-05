#!/bin/bash

# Loco DDD Release Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to confirm action
confirm() {
    read -p "$1 (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
}

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]] || [[ ! -f "src/lib.rs" ]]; then
    print_error "Please run this script from the root of the loco-ddd repository"
    exit 1
fi

# Get current version
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "//; s/"//g')
print_status "Current version: $CURRENT_VERSION"

# Ask for new version
read -p "Enter new version (current: $CURRENT_VERSION): " NEW_VERSION
if [[ -z "$NEW_VERSION" ]]; then
    NEW_VERSION=$CURRENT_VERSION
fi

# Validate version format
if [[ ! $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
    print_error "Invalid version format. Use semantic versioning (e.g., 1.0.0 or 1.0.0-beta.1)"
    exit 1
fi

print_status "New version will be: $NEW_VERSION"
confirm "Continue with release?"

# Update version in Cargo.toml
print_status "Updating version in Cargo.toml..."
sed -i.bak "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm Cargo.toml.bak

# Run tests
print_status "Running tests..."
cargo test --all-features
if [[ $? -ne 0 ]]; then
    print_error "Tests failed"
    exit 1
fi
print_success "All tests passed"

# Run clippy
print_status "Running clippy..."
cargo clippy -- -D warnings
if [[ $? -ne 0 ]]; then
    print_error "Clippy checks failed"
    exit 1
fi
print_success "Clippy checks passed"

# Check formatting
print_status "Checking code formatting..."
cargo fmt --all -- --check
if [[ $? -ne 0 ]]; then
    print_error "Code formatting issues found"
    print_status "Run 'cargo fmt' to fix formatting"
    exit 1
fi
print_success "Code formatting is correct"

# Build documentation
print_status "Building documentation..."
cargo doc --all-features --no-deps
if [[ $? -ne 0 ]]; then
    print_error "Documentation build failed"
    exit 1
fi
print_success "Documentation built successfully"

# Create git commit
print_status "Creating git commit..."
git add Cargo.toml
git commit -m "Release version $NEW_VERSION"

# Create git tag
print_status "Creating git tag..."
git tag -a "v$NEW_VERSION" -m "Version $NEW_VERSION"

# Push changes and tag
print_status "Pushing changes to remote..."
git push origin main
git push origin "v$NEW_VERSION"

# Publish to crates.io
print_status "Publishing to crates.io..."
cargo publish --no-verify

# Wait for crates.io to process
print_status "Waiting for crates.io to process..."
sleep 30

# Verify publication
print_status "Verifying publication..."
cargo install --force --git "https://github.com/your-org/loco-ddd" --tag "v$NEW_VERSION" loco-ddd
if [[ $? -eq 0 ]]; then
    print_success "Package successfully published to crates.io"
else
    print_error "Failed to verify publication"
    exit 1
fi

# Create release notes
print_status "Creating release notes..."
cat > "release_notes_v$NEW_VERSION.md" << EOF
# Loco DDD v$NEW_VERSION Release Notes

## What's Changed

- 🚀 New features and improvements
- 🐛 Bug fixes
- 📚 Documentation updates
- 🔧 Internal improvements

## Installation

\`\`\`toml
[dependencies]
loco-ddd = "$NEW_VERSION"
\`\`\`

## Documentation

- [API Documentation](https://docs.rs/loco-ddd/$NEW_VERSION)
- [Examples](https://github.com/your-org/loco-ddd/tree/v$NEW_VERSION/examples)

## Contributors

Thanks to all contributors who made this release possible!

---

*Released on $(date +%Y-%m-%d)*
EOF

print_success "Release notes created: release_notes_v$NEW_VERSION.md"

print_success "🎉 Release v$NEW_VERSION completed successfully!"
print_status "Don't forget to:"
print_status "1. Create a GitHub release with the release notes"
print_status "2. Update documentation website"
print_status "3. Announce the release in appropriate channels"
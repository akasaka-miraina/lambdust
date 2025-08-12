#!/bin/bash
# Setup script for libffi on macOS ARM64 to resolve build issues
#
# This script installs libffi via Homebrew and sets up the necessary
# environment variables to use the system libffi instead of building
# from source, which resolves ARM64 CFI assembly errors.

set -e

echo "Setting up libffi for macOS ARM64..."

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew is not installed. Please install Homebrew first:"
    echo "   /bin/bash -c \"\$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)\""
    exit 1
fi

# Install libffi if not already installed
if ! brew list libffi &>/dev/null; then
    echo "📦 Installing libffi via Homebrew..."
    brew install libffi
else
    echo "✅ libffi is already installed"
fi

# Get libffi paths
LIBFFI_PREFIX=$(brew --prefix libffi)

# Export environment variables
echo "🔧 Setting up environment variables..."

cat << EOF
Add these environment variables to your shell profile (~/.bashrc, ~/.zshrc, etc.):

export PKG_CONFIG_PATH="${LIBFFI_PREFIX}/lib/pkgconfig:\$PKG_CONFIG_PATH"
export LDFLAGS="-L${LIBFFI_PREFIX}/lib"
export CPPFLAGS="-I${LIBFFI_PREFIX}/include"

Or run them in your current shell session:
EOF

echo "export PKG_CONFIG_PATH=\"${LIBFFI_PREFIX}/lib/pkgconfig:\$PKG_CONFIG_PATH\""
echo "export LDFLAGS=\"-L${LIBFFI_PREFIX}/lib\""
echo "export CPPFLAGS=\"-I${LIBFFI_PREFIX}/include\""

echo ""
echo "🧪 Testing FFI compilation..."

# Set environment variables for the test
export PKG_CONFIG_PATH="${LIBFFI_PREFIX}/lib/pkgconfig:$PKG_CONFIG_PATH"
export LDFLAGS="-L${LIBFFI_PREFIX}/lib"
export CPPFLAGS="-I${LIBFFI_PREFIX}/include"

# Test compilation
if cargo check --features=ffi --quiet; then
    echo "✅ FFI compilation test passed!"
else
    echo "❌ FFI compilation test failed. Please check the output above."
    exit 1
fi

echo ""
echo "🎉 libffi setup completed successfully!"
echo ""
echo "You can now run:"
echo "  cargo build --features=ffi"
echo "  cargo clippy --features=ffi"
echo "  cargo clippy --all-features  # (may have other unrelated issues)"
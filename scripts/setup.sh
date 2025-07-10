#!/usr/bin/env bash
# setup.sh - Set up development environment for domain-key

set -euo pipefail

echo "🚀 Setting up domain-key development environment..."

# Check Rust installation
if ! command -v rustc &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first:"
    echo "   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust is installed: $(rustc --version)"

# Install required targets
echo "📦 Installing cross-compilation targets..."

targets=(
    "wasm32-unknown-unknown"
    "aarch64-unknown-linux-gnu"
    "thumbv7em-none-eabihf"
)

for target in "${targets[@]}"; do
    echo "Installing $target..."
    if rustup target add "$target"; then
        echo "✅ $target installed"
    else
        echo "⚠️  Failed to install $target (might already be installed)"
    fi
done

# Install useful development tools
echo "🔧 Installing development tools..."

tools=(
    "cargo-audit"
    "cargo-udeps"
    "cargo-machete"
    "cargo-hack"
    "cargo-expand"
    "cargo-tarpaulin"
)

for tool in "${tools[@]}"; do
    echo "Installing $tool..."
    if cargo install "$tool" 2>/dev/null; then
        echo "✅ $tool installed"
    else
        echo "⚠️  Failed to install $tool (might already be installed)"
    fi
done

# Optional: Install cross for better cross-compilation
echo "🌐 Installing cross for cross-compilation..."
if cargo install cross 2>/dev/null; then
    echo "✅ cross installed"
else
    echo "⚠️  Failed to install cross (might already be installed)"
fi

# Optional: Install just command runner
echo "⚡ Installing just command runner..."
if cargo install just 2>/dev/null; then
    echo "✅ just installed"
else
    echo "⚠️  Failed to install just (might already be installed)"
fi

# Check platform and suggest optimal flags
echo "🔍 Checking platform configuration..."
OS=$(uname -s)
ARCH=$(uname -m)

echo "Platform: $OS $ARCH"

if [[ "$OS" == "Darwin" ]] && [[ "$ARCH" == "arm64" ]]; then
    echo "🍎 macOS ARM64 detected - for optimal GxHash performance, use:"
    echo "   export RUSTFLAGS=\"-C target-cpu=native -C target-feature=+aes,+neon\""
elif [[ "$ARCH" == "x86_64" ]]; then
    echo "💻 x86_64 detected - for optimal performance, use:"
    echo "   export RUSTFLAGS=\"-C target-cpu=native\""
fi

# Test basic compilation
echo "🧪 Testing basic compilation..."
if cargo check --quiet; then
    echo "✅ Basic compilation works"
else
    echo "❌ Basic compilation failed"
    exit 1
fi

# Test with features
echo "🧪 Testing with fast feature..."
if cargo check --features=fast --quiet; then
    echo "✅ Fast feature compilation works"
else
    echo "⚠️  Fast feature compilation failed - this might be expected on some platforms"
fi

# Test no_std
echo "🧪 Testing no_std compilation..."
if cargo check --no-default-features --features=no_std --quiet; then
    echo "✅ no_std compilation works"
else
    echo "❌ no_std compilation failed"
fi

# Test WASM
echo "🧪 Testing WASM compilation..."
if cargo check --target=wasm32-unknown-unknown --no-default-features --features=no_std --quiet; then
    echo "✅ WASM compilation works"
else
    echo "❌ WASM compilation failed"
fi

echo "🎉 Setup complete!"
echo ""
echo "📋 Next steps:"
echo "1. Run 'just' to see available commands (if just is installed)"
echo "2. Run 'cargo test' to run the test suite"
echo "3. Run 'cargo test --all-features' to test all feature combinations"
echo "4. Run 'cargo bench --features=fast' to run benchmarks (if available)"
echo ""
echo "💡 Useful commands:"
echo "- cargo check --all-features        # Check all feature combinations"
echo "- cargo test --features=fast        # Test with fast feature"
echo "- cargo doc --open --all-features   # Build and open documentation"
echo "- cargo audit                       # Check for security vulnerabilities"
echo ""
echo "🔧 Platform-specific RUSTFLAGS:"
if [[ "$OS" == "Darwin" ]] && [[ "$ARCH" == "arm64" ]]; then
    echo "export RUSTFLAGS=\"-C target-cpu=native -C target-feature=+aes,+neon\""
else
    echo "export RUSTFLAGS=\"-C target-cpu=native\""
fi
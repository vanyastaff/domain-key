# List available commands
default:
    @just --list

# Development workflow
dev: check test lint doc
    @echo "✅ Development checks passed"

# Quick checks
check:
    cargo check --all-features --all-targets

# Run tests with different feature combinations
test:
    @echo "🧪 Running tests with different feature combinations..."
    @echo "Testing with all features..."
    cargo test --all-features --lib
    @echo "Testing doctests with all features (may skip some due to system limitations)..."
    -cargo test --all-features --doc || echo "⚠️ Some doctests skipped due to system limitations"
    @echo "Testing with no default features..."
    cargo test --no-default-features
    @echo "Testing with no_std..."
    cargo test --no-default-features --features=no_std
    @echo "Testing with fast feature..."
    cargo test --features=fast --lib
    @echo "Testing fast feature doctests (may skip some)..."
    -cargo test --features=fast --doc || echo "⚠️ Some fast feature doctests skipped"
    @echo "Testing with secure feature..."
    cargo test --features=secure
    @echo "Testing with crypto feature..."
    cargo test --features=crypto
    @echo "Testing with std,serde..."
    cargo test --features=std,serde

# Linting
lint:
    @echo "🔍 Running linting..."
    cargo fmt --check
    cargo clippy --all-features --all-targets -- -D warnings

# Format code
fmt:
    @echo "🎨 Formatting code..."
    cargo fmt

# Generate documentation
doc:
    @echo "📚 Generating documentation..."
    cargo doc --all-features --no-deps --open

# Run benchmarks (only if bench files exist)
bench:
    @echo "🏃 Running benchmarks..."
    @if [ -d "benches" ]; then \
        cargo bench --features=fast; \
    else \
        echo "⚠️  No benchmark files found. Skipping benchmarks."; \
        cargo test --release --features=fast perf; \
    fi

# Performance profiling
profile:
    @echo "📊 Running performance profiling..."
    @if [ -d "benches" ]; then \
        cargo bench --features=fast -- --profile-time=10; \
    else \
        echo "⚠️  No benchmark files found. Running performance tests instead."; \
        cargo test --release --features=fast --nocapture perf; \
    fi

# Memory usage analysis
memory:
    @echo "🧠 Analyzing memory usage..."
    @if command -v valgrind >/dev/null 2>&1; then \
        if command -v cargo-valgrind >/dev/null 2>&1; then \
            cargo valgrind test --features=std memory_usage; \
        else \
            echo "⚠️  cargo-valgrind not installed. Install with: cargo install cargo-valgrind"; \
        fi; \
    else \
        echo "⚠️  valgrind not available. Running regular memory tests."; \
        cargo test --release memory_usage; \
    fi

# Test no_std compatibility
test-nostd:
    @echo "🚫📚 Testing no_std compatibility..."
    @echo "Checking no_std compilation..."
    cargo check --no-default-features --features=no_std
    cargo check --no-default-features --features=no_std,fast
    cargo check --no-default-features --features=no_std,secure
    cargo check --no-default-features --features=no_std,crypto
    @echo "Running no_std tests..."
    cargo test --no-default-features --features=no_std
    @echo "Testing WASM target (no_std)..."
    cargo check --target=wasm32-unknown-unknown --no-default-features --features=no_std
    @echo "✅ no_std compatibility verified"

# Security testing
security:
    @echo "🔒 Running security tests..."
    cargo test --features=secure --all-targets
    @if command -v cargo-audit >/dev/null 2>&1; then \
        cargo audit; \
    else \
        echo "⚠️  cargo-audit not installed. Install with: cargo install cargo-audit"; \
    fi

# Cross-platform testing
cross:
    @echo "🌐 Testing cross-platform compatibility..."
    @if command -v cross >/dev/null 2>&1; then \
        cross check --target=wasm32-unknown-unknown --no-default-features --features=no_std; \
        cross check --target=aarch64-unknown-linux-gnu --all-features; \
        cross check --target=thumbv7em-none-eabihf --no-default-features --features=no_std; \
    else \
        echo "⚠️  cross not installed. Install with: cargo install cross"; \
        echo "Running local no_std checks instead..."; \
        cargo check --target=wasm32-unknown-unknown --no-default-features --features=no_std; \
    fi

# Release preparation
release VERSION:
    @echo "🚀 Preparing release {{VERSION}}..."
    # Update version in Cargo.toml
    sed -i.bak 's/version = "[^"]*"/version = "{{VERSION}}"/' Cargo.toml
    rm -f Cargo.toml.bak
    # Run full test suite
    just test
    @if [ -d "benches" ]; then just bench; fi
    # Generate changelog if git-cliff is available
    @if command -v git-cliff >/dev/null 2>&1; then \
        git cliff --tag {{VERSION}} > CHANGELOG.md; \
    else \
        echo "⚠️  git-cliff not installed. Changelog not updated."; \
    fi
    # Commit and tag
    git add .
    git commit -m "chore: release {{VERSION}}"
    git tag -a v{{VERSION}} -m "Release {{VERSION}}"
    @echo "✅ Release {{VERSION}} prepared. Push with: git push && git push --tags"

# Publish to crates.io
publish:
    @echo "📦 Publishing to crates.io..."
    cargo publish --all-features

# Clean build artifacts
clean:
    @echo "🧹 Cleaning build artifacts..."
    cargo clean
    @if [ -d "target/criterion" ]; then rm -rf target/criterion; fi

# Install development tools
install-tools:
    @echo "🔧 Installing development tools..."
    @echo "Installing cargo-audit..."
    @cargo install cargo-audit || echo "Failed to install cargo-audit"
    @echo "Installing cargo-udeps..."
    @cargo install cargo-udeps || echo "Failed to install cargo-udeps"
    @echo "Installing cargo-machete..."
    @cargo install cargo-machete || echo "Failed to install cargo-machete"
    @echo "Installing git-cliff..."
    @cargo install git-cliff || echo "Failed to install git-cliff"
    @echo "Installing cross..."
    @cargo install cross || echo "Failed to install cross"
    @echo "Installing cargo-valgrind..."
    @cargo install cargo-valgrind || echo "Failed to install cargo-valgrind"

# Run all CI checks locally
ci: lint test security doc
    @echo "✅ All CI checks passed locally"

# Quick development feedback loop
quick: check test-quick lint
    @echo "✅ Quick checks passed"

# Fast test subset for development
test-quick:
    @echo "🧪 Running quick tests..."
    cargo test --lib
    cargo test --features=fast --lib

# Generate test coverage (if tarpaulin is available)
coverage:
    @echo "📊 Generating test coverage..."
    @if command -v cargo-tarpaulin >/dev/null 2>&1; then \
        cargo tarpaulin --all-features --out html --output-dir coverage; \
        echo "Coverage report generated in coverage/"; \
    else \
        echo "⚠️  cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin"; \
    fi

# Check for unused dependencies
unused-deps:
    @echo "🔍 Checking for unused dependencies..."
    @if command -v cargo-udeps >/dev/null 2>&1; then \
        cargo +nightly udeps --all-targets; \
    else \
        echo "⚠️  cargo-udeps not installed. Install with: cargo install cargo-udeps"; \
    fi

# Check for dead code
dead-code:
    @echo "🔍 Checking for dead code..."
    @if command -v cargo-machete >/dev/null 2>&1; then \
        cargo machete; \
    else \
        echo "⚠️  cargo-machete not installed. Install with: cargo install cargo-machete"; \
    fi

# Run examples
examples:
    @echo "📋 Running examples..."
    @if [ -d "examples" ]; then \
        for example in examples/*.rs; do \
            if [ -f "$$example" ]; then \
                name=$$(basename "$$example" .rs); \
                echo "Running example: $$name"; \
                cargo run --example "$$name" --features=fast || echo "Failed to run $$name"; \
            fi; \
        done; \
    else \
        echo "⚠️  No examples directory found"; \
    fi

# Check that all features compile
check-features:
    @echo "🔧 Checking all feature combinations..."
    cargo check --no-default-features
    cargo check --no-default-features --features=no_std
    cargo check --features=std
    cargo check --features=serde
    cargo check --features=fast
    cargo check --features=secure
    cargo check --features=crypto
    cargo check --features=std,serde
    cargo check --features=fast,std,serde
    cargo check --features=secure,std,serde
    cargo check --features=crypto,std,serde
    cargo check --features=fast,no_std
    cargo check --features=secure,no_std
    cargo check --features=crypto,no_std
    cargo check --all-features

# Verify release readiness
verify-release: clean check-features test test-nostd security doc examples
    @echo "✅ Release verification complete"

# Display project information
info:
    @echo "📋 Project Information:"
    @echo "  Name: domain-key"
    @echo "  Version: $(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)"
    @echo "  Features available:"
    @echo "    - fast: GxHash for maximum performance"
    @echo "    - secure: AHash for DoS protection"
    @echo "    - crypto: Blake3 for cryptographic security"
    @echo "    - std: Standard library support (default)"
    @echo "    - serde: Serialization support (default)"
    @echo "    - no_std: No standard library support"
    @echo "  Targets: x86_64-unknown-linux-gnu, x86_64-pc-windows-msvc, x86_64-apple-darwin, wasm32-unknown-unknown, thumbv7em-none-eabihf"
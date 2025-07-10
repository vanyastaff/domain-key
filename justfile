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
    cargo test --all-features
    cargo test --no-default-features --features=no_std
    cargo test --features=max-performance
    cargo test --features=security
    cargo test --features=cryptographic

# Linting
lint:
    cargo fmt --check
    cargo clippy --all-features --all-targets -- -D warnings

# Format code
fmt:
    cargo fmt

# Generate documentation
doc:
    cargo doc --all-features --no-deps --open

# Run benchmarks
bench:
    cargo bench --features=max-performance

# Performance profiling
profile:
    cargo bench --features=max-performance -- --profile-time=10

# Memory usage analysis
memory:
    cargo bench --bench memory_benchmarks --features=max-performance

# Security testing
security:
    cargo test --features=security --all-targets
    cargo audit

# Cross-platform testing
cross:
    cross check --target=wasm32-unknown-unknown --no-default-features --features=no_std
    cross check --target=aarch64-unknown-linux-gnu --all-features

# Release preparation
release VERSION:
    # Update version in Cargo.toml
    sed -i 's/version = ".*"/version = "{{VERSION}}"/' Cargo.toml
    # Run full test suite
    just test
    just bench
    # Generate changelog
    git cliff --tag {{VERSION}} > CHANGELOG.md
    # Commit and tag
    git add .
    git commit -m "chore: release {{VERSION}}"
    git tag -a v{{VERSION}} -m "Release {{VERSION}}"

# Publish to crates.io
publish:
    cargo publish --all-features

# Clean build artifacts
clean:
    cargo clean
    rm -rf target/criterion

# Install development tools
install-tools:
    cargo install cargo-audit
    cargo install cargo-udeps
    cargo install cargo-machete
    cargo install git-cliff
    cargo install cross
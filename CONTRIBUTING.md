# Contributing to domain-key

We love your input! We want to make contributing to domain-key as easy and transparent as possible, whether it's:

- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features
- Becoming a maintainer

## 🚀 Quick Start

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Create a branch** for your changes
4. **Make your changes** and test them
5. **Submit a pull request**

## 📋 Development Process

We use GitHub to host code, to track issues and feature requests, as well as accept pull requests.

### Pull Requests

1. Fork the repo and create your branch from `main`
2. If you've added code that should be tested, add tests
3. If you've changed APIs, update the documentation
4. Ensure the test suite passes
5. Make sure your code follows the existing style
6. Issue that pull request!

## 🧪 Testing

We take testing seriously! Please ensure your changes include appropriate tests:

```bash
# Run all tests
cargo test --all-features

# Run specific test categories
cargo test --lib                    # Unit tests
cargo test --test integration       # Integration tests
cargo test --doc                    # Documentation tests

# Run property-based tests
cargo test -- prop_

# Run with different feature combinations
cargo test --no-default-features
cargo test --features fast
cargo test --features secure
cargo test --features max-performance
```

### Testing Guidelines

- **Unit tests**: Test individual functions and methods
- **Integration tests**: Test the library as users would use it
- **Property tests**: Use proptest for testing invariants
- **Documentation tests**: Ensure all code examples work
- **Benchmark tests**: Verify performance claims

## 🎨 Code Style

We use standard Rust formatting and linting:

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-features -- -D warnings

# Check documentation
cargo doc --all-features --no-deps
```

### Style Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Ensure `cargo clippy` passes with no warnings
- Write clear, descriptive commit messages
- Add documentation for public APIs
- Use meaningful variable and function names

## 📝 Documentation

Good documentation is crucial:

- **Public APIs**: Must have documentation comments
- **Examples**: Include usage examples in docs
- **README**: Keep the main README up to date
- **Changelog**: Update CHANGELOG.md for user-facing changes
- **Comments**: Use comments for complex logic

```rust
/// Creates a new key with validation
///
/// # Arguments
///
/// * `key` - The key string to validate and store
///
/// # Errors
///
/// Returns `KeyParseError` if the key fails validation
///
/// # Examples
///
/// ```rust
/// use domain_key::{Key, KeyDomain};
/// 
/// #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// struct TestDomain;
/// 
/// impl KeyDomain for TestDomain {
///     const DOMAIN_NAME: &'static str = "test";
/// }
/// 
/// type TestKey = Key<TestDomain>;
/// 
/// let key = TestKey::new("valid_key")?;
/// assert_eq!(key.as_str(), "valid_key");
/// # Ok::<(), domain_key::KeyParseError>(())
/// ```
pub fn new(key: impl AsRef<str>) -> Result<Self, KeyParseError> {
    // Implementation
}
```

## 🐛 Bug Reports

We use GitHub issues to track public bugs. Report a bug by [opening a new issue](https://github.com/vanyastaff/domain-key/issues/new).

**Great Bug Reports** tend to have:

- A quick summary and/or background
- Steps to reproduce
    - Be specific!
    - Give sample code if you can
- What you expected would happen
- What actually happens
- Notes (possibly including why you think this might be happening, or stuff you tried that didn't work)

### Bug Report Template

```markdown
**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Create a key with domain '...'
2. Call method '...'
3. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Code sample**
```rust
// Minimal reproducible example
```

**Environment:**
- OS: [e.g. Linux, Windows, macOS]
- Rust version: [e.g. 1.70]
- domain-key version: [e.g. 0.1.0]
- Features enabled: [e.g. max-performance]

**Additional context**
Add any other context about the problem here.
```

## 💡 Feature Requests

We welcome feature requests! Please:

1. **Check existing issues** to avoid duplicates
2. **Describe the motivation** - what problem does it solve?
3. **Provide examples** of how it would be used
4. **Consider backwards compatibility**

### Feature Request Template

```markdown
**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
Alternative solutions or features you've considered.

**Additional context**
Any other context, code examples, or screenshots.
```

## 🔧 Development Setup

### Prerequisites

- Rust 1.70 or later
- Git

### Setup

```bash
# Clone your fork
git clone https://github.com/yourusername/domain-key.git
cd domain-key

# Create a branch for your work
git checkout -b feature/my-new-feature

# Install development tools (optional but recommended)
cargo install cargo-tarpaulin  # Code coverage
cargo install cargo-audit      # Security audit
cargo install cargo-insta      # Snapshot testing
```

### Useful Commands

```bash
# Run all checks (what CI runs)
./scripts/check-all.sh  # If available

# Or manually:
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --all-features
cargo doc --all-features --no-deps
```

## 📊 Performance Guidelines

domain-key is a performance-focused library:

- **Benchmark changes**: Use `cargo bench` to verify performance
- **No regressions**: New features shouldn't slow down existing code
- **Memory efficiency**: Be mindful of allocations
- **Profile when needed**: Use profiling tools for complex optimizations

```bash
# Run benchmarks
cargo bench --features max-performance

# Compare performance
git checkout main
cargo bench --features max-performance > baseline.txt
git checkout feature-branch
cargo bench --features max-performance > feature.txt
# Compare baseline.txt and feature.txt
```

## 📜 License

By contributing, you agree that your contributions will be licensed under the same license as the project (MIT License).

## 🤝 Code of Conduct

### Our Pledge

We pledge to make participation in our project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, education, socio-economic status, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

Examples of behavior that contributes to creating a positive environment include:

- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported by contacting the project team. All complaints will be reviewed and investigated and will result in a response that is deemed necessary and appropriate to the circumstances.

## 🎯 Areas for Contribution

We especially welcome contributions in these areas:

### 🔧 Core Library
- Performance optimizations
- New hash algorithms
- Memory usage improvements
- no_std compatibility

### 📚 Documentation
- Usage examples
- Tutorial improvements
- API documentation
- Performance guides

### 🧪 Testing
- Property-based tests
- Edge case coverage
- Performance regression tests
- Platform compatibility tests

### 🌟 Ecosystem
- Integration examples
- Framework adapters
- Tool integrations
- Real-world case studies

## 📞 Getting Help

- **GitHub Issues**: For bugs and feature requests
- **GitHub Discussions**: For questions and general discussion
- **Documentation**: Check [docs.rs/domain-key](https://docs.rs/domain-key)

## 🙏 Recognition

Contributors will be:
- Listed in the project's README
- Mentioned in release notes for significant contributions
- Invited to be maintainers for substantial ongoing contributions

Thank you for contributing to domain-key! 🚀
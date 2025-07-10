# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project setup
- Core domain-key functionality
- Type-safe key system with domain separation
- Performance optimizations for key operations
- Feature flags for different use cases (fast, secure, crypto)
- Comprehensive test suite setup
- Documentation and examples structure

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

## [0.1.0] - 2024-XX-XX

### Added
- Initial release of domain-key library
- Domain-driven key system with compile-time type safety
- High-performance key operations with 15-40% improvements
- Memory-efficient string handling with SmartString
- Optional DoS protection with different hash algorithms
- Extensible validation and normalization per domain
- Zero-cost abstractions for domain separation
- Comprehensive feature flags:
    - `fast` - GxHash for maximum performance
    - `secure` - AHash for DoS protection
    - `crypto` - Blake3 for cryptographic security
    - `max-performance` - All performance optimizations
    - `security` - Security-focused configuration
- Support for custom domain validation rules
- Support for custom domain normalization
- Serialization support via serde (optional)
- no_std support (optional)
- Extensive documentation and examples
- Property-based testing
- Comprehensive benchmark suite

---

## Release Template

When creating a new release, use this template:

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New features

### Changed  
- Changes in existing functionality

### Deprecated
- Soon-to-be removed features

### Removed
- Now removed features

### Fixed
- Bug fixes

### Security
- Security improvements
```

## Version Numbering

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version when you make incompatible API changes
- **MINOR** version when you add functionality in a backwards compatible manner
- **PATCH** version when you make backwards compatible bug fixes

### Pre-1.0 Versioning

During the 0.x series:
- **0.MINOR.PATCH** where MINOR changes may include breaking changes
- Breaking changes will be clearly documented in the changelog
- API stability is not guaranteed until 1.0.0

## Links

- [Repository](https://github.com/vanyastaff/domain-key)
- [Crates.io](https://crates.io/crates/domain-key)
- [Documentation](https://docs.rs/domain-key)
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2025-01-20

### Added
- **`Id<D>` typed numeric identifier**: Lightweight `NonZeroU64` wrapper with domain typing (8 bytes, `Copy`, niche-optimized `Option`)
- **`Uuid<D>` typed UUID identifier**: `uuid::Uuid` wrapper with domain typing (16 bytes, `Copy`), behind `uuid` feature flag
- **`Domain` supertrait**: New base trait holding `DOMAIN_NAME`, shared by `KeyDomain`, `IdDomain`, and `UuidDomain`
- **`IdDomain` / `UuidDomain` marker traits**: Lightweight domain markers for `Id<D>` and `Uuid<D>`
- **Combined macros**: `define_id!`, `define_uuid!` for one-liner domain + type alias definition
- **Domain macros**: `define_id_domain!`, `define_uuid_domain!`, `id_type!`, `uuid_type!`
- **`stringify!` shorthand**: `define_id_domain!(MyDomain)` without explicit name string
- **UUID feature flags**: `uuid`, `uuid-v4`, `uuid-v7` for granular UUID support
- **Identifier Types** section in crate-level documentation

### Changed
- **`KeyDomain` no longer requires `PartialEq + Eq + Hash + Ord + PartialOrd`** — manual trait impls on `Key<T>` removed these bounds
- **`Key<T>` uses manual `PartialEq`/`Eq`/`PartialOrd`/`Ord`** — compares only the inner string, fixing Hash/Eq contract violation
- **`Key<T>::Display` outputs value only** — was `"domain:value"`, now just the value (consistent with `AsRef<str>` and serde)
- **Validation runs on normalized string** — normalize-before-validate ordering fix
- **`Id<D>` and `Uuid<D>` have domain-aware `Debug`** — prints `user(42)` instead of `Id { value: 42, _marker: PhantomData<...> }`
- **`Uuid<D>` serde delegates to `uuid`'s own impl** — zero-alloc serialization, correct deserialization for all formats

### Removed
- **`PerformanceInfo`**, **`performance_info()`**, **`analyze_current_configuration()`** and other diagnostic bloat from `features.rs`
- **`features` module** — `hash_algorithm()` moved to `utils` module (re-exported at crate root, public API unchanged)

## [0.3.1] - 2026-03-02

### Fixed
- **no_std**: `String` and test helpers (`format!`, `ToString`) in `id.rs` and `uuid.rs` for builds without `std`
- **Rust 1.75**: Replaced `#[expect(...)]` with `#[allow(...)]` (lint reasons not stabilized on 1.75)
- **Dependencies**: Pinned `blake3` &lt;1.8.3 and `uuid` &lt;1.21 to avoid `edition2024`/`getrandom` 0.4 (Rust 1.85+)
- **CI**: Portable RUSTFLAGS (`+aes,+sse2` / `+aes,+neon`), Windows env fix, docs workflow gxhash flags

## [0.3.0] - 2026-03-01

### Added
- Unified UUID identifier API with `Uuid::<D>::new()` as the primary random constructor (requires `uuid-v4` feature)

### Changed
- `Uuid<D>` internals now construct from `uuid::Uuid` directly rather than through `Uuid::new(uuid)` helper

### Deprecated
- `Uuid::<D>::v4()` in favor of `Uuid::<D>::new()`; `v4()` remains as a deprecated alias for this release
- Inherent `Uuid::<D>::new(uuid::Uuid)` constructor has been removed in favor of the existing `From<uuid::Uuid> for Uuid<D>` implementation

### Breaking
- Code that previously called `Uuid::<D>::new(uuid)` must migrate to `Uuid::<D>::from(uuid)` or `uuid.into()` and, for random generation, to `Uuid::<D>::new()`

## [0.1.1] - 2025-01-10

### Fixed
- **docs.rs build failure**: Fixed compilation issues on docs.rs by avoiding gxhash dependency
- **Feature configuration**: Changed docs.rs metadata from `all-features = true` to specific features `["std", "serde"]`
- **Platform compatibility**: Resolved AES+SSE2 CPU instruction requirements that caused build failures on docs.rs environment
- **Documentation generation**: Ensured documentation builds successfully on docs.rs infrastructure

### Technical Details
- docs.rs now uses `std` and `serde` features instead of `fast` feature to avoid gxhash
- gxhash requires AES and SSE2 CPU instructions not available in docs.rs build environment
- Local builds with `fast` feature continue to work with proper RUSTFLAGS configuration
- No functional changes to the library itself

## [0.1.0] - 2025-01-20

### Added
- **Core domain-key functionality** with type-safe key system
- **Domain-driven design** approach with compile-time type safety
- **High-performance optimizations** with up to 75% performance improvements
- **Multiple hash algorithm support**:
  - `fast` feature: GxHash for maximum performance (40% faster hash operations)
  - `secure` feature: AHash for DoS protection
  - `crypto` feature: Blake3 for cryptographic security
  - Fallback to standard library hasher or FNV-1a for compatibility
- **Memory efficiency** with SmartString for optimal allocation:
  - Stack allocation for keys ≤23 characters
  - Heap allocation only when necessary
  - Cached hash and length for O(1) operations
- **Comprehensive validation system**:
  - Domain-specific validation rules
  - Custom character sets and normalization
  - Length limits and structural validation
  - Detailed error reporting with suggestions
- **Advanced key operations**:
  - Multi-part key construction with `from_parts`
  - Prefix and suffix management with `ensure_prefix`/`ensure_suffix`
  - String splitting with caching optimizations
  - Static key creation with compile-time validation
- **Cross-platform support**:
  - Full support on Linux, Windows, macOS (Intel and Apple Silicon)
  - WebAssembly compatibility with no_std
  - ARM64 Linux and embedded ARM support
  - Proper target feature detection and fallbacks
- **Extensive feature flags**:
  - `std` (default): Standard library support
  - `serde` (default): Serialization/deserialization support
  - `no_std`: No standard library support for embedded systems
  - Performance and security profiles for different use cases
- **Built-in domain types**:
  - `DefaultDomain`: General-purpose keys with sensible defaults
  - `IdentifierDomain`: Strict identifier validation (programming language compatible)
  - `PathDomain`: Hierarchical path-like keys with slash separators
- **Comprehensive macro system**:
  - `static_key!`: Compile-time validated static keys
  - `define_domain!`: Simplified domain definition
  - `key_type!`: Type alias creation
  - `batch_keys!`: Bulk key creation with error handling
  - `test_domain!`: Automated test generation for domains
- **Development tools and utilities**:
  - Performance benchmarking utilities
  - Memory usage analysis
  - Diagnostic tools for troubleshooting
  - Feature detection and configuration analysis
- **Comprehensive documentation**:
  - User guide with real-world examples
  - Migration guide from string-based keys
  - Performance optimization guide
  - Security considerations and best practices
  - API documentation with extensive examples
- **Examples and patterns**:
  - E-commerce domain modeling
  - Multi-tenant SaaS applications
  - Web application session management
  - Database key patterns
  - Cache key management

### Performance Improvements
- **28% faster** key creation for short keys through stack allocation
- **75% faster** string operations with cached length and optimized accessors
- **40% faster** hash operations with GxHash on supported platforms
- **Constant-time** length access eliminating O(n) string traversal
- **40% faster** collection lookups with cached hash values
- **29% faster** split operations with position caching

### Security Features
- **DoS attack protection** with AHash when using `secure` feature
- **Cryptographic security** with Blake3 when using `crypto` feature
- **Input validation** comprehensive pipeline preventing injection attacks
- **Type safety** preventing accidental key mixing at compile time
- **Memory safety** with no unsafe code and bounds checking
- **Length limits** preventing buffer overflow and DoS attacks

### Technical Details
- **MSRV**: Rust 1.75+
- **Memory layout**: Cache-line friendly 32-byte key structure
- **Hash algorithms**: Runtime selection based on CPU capabilities
- **Platform optimizations**: Automatic target feature detection
- **Error handling**: Comprehensive error types with recovery suggestions
- **Testing**: >95% test coverage with property-based testing
- **Benchmarks**: 20+ performance scenarios across platforms
- **Documentation**: >98% documentation coverage

### Platform-Specific Optimizations
- **x86_64**: AES-NI instruction support for GxHash
- **ARM64**: NEON and AES instruction support
- **Apple Silicon**: Explicit target feature configuration for GxHash
- **WebAssembly**: Optimized builds with size optimization
- **Embedded**: no_std support with minimal dependencies

### Breaking Changes
- None (initial release)

### Migration Notes
- This is the initial release
- See [Migration Guide](docs/migration.md) for converting from string-based keys
- All APIs are stable and follow semantic versioning

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

### Performance
- Performance improvements with measurements
```

## Version Numbering

This project follows [Semantic Versioning](https://semver.org/):

- **MAJOR** version when you make incompatible API changes
- **MINOR** version when you add functionality in a backwards compatible manner
- **PATCH** version when you make backwards compatible bug fixes

### Pre-1.0 Versioning Policy

- **Breaking changes**: Will increment MINOR version (0.x.y)
- **New features**: Will increment MINOR version (0.x.y)
- **Bug fixes**: Will increment PATCH version (0.x.y)
- **API stability**: Not guaranteed until 1.0.0
- **Migration guides**: Provided for all breaking changes

## Links

- [Repository](https://github.com/vanyastaff/domain-key)
- [Crates.io](https://crates.io/crates/domain-key)
- [Documentation](https://docs.rs/domain-key)
- [User Guide](docs/guide.md)
- [Migration Guide](docs/migration.md)
- [Performance Guide](docs/performance.md)
- [Examples](examples/)
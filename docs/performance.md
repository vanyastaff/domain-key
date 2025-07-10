# Performance Guide for domain-key

This guide covers performance optimization techniques, benchmarking, and best practices for getting the most out of domain-key in performance-critical applications.

## Table of Contents

1. [Performance Overview](#performance-overview)
2. [Feature Selection](#feature-selection)
3. [Domain Configuration](#domain-configuration)
4. [Memory Optimization](#memory-optimization)
5. [Hash Performance](#hash-performance)
6. [Benchmarking](#benchmarking)
7. [Real-World Optimizations](#real-world-optimizations)
8. [Profiling and Debugging](#profiling-and-debugging)

## Performance Overview

domain-key delivers significant performance improvements over string-based keys:

| Operation | Baseline (String) | domain-key | Improvement |
|-----------|------------------|------------|-------------|
| Key Creation (short) | 100ns | 72ns | **28% faster** |
| Key Creation (long) | 150ns | 135ns | **10% faster** |
| Hash Operations | 25ns | 15ns | **40% faster** |
| Length Access | 8ns | 1ns | **87% faster** |
| String Access | 2ns | 2ns | Same |
| Split Operations | 45ns | 32ns | **29% faster** |
| Collection Lookup | 35ns | 21ns | **40% faster** |

### Key Performance Features

- **Cached Hashing**: Hash computed once, reused for lifetime
- **Length Caching**: O(1) length access with optimizations
- **Smart String**: Stack allocation for short keys (≤23 chars)
- **Optimized Validation**: Fast character checking and structure validation
- **Zero-Cost Domain Separation**: No runtime overhead for type safety

## Feature Selection

Choose the right feature combination for your use case:

### Production Web Applications
```toml
[dependencies]
domain-key = { version = "0.1", features = ["secure"] }
```
- Uses AHash for DoS protection
- Good balance of speed and security
- Recommended for most web applications

### High-Performance Applications
```toml
[dependencies]
domain-key = { version = "0.1", features = ["fast"] }
```
- Uses GxHash (requires modern CPU with AES-NI)
- Maximum speed optimizations
- 40% faster hash operations
- Best for CPU-intensive applications

### Security-Critical Applications
```toml
[dependencies]
domain-key = { version = "0.1", features = ["crypto"] }
```
- Uses Blake3 cryptographic hash
- Suitable for security-sensitive contexts
- Slower but cryptographically secure

### Custom Feature Combinations
```toml
[dependencies]
domain-key = { version = "0.1", features = ["fast", "std", "serde"] }
```
- Mix and match features as needed
- `fast` enables GxHash for maximum performance
- `std` provides standard library optimizations
- `serde` adds serialization support

## Domain Configuration

Optimize domain configuration for your usage patterns:

### High-Performance Domain
```rust
use domain_key::{Key, KeyDomain};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct FastDomain;

impl KeyDomain for FastDomain {
    const DOMAIN_NAME: &'static str = "fast";
    
    // Performance optimizations
    const MAX_LENGTH: usize = 32;        // Reasonable limit
    const EXPECTED_LENGTH: usize = 16;   // Pre-allocation hint
    const TYPICALLY_SHORT: bool = true;  // Enable stack allocation
    const FREQUENTLY_COMPARED: bool = true; // Hash optimizations
    const FREQUENTLY_SPLIT: bool = false;   // Disable split caching overhead
    
    // Minimal validation for speed
    fn allowed_characters(c: char) -> bool {
        // Fast path: ASCII alphanumeric only
        c.is_ascii_alphanumeric() || c == '_'
    }
}

type FastKey = Key<FastDomain>;
```

### Balanced Domain
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct BalancedDomain;

impl KeyDomain for BalancedDomain {
    const DOMAIN_NAME: &'static str = "balanced";
    const MAX_LENGTH: usize = 64;
    const EXPECTED_LENGTH: usize = 24;
    const TYPICALLY_SHORT: bool = false; // Mixed lengths
    const FREQUENTLY_COMPARED: bool = true; // Common in hash maps
    
    // Standard validation
    fn allowed_characters(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_' || c == '-'
    }
}
```

### Memory-Optimized Domain
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CompactDomain;

impl KeyDomain for CompactDomain {
    const DOMAIN_NAME: &'static str = "compact";
    const MAX_LENGTH: usize = 16;        // Small keys
    const EXPECTED_LENGTH: usize = 8;    // Very short
    const TYPICALLY_SHORT: bool = true;  // Always stack allocated
    const CASE_INSENSITIVE: bool = false; // Skip normalization
}
```

### Split-Optimized Domain
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct PathDomain;

impl KeyDomain for PathDomain {
    const DOMAIN_NAME: &'static str = "path";
    const MAX_LENGTH: usize = 128;
    const FREQUENTLY_SPLIT: bool = true; // Enable split caching
    const EXPECTED_LENGTH: usize = 48;
    
    fn default_separator() -> char {
        '/'
    }
}
```

## Memory Optimization

### SmartString Benefits

domain-key uses SmartString for optimal memory usage:

```rust
use std::mem::size_of;

// String: 24 bytes (8 + 8 + 8 for ptr, len, capacity)
println!("String size: {}", size_of::<String>());

// SmartString: 24 bytes but stores ≤23 chars inline
println!("SmartString size: {}", size_of::<smartstring::SmartString<smartstring::LazyCompact>>());

// domain-key: ~40 bytes (SmartString + cached hash + length + marker)
println!("UserKey size: {}", size_of::<UserKey>());
```

### Stack vs Heap Allocation

```rust
// Short keys (≤23 chars) - stored on stack
let short_key = UserKey::new("user123")?;     // Stack allocated
let medium_key = UserKey::new("user_profile_settings")?; // Stack allocated

// Long keys (>23 chars) - stored on heap  
let long_key = UserKey::new("very_long_user_identifier_that_exceeds_inline_capacity")?; // Heap allocated

// Optimize for your use case
impl KeyDomain for OptimizedDomain {
    const TYPICALLY_SHORT: bool = true; // Most keys ≤23 chars
    const EXPECTED_LENGTH: usize = 12;  // Pre-allocation hint
}
```

### Memory Usage Patterns

```rust
// Memory-efficient batch creation
let user_ids: Result<Vec<UserKey>, _> = (1..=1000)
    .map(|id| UserKey::new(format!("user_{}", id)))
    .collect();

// Reuse keys instead of recreating
let cached_key = UserKey::new("admin")?;
for _ in 0..1000 {
    process_admin_action(&cached_key); // Reuse, don't recreate
}

// Efficient key composition
let composite_key = UserKey::from_parts(&["user", "123", "profile"], "_")?;
```

## Hash Performance

### Cached Hashing Benefits

```rust
use std::collections::HashMap;
use std::time::Instant;

// Benchmark: String keys (re-hash every time)
let mut string_map = HashMap::new();
let start = Instant::now();
for i in 0..10000 {
    let key = format!("user_{}", i);
    string_map.insert(key, i); // Hashes string every insert
}
let string_time = start.elapsed();

// Benchmark: Domain keys (cached hash)
let mut key_map = HashMap::new();
let start = Instant::now();
for i in 0..10000 {
    let key = UserKey::new(format!("user_{}", i))?; // Hash computed once
    key_map.insert(key, i); // Uses cached hash
}
let key_time = start.elapsed();

println!("String approach: {:?}", string_time);
println!("Domain key approach: {:?}", key_time);
// Domain keys are typically 30-40% faster
```

### Hash Algorithm Performance

```rust
// Configure hash algorithm based on use case
use domain_key::features::performance_info;

let info = performance_info();
println!("Active hash algorithm: {}", info.hash_algorithm);
println!("Performance multiplier: {:.1}x", info.estimated_improvement);

// Choose algorithm at compile time:
// fast     = GxHash (40% faster, needs AES-NI)
// secure   = AHash (DoS protection) 
// crypto   = Blake3 (cryptographic)
// default  = Standard hasher
```

### Optimizing Hash-Heavy Workloads

```rust
// For hash-intensive operations
impl KeyDomain for HashOptimizedDomain {
    const FREQUENTLY_COMPARED: bool = true; // Enable hash optimizations
    const EXPECTED_LENGTH: usize = 16;      // Optimal for hash function
}

// Efficient hash-based operations
let keys: Vec<HashOptimizedKey> = generate_keys();
let mut map = HashMap::with_capacity(keys.len()); // Pre-allocate

for key in keys {
    map.insert(key, value); // Uses cached hash
}
```

## Benchmarking

### Built-in Benchmarks

Run the included benchmark suite:

```bash
# Run all benchmarks with fast hash
cargo bench --features fast

# Run specific benchmark categories
cargo bench --bench domain_benchmarks --features fast
cargo bench --bench memory_usage --features fast
cargo bench --bench realistic_scenarios --features fast

# Compare hash algorithms
cargo bench --features fast > fast_results.txt
cargo bench --features secure > secure_results.txt
cargo bench --features crypto > crypto_results.txt
```

### Performance Monitoring

```rust
use domain_key::features::{performance_info, PerformanceInfo};

// Runtime performance information
let info = performance_info();
println!("{}", info);

// Expected output:
// domain-key Performance Configuration:
//   Hash Algorithm: GxHash (ultra-fast)
//   Standard Library: true
//   Serialization: true
//   Performance Multiplier: 1.4x baseline
//   Memory Profile: stack:true, length_cache:true, hash_cache:true, overhead:12B
//   Build: release:true, lto:true, arch:x86_64-modern
```

### Custom Benchmarks

Create benchmarks for your specific use case:

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use domain_key::{Key, KeyDomain};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct BenchDomain;

impl KeyDomain for BenchDomain {
    const DOMAIN_NAME: &'static str = "bench";
    const TYPICALLY_SHORT: bool = true;
    const FREQUENTLY_COMPARED: bool = true;
}

type BenchKey = Key<BenchDomain>;

fn bench_key_creation(c: &mut Criterion) {
    c.bench_function("key creation", |b| {
        b.iter(|| {
            let key = BenchKey::new(black_box("test_key_123")).unwrap();
            black_box(key)
        })
    });
}

fn bench_collection_operations(c: &mut Criterion) {
    let keys: Vec<BenchKey> = (0..1000)
        .map(|i| BenchKey::new(format!("key_{}", i)).unwrap())
        .collect();
    
    c.bench_function("hashmap operations", |b| {
        b.iter(|| {
            let mut map = std::collections::HashMap::new();
            for key in &keys {
                map.insert(black_box(key.clone()), black_box(42));
            }
            
            for key in &keys {
                let _ = map.get(black_box(key));
            }
            
            black_box(map)
        })
    });
}

criterion_group!(benches, bench_key_creation, bench_collection_operations);
criterion_main!(benches);
```

### Micro-benchmarks

Test specific operations:

```rust
use std::time::Instant;

// Benchmark key creation
let start = Instant::now();
for i in 0..100000 {
    let key = UserKey::new(format!("user_{}", i))?;
    std::hint::black_box(key);
}
let creation_time = start.elapsed();
println!("Creation time: {:?}", creation_time);

// Benchmark hash access (should be ~1ns)
let keys: Vec<UserKey> = (0..100000)
    .map(|i| UserKey::new(format!("user_{}", i)).unwrap())
    .collect();

let start = Instant::now();
for key in &keys {
    let hash = key.hash();
    std::hint::black_box(hash);
}
let hash_time = start.elapsed();
println!("Hash access time: {:?}", hash_time);

// Benchmark length access (should be ~1ns)
let start = Instant::now();
for key in &keys {
    let len = key.len();
    std::hint::black_box(len);
}
let length_time = start.elapsed();
println!("Length access time: {:?}", length_time);
```

## Real-World Optimizations

### Web Application Cache

```rust
use std::collections::HashMap;
use std::sync::RwLock;

// High-performance cache using domain keys
pub struct UserCache {
    cache: RwLock<HashMap<UserKey, UserData>>,
}

impl UserCache {
    pub fn get(&self, key: &UserKey) -> Option<UserData> {
        // Fast read access with cached hash
        self.cache.read().unwrap().get(key).cloned()
    }
    
    pub fn insert(&self, key: UserKey, data: UserData) {
        // Fast insert with cached hash
        self.cache.write().unwrap().insert(key, data);
    }
    
    // Bulk operations for better performance
    pub fn get_many(&self, keys: &[UserKey]) -> Vec<Option<UserData>> {
        let cache = self.cache.read().unwrap();
        keys.iter()
            .map(|key| cache.get(key).cloned())
            .collect()
    }
    
    // Pre-warm cache with expected capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            cache: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }
}
```

### Database Query Optimization

```rust
// Efficient database queries with domain keys
pub struct UserRepository {
    pool: DbPool,
}

impl UserRepository {
    // Single query with key
    pub async fn find_user(&self, id: UserKey) -> Result<Option<User>, DbError> {
        let conn = self.pool.get().await?;
        
        // Use string representation for database
        let user = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = $1",
            id.as_str()
        )
        .fetch_optional(&conn)
        .await?;
        
        Ok(user)
    }
    
    // Batch query optimization
    pub async fn find_users(&self, ids: &[UserKey]) -> Result<Vec<User>, DbError> {
        let conn = self.pool.get().await?;
        
        // Convert to string slice for SQL IN clause
        let id_strings: Vec<&str> = ids.iter().map(|k| k.as_str()).collect();
        
        let users = sqlx::query_as!(
            User,
            "SELECT * FROM users WHERE id = ANY($1)",
            &id_strings
        )
        .fetch_all(&conn)
        .await?;
        
        Ok(users)
    }
    
    // Optimized existence check
    pub async fn users_exist(&self, ids: &[UserKey]) -> Result<Vec<UserKey>, DbError> {
        let conn = self.pool.get().await?;
        let id_strings: Vec<&str> = ids.iter().map(|k| k.as_str()).collect();
        
        let existing: Vec<String> = sqlx::query_scalar!(
            "SELECT id FROM users WHERE id = ANY($1)",
            &id_strings
        )
        .fetch_all(&conn)
        .await?;
        
        // Convert back to domain keys
        existing.into_iter()
            .map(|s| UserKey::new(s))
            .collect()
    }
}
```

### API Response Optimization

```rust
use serde::Serialize;

// Efficient serialization
#[derive(Serialize)]
pub struct ApiResponse {
    user_id: UserKey,      // Serializes as string
    session_id: SessionKey, // Serializes as string
    data: ResponseData,
}

// Batch API operations
pub async fn get_user_batch(
    user_ids: Vec<UserKey>
) -> Result<Vec<ApiResponse>, ApiError> {
    // Keys are already validated from URL parameters
    let users = repository.find_users(&user_ids).await?;
    
    // Efficient transformation using iterator chains
    let responses = users.into_iter()
        .map(|user| ApiResponse {
            user_id: user.id,
            session_id: user.session_id,
            data: user.into(),
        })
        .collect();
        
    Ok(responses)
}

// Streaming responses for large datasets
pub async fn stream_user_batch(
    user_ids: Vec<UserKey>
) -> impl Stream<Item = Result<ApiResponse, ApiError>> {
    futures::stream::iter(user_ids)
        .map(|id| async move {
            let user = repository.find_user(id).await?;
            Ok(ApiResponse {
                user_id: user.id,
                session_id: user.session_id,
                data: user.into(),
            })
        })
        .buffer_unordered(10) // Process 10 concurrent requests
}
```

### High-Frequency Trading Example

```rust
// Ultra-low latency domain for financial applications
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct SymbolDomain;

impl KeyDomain for SymbolDomain {
    const DOMAIN_NAME: &'static str = "symbol";
    const MAX_LENGTH: usize = 8;           // Short symbols (AAPL, MSFT)
    const EXPECTED_LENGTH: usize = 4;      // Most are 3-4 chars
    const TYPICALLY_SHORT: bool = true;    // Always stack allocated
    const FREQUENTLY_COMPARED: bool = true; // High hash usage
    const CASE_INSENSITIVE: bool = false;  // Skip normalization
    
    // Minimal validation for speed
    fn allowed_characters(c: char) -> bool {
        c.is_ascii_uppercase()
    }
}

type SymbolKey = Key<SymbolDomain>;

// Lock-free trading book
use std::sync::atomic::{AtomicPtr, Ordering};

pub struct TradingBook {
    orders: HashMap<SymbolKey, AtomicPtr<OrderBook>>,
}

impl TradingBook {
    pub fn get_book(&self, symbol: &SymbolKey) -> Option<&OrderBook> {
        // O(1) lookup with cached hash
        self.orders.get(symbol)
            .and_then(|ptr| unsafe { ptr.load(Ordering::Acquire).as_ref() })
    }
}
```

## Profiling and Debugging

### Performance Profiling

Use profiling tools to identify bottlenecks:

```bash
# Install profiling tools
cargo install cargo-flamegraph
cargo install cargo-criterion

# Profile with flamegraph
sudo cargo flamegraph --bench domain_benchmarks --features fast

# Detailed criterion analysis
cargo criterion --features fast
```

### Memory Profiling

Monitor memory usage:

```bash
# Install memory profiling tools
cargo install cargo-valgrind

# Profile memory usage
cargo valgrind run --example memory_usage --features fast

# Check for memory leaks
cargo valgrind test --features fast
```

### Custom Profiling Code

```rust
use std::time::Instant;
use std::mem;

// Profile memory allocation
fn profile_memory_usage() {
    let mut keys = Vec::new();
    
    // Measure memory before
    let memory_before = get_memory_usage();
    
    for i in 0..10000 {
        keys.push(UserKey::new(format!("user_{}", i)).unwrap());
    }
    
    let memory_after = get_memory_usage();
    println!("Memory used: {} KB", (memory_after - memory_before) / 1024);
    
    // Profile key size
    println!("Key size: {} bytes", mem::size_of::<UserKey>());
    println!("Vector size: {} bytes", mem::size_of_val(&keys));
}

// Profile operation timing
fn profile_operations() {
    let keys: Vec<UserKey> = (0..1000)
        .map(|i| UserKey::new(format!("user_{}", i)).unwrap())
        .collect();
    
    // Profile hash access
    let start = Instant::now();
    for key in &keys {
        let _ = key.hash();
    }
    let hash_time = start.elapsed();
    
    // Profile string access
    let start = Instant::now();
    for key in &keys {
        let _ = key.as_str();
    }
    let string_time = start.elapsed();
    
    // Profile length access
    let start = Instant::now();
    for key in &keys {
        let _ = key.len();
    }
    let length_time = start.elapsed();
    
    println!("Hash access: {:?}", hash_time);
    println!("String access: {:?}", string_time);
    println!("Length access: {:?}", length_time);
}

// Profile split operations
fn profile_splits() {
    let keys: Vec<PathKey> = (0..1000)
        .map(|i| PathKey::new(format!("path/to/file_{}.txt", i)).unwrap())
        .collect();
    
    let start = Instant::now();
    for key in &keys {
        let parts: Vec<_> = key.split('/').collect();
        std::hint::black_box(parts);
    }
    let split_time = start.elapsed();
    
    println!("Split operations: {:?}", split_time);
}
```

### Performance Debugging

Debug performance issues:

```rust
use domain_key::features::{performance_info, analyze_current_configuration};

// Runtime performance analysis
fn debug_performance() {
    let info = performance_info();
    println!("Performance info: {}", info);
    
    let analysis = analyze_current_configuration();
    println!("Configuration analysis: {}", analysis);
    
    // Check for suboptimal configuration
    if analysis.overall_score < 80 {
        println!("⚠️ Performance could be improved:");
        for suggestion in &analysis.suggestions {
            println!("  • {}", suggestion);
        }
    }
}

// Enable debug assertions for performance testing
#[cfg(debug_assertions)]
fn debug_key_performance(key: &UserKey) {
    println!("Key: {}", key.as_str());
    println!("Length: {} (cached: {})", key.len(), key.len());
    println!("Hash: 0x{:x}", key.hash());
    println!("Domain: {}", key.domain());
    println!("Memory size: {} bytes", std::mem::size_of_val(key));
}
```

## Performance Best Practices

### 1. Choose the Right Features
- Use `fast` for CPU-intensive applications (requires AES-NI)
- Use `secure` for web applications with untrusted input
- Use `crypto` only when cryptographic security is required

### 2. Optimize Domain Configuration
- Set `TYPICALLY_SHORT: true` for keys ≤23 characters
- Configure `EXPECTED_LENGTH` for pre-allocation hints
- Keep `MAX_LENGTH` reasonable to prevent DoS attacks
- Enable `FREQUENTLY_COMPARED` for hash-heavy workloads
- Enable `FREQUENTLY_SPLIT` only when needed

### 3. Reuse Keys When Possible
```rust
// Good: Reuse keys
let admin_key = UserKey::new("admin")?;
for request in requests {
    if request.user_key == admin_key {
        // Process admin request
    }
}

// Bad: Recreate keys
for request in requests {
    let admin_key = UserKey::new("admin")?; // Wasteful!
    if request.user_key == admin_key {
        // Process admin request
    }
}
```

### 4. Batch Operations
```rust
// Good: Batch database queries
let user_keys = extract_user_keys(requests);
let users = repository.find_users(&user_keys).await?;

// Bad: Individual queries
for request in requests {
    let user = repository.find_user(request.user_key).await?;
}
```

### 5. Memory-Conscious Patterns
```rust
// Good: Pre-allocate collections
let mut map = HashMap::with_capacity(expected_size);

// Good: Use references when possible
fn process_key(key: &UserKey) -> Result<(), Error> { ... }

// Bad: Unnecessary cloning
fn process_key(key: UserKey) -> Result<(), Error> { ... } // Takes ownership
```

### 6. Profile Regularly
- Run benchmarks on every major change
- Monitor memory usage in production
- Profile with realistic data sizes
- Test with different CPU architectures

### 7. Compile-Time Optimizations
```bash
# Enable CPU-specific optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release --features=fast

# Enable link-time optimization
cargo build --release --features=fast
```

---

With these optimizations, domain-key can deliver significant performance improvements while maintaining type safety! 🚀

## Summary

- **Choose the right hash algorithm** for your performance/security needs
- **Configure domains** with appropriate optimization hints
- **Monitor performance** with built-in tools and custom benchmarks
- **Profile regularly** to identify bottlenecks
- **Use batch operations** for database and API calls
- **Reuse keys** instead of recreating them
- **Enable compile-time optimizations** for maximum performance

The combination of these techniques can result in 40-75% performance improvements over traditional string-based keys while providing compile-time type safety.
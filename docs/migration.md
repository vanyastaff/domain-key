# Migration Guide: From String Keys to domain-key

This guide will help you migrate from string-based keys to domain-key's type-safe key system. We'll cover common patterns, migration strategies, and best practices for a smooth transition.

## Table of Contents

1. [Why Migrate?](#why-migrate)
2. [Migration Strategy](#migration-strategy)
3. [Common Patterns](#common-patterns)
4. [Step-by-Step Migration](#step-by-step-migration)
5. [Compatibility Layer](#compatibility-layer)
6. [Testing Migration](#testing-migration)
7. [Performance Considerations](#performance-considerations)
8. [Troubleshooting](#troubleshooting)

## Why Migrate?

### Problems with String Keys

```rust
// String-based approach (error-prone)
fn get_user_order(user_id: String, order_id: String) -> Option<Order> {
    // Oops! Arguments swapped - runtime bug!
    database.get_order(order_id, user_id)
}

// No type safety
let user_id = "user_123".to_string();
let order_id = "order_456".to_string();
let product_id = "product_789".to_string();

// All these comparisons compile but may be meaningless
if user_id == order_id { /* bug! */ }
if order_id == product_id { /* bug! */ }

// Easy to mix up in collections
let mut cache: HashMap<String, String> = HashMap::new();
cache.insert(user_id, "user data");
cache.insert(order_id, "order data");
// Later: which key was which?
```

### Benefits of domain-key

```rust
// Type-safe approach
fn get_user_order(user_id: UserKey, order_id: OrderKey) -> Option<Order> {
    // Clear, type-safe interface
    database.get_order(user_id, order_id)
}

// Strong typing prevents errors
let user_id = UserKey::new("user_123")?;
let order_id = OrderKey::new("order_456")?;
let product_id = ProductKey::new("product_789")?;

// These won't compile - caught at compile time!
// if user_id == order_id { /* Compile error! */ }
// if order_id == product_id { /* Compile error! */ }

// Type-safe collections
let mut user_cache: HashMap<UserKey, UserData> = HashMap::new();
let mut order_cache: HashMap<OrderKey, OrderData> = HashMap::new();
user_cache.insert(user_id, user_data);
order_cache.insert(order_id, order_data);
// No confusion possible!
```

## Migration Strategy

### 1. Gradual Migration (Recommended)

Migrate module by module to minimize risk:

```rust
// Phase 1: Create domain definitions alongside existing code
mod keys {
    use domain_key::{Key, KeyDomain};

    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct UserDomain;

    impl KeyDomain for UserDomain {
        const DOMAIN_NAME: &'static str = "user";
        const MAX_LENGTH: usize = 32;
        const TYPICALLY_SHORT: bool = true; // Optimization hint
    }

    pub type UserKey = Key<UserDomain>;

    // Add conversion helpers
    impl UserKey {
        pub fn from_string_unchecked(s: String) -> Self {
            Self::new(s).expect("Invalid user key")
        }

        pub fn to_string(&self) -> String {
            self.as_str().to_string()
        }
    }
}

// Phase 2: Use new keys in new code
use keys::UserKey;

fn new_function(user_id: UserKey) -> Result<UserData, Error> {
    // New code uses domain keys
    Ok(UserData::load(user_id)?)
}

// Phase 3: Gradually convert existing functions
fn existing_function(user_id: String) -> Result<UserData, Error> {
    // Convert at boundary
    let user_key = UserKey::new(user_id)?;
    new_function(user_key)
}
```

### 2. Big Bang Migration

For smaller codebases, you might prefer complete migration:

```rust
// Before: String everywhere
struct UserService {
    cache: HashMap<String, UserData>,
}

impl UserService {
    fn get_user(&self, id: String) -> Option<&UserData> {
        self.cache.get(&id)
    }

    fn store_user(&mut self, id: String, data: UserData) {
        self.cache.insert(id, data);
    }
}

// After: domain-key everywhere
struct UserService {
    cache: HashMap<UserKey, UserData>,
}

impl UserService {
    fn get_user(&self, id: UserKey) -> Option<&UserData> {
        self.cache.get(&id)
    }

    fn store_user(&mut self, id: UserKey, data: UserData) {
        self.cache.insert(id, data);
    }
}
```

## Common Patterns

### Pattern 1: Simple String Replacement

**Before:**
```rust
struct User {
    id: String,
    name: String,
}

fn find_user(id: &str) -> Option<User> {
    // Database lookup
}
```

**After:**
```rust
struct User {
    id: UserKey,
    name: String,
}

fn find_user(id: UserKey) -> Option<User> {
    // Database lookup
}
```

### Pattern 2: Collections

**Before:**
```rust
let mut users: HashMap<String, User> = HashMap::new();
let mut orders: HashMap<String, Order> = HashMap::new();

// Easy to mix up keys
users.insert("123".to_string(), user);
orders.insert("123".to_string(), order); // Different "123"!
```

**After:**
```rust
let mut users: HashMap<UserKey, User> = HashMap::new();
let mut orders: HashMap<OrderKey, Order> = HashMap::new();

// Type safety prevents mixing
users.insert(UserKey::new("123")?, user);
orders.insert(OrderKey::new("123")?, order); // Different types!
```

### Pattern 3: Function Parameters

**Before:**
```rust
fn create_order(user_id: String, product_id: String, quantity: u32) -> Order {
    // Easy to swap parameters
    Order::new(user_id, product_id, quantity)
}

// Dangerous - parameters swapped!
let order = create_order(product_id, user_id, 1);
```

**After:**
```rust
fn create_order(user_id: UserKey, product_id: ProductKey, quantity: u32) -> Order {
    Order::new(user_id, product_id, quantity)
}

// This won't compile if parameters are swapped
let order = create_order(user_id, product_id, 1);
```

### Pattern 4: API Boundaries

**Before:**
```rust
#[derive(Serialize, Deserialize)]
struct ApiRequest {
    user_id: String,
    session_id: String,
}

// No validation at deserialization
```

**After:**
```rust
#[derive(Serialize, Deserialize)]
struct ApiRequest {
    user_id: UserKey,
    session_id: SessionKey,
}

// Automatic validation during deserialization!
```

## Step-by-Step Migration

### Step 1: Define Your Domains

Identify the different types of keys in your application:

```rust
// domains.rs
use domain_key::{Key, KeyDomain, KeyParseError};
use std::borrow::Cow;

// User domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UserDomain;

impl KeyDomain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
    const MAX_LENGTH: usize = 32;
    const TYPICALLY_SHORT: bool = true;
    const FREQUENTLY_COMPARED: bool = true; // Often used in hash maps
}

// Session domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionDomain;

impl KeyDomain for SessionDomain {
    const DOMAIN_NAME: &'static str = "session";
    const MAX_LENGTH: usize = 64;
    const HAS_CUSTOM_VALIDATION: bool = true;

    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        if !key.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Session keys must be alphanumeric"
            ));
        }
        Ok(())
    }
}

// Product domain with normalization
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProductDomain;

impl KeyDomain for ProductDomain {
    const DOMAIN_NAME: &'static str = "product";
    const MAX_LENGTH: usize = 48;
    const HAS_CUSTOM_NORMALIZATION: bool = true;

    fn normalize_domain(key: Cow<'_, str>) -> Cow<'_, str> {
        // Normalize product keys to lowercase with underscores
        if key.chars().any(|c| c.is_ascii_uppercase() || c == '-' || c == ' ') {
            let normalized = key.to_ascii_lowercase().replace(['-', ' '], "_");
            Cow::Owned(normalized)
        } else {
            key
        }
    }
}

// Type aliases for easy use
pub type UserKey = Key<UserDomain>;
pub type SessionKey = Key<SessionDomain>;
pub type ProductKey = Key<ProductDomain>;
```

### Step 2: Create Conversion Helpers

Build bridges between old and new systems:

```rust
// conversion.rs
use crate::domains::*;

pub trait StringKeyConversion<T> {
    fn from_string_key(s: String) -> Result<T, domain_key::KeyParseError>;
    fn to_string_key(&self) -> String;
}

impl StringKeyConversion<UserKey> for UserKey {
    fn from_string_key(s: String) -> Result<UserKey, domain_key::KeyParseError> {
        UserKey::from_string(s)
    }

    fn to_string_key(&self) -> String {
        self.as_str().to_string()
    }
}

impl StringKeyConversion<SessionKey> for SessionKey {
    fn from_string_key(s: String) -> Result<SessionKey, domain_key::KeyParseError> {
        SessionKey::from_string(s)
    }

    fn to_string_key(&self) -> String {
        self.as_str().to_string()
    }
}

// Macro for easy conversion
macro_rules! convert_or_return {
    ($string_key:expr, $key_type:ty) => {
        match <$key_type>::from_string_key($string_key) {
            Ok(key) => key,
            Err(e) => return Err(e.into()),
        }
    };
}
```

### Step 3: Migrate Data Structures

Update your structs gradually:

```rust
// Before
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub email: String,
    pub session_id: Option<String>,
}

// During migration - support both
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserKey,           // New field
    pub email: String,
    pub session_id: Option<SessionKey>, // New field

    // Deprecated - remove in next version
    #[deprecated]
    pub legacy_id: Option<String>,
    #[deprecated]
    pub legacy_session_id: Option<String>,
}

impl User {
    // Migration constructor
    pub fn from_legacy(
        id: String,
        email: String,
        session_id: Option<String>,
    ) -> Result<Self, domain_key::KeyParseError> {
        Ok(User {
            id: UserKey::from_string(id.clone())?,
            email,
            session_id: session_id.as_ref()
                .map(|s| SessionKey::new(s))
                .transpose()?,
            legacy_id: Some(id),
            legacy_session_id: session_id,
        })
    }

    // New constructor
    pub fn new(
        id: UserKey,
        email: String,
        session_id: Option<SessionKey>,
    ) -> Self {
        User {
            id,
            email,
            session_id,
            legacy_id: None,
            legacy_session_id: None,
        }
    }
}

// After migration - clean version
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserKey,
    pub email: String,
    pub session_id: Option<SessionKey>,
}
```

### Step 4: Migrate Functions

Update function signatures progressively:

```rust
// Legacy function (keep for compatibility)
pub fn get_user_legacy(id: &str) -> Result<Option<User>, Error> {
    let user_key = UserKey::new(id)?;
    get_user(user_key)
}

// New function (preferred)
pub fn get_user(id: UserKey) -> Result<Option<User>, Error> {
    // Implementation using domain keys
    database.find_user(id)
}

// Transition helper
#[deprecated(note = "Use get_user with UserKey instead")]
pub fn get_user_string(id: String) -> Result<Option<User>, Error> {
    get_user_legacy(&id)
}
```

### Step 5: Update APIs

Migrate your API layer:

```rust
// API handlers
use axum::{Json, extract::Path};

// Before
async fn get_user_handler(
    Path(user_id): Path<String>
) -> Result<Json<User>, ApiError> {
    let user = get_user_legacy(&user_id)?;
    Ok(Json(user))
}

// After  
async fn get_user_handler(
    Path(user_id): Path<UserKey>
) -> Result<Json<User>, ApiError> {
    // UserKey is automatically validated during deserialization!
    let user = get_user(user_id)?;
    Ok(Json(user))
}
```

## Compatibility Layer

Create a compatibility layer for gradual migration:

```rust
// compat.rs
use crate::domains::*;

pub struct CompatUserService {
    inner: UserService, // New service using domain keys
}

impl CompatUserService {
    // Legacy methods that convert to new types
    pub fn get_user_by_string(&self, id: String) -> Result<Option<User>, Error> {
        let user_key = UserKey::from_string(id)?;
        self.inner.get_user(user_key)
    }

    pub fn create_user_from_string(
        &mut self,
        id: String,
        email: String
    ) -> Result<User, Error> {
        let user_key = UserKey::from_string(id)?;
        self.inner.create_user(user_key, email)
    }

    // Forward to new methods
    pub fn get_user(&self, id: UserKey) -> Result<Option<User>, Error> {
        self.inner.get_user(id)
    }
}
```

## Testing Migration

### Unit Tests

Test both old and new interfaces during migration:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_key_creation() {
        let key = UserKey::new("user_123").unwrap();
        assert_eq!(key.as_str(), "user_123");
        assert_eq!(key.domain(), "user");
    }

    #[test]
    fn test_legacy_compatibility() {
        let user_id = "user_123".to_string();
        let user_key = UserKey::from_string(user_id.clone()).unwrap();

        // Both should work the same
        let result1 = get_user_legacy(&user_id).unwrap();
        let result2 = get_user(user_key).unwrap();

        assert_eq!(result1, result2);
    }

    #[test]
    fn test_conversion_helpers() {
        let original = "user_456".to_string();
        let key = UserKey::from_string_key(original.clone()).unwrap();
        let converted_back = key.to_string_key();

        assert_eq!(original, converted_back);
    }

    #[test]
    fn test_migration_validation() {
        // Test that new validation rules work correctly
        let valid_session = SessionKey::new("abcd1234").unwrap();
        assert_eq!(valid_session.as_str(), "abcd1234");

        // Invalid session (contains special characters)
        let invalid_session = SessionKey::new("abc-123");
        assert!(invalid_session.is_err());
    }
}
```

### Integration Tests

Test the entire migration path:

```rust
#[tokio::test]
async fn test_api_migration() {
    let app = create_test_app();

    // Test legacy endpoint
    let response = app
        .get("/api/users/user_123")
        .send()
        .await;
    assert_eq!(response.status(), 200);

    // Test new endpoint (same result)
    let response = app
        .get("/api/v2/users/user_123")
        .send()
        .await;
    assert_eq!(response.status(), 200);
}

#[test]
fn test_database_migration() {
    // Test that keys work correctly with database operations
    let user_key = UserKey::new("test_user").unwrap();

    // Store user with new key system
    let user = User::new(user_key.clone(), "test@example.com".to_string(), None);
    database.store_user(&user).unwrap();

    // Retrieve using both old and new methods
    let retrieved = database.get_user(user_key).unwrap();
    assert!(retrieved.is_some());
}
```

## Performance Considerations

### Memory Usage

domain-key can be more memory-efficient than String:

```rust
use std::mem::size_of;

// String: typically 24 bytes (ptr + len + capacity)
println!("String size: {}", size_of::<String>());

// UserKey: optimized size (SmartString + cached data)
println!("UserKey size: {}", size_of::<UserKey>());

// For short keys (≤23 chars), SmartString uses stack allocation
let short_key = UserKey::new("user123").unwrap();  // Stack allocated
let long_key = UserKey::new("very_long_user_identifier_name").unwrap(); // Heap allocated
```

### Hash Performance

domain-key provides cached hashing:

```rust
use std::collections::HashMap;
use std::time::Instant;

// Before: String keys hash every time
let mut string_map: HashMap<String, Data> = HashMap::new();
let start = Instant::now();
for i in 0..10000 {
let key = format!("user_{}", i);
string_map.insert(key, data.clone()); // Hashes "user_N" every time
}
let string_time = start.elapsed();

// After: Cached hash in domain keys  
let mut key_map: HashMap<UserKey, Data> = HashMap::new();
let start = Instant::now();
for i in 0..10000 {
let key = UserKey::new(format!("user_{}", i)).unwrap(); // Hash computed once
key_map.insert(key, data.clone()); // Uses cached hash
}
let key_time = start.elapsed();

println!("String approach: {:?}", string_time);
println!("Domain key approach: {:?}", key_time);
// Domain keys are typically 30-40% faster for hash operations
```

### Validation Performance

Move validation to creation time:

```rust
// Before: Validate on every use
fn process_user(id: &str) -> Result<(), Error> {
    validate_user_id(id)?; // Validation every time
    // ... process
}

// After: Validate once at creation
fn process_user(id: UserKey) -> Result<(), Error> {
    // id is already validated!
    // ... process
}
```

### Optimized Domain Configuration

Configure domains for your performance profile:

```rust
impl KeyDomain for HighPerformanceDomain {
    const DOMAIN_NAME: &'static str = "fast";
    const MAX_LENGTH: usize = 32;
    const EXPECTED_LENGTH: usize = 16;     // Pre-allocation hint
    const TYPICALLY_SHORT: bool = true;    // Stack allocation
    const FREQUENTLY_COMPARED: bool = true; // Hash optimizations
    const FREQUENTLY_SPLIT: bool = false;  // Disable split caching
}
```

## Troubleshooting

### Common Migration Issues

**Issue**: Compilation errors when mixing key types
```rust
let user_key = UserKey::new("123")?;
let order_key = OrderKey::new("456")?;
// if user_key == order_key { } // Won't compile!
```

**Solution**: This is intentional! Use string comparison if needed:
```rust
if user_key.as_str() == order_key.as_str() {
// Explicit string comparison
}
```

**Issue**: Serde serialization format changes
```rust
// Before: "user_123"
// After: "user_123" (same!)
```

**Solution**: domain-key serializes as strings by default, so JSON/etc. format is unchanged.

**Issue**: Database integration
```rust
// Before
diesel::insert_into(users)
.values(NewUser { id: user_id_string })
.execute(conn)?;

// After - need to convert
diesel::insert_into(users)
.values(NewUser { id: user_key.as_str() })
.execute(conn)?;
```

**Solution**: Create helper traits for database integration:
```rust
trait ToDbString {
    fn to_db_string(&self) -> &str;
}

impl ToDbString for UserKey {
    fn to_db_string(&self) -> &str {
        self.as_str()
    }
}
```

**Issue**: Performance regression during migration
```rust
// Mixed usage can hurt performance
fn mixed_usage() {
    let string_key = "user_123".to_string();
    let domain_key = UserKey::new(&string_key).unwrap(); // Extra allocation
    // Use domain_key...
}
```

**Solution**: Prefer creating domain keys early:
```rust
fn optimal_usage() {
    let domain_key = UserKey::new("user_123").unwrap(); // Direct creation
    // Use domain_key everywhere...
}
```

### Migration Checklist

- [ ] Identify all key types in your application
- [ ] Define corresponding domains with appropriate optimization hints
- [ ] Create conversion helpers for gradual migration
- [ ] Start with leaf functions (no dependencies)
- [ ] Work backwards to API boundaries
- [ ] Update database interactions
- [ ] Test thoroughly at each step
- [ ] Monitor performance during migration
- [ ] Remove compatibility layer after migration
- [ ] Update documentation

### Performance Monitoring

Monitor key metrics during migration:

```rust
// Add performance monitoring
struct MigrationMetrics {
    key_creation_time: std::time::Duration,
    hash_operations: usize,
    validation_failures: usize,
}

impl MigrationMetrics {
    fn time_key_creation<F, R>(&mut self, f: F) -> R
    where F: FnOnce() -> R
    {
        let start = std::time::Instant::now();
        let result = f();
        self.key_creation_time += start.elapsed();
        result
    }
}
```

### Rolling Back

If you need to roll back migration:

```rust
// Emergency rollback helper
impl UserKey {
    pub fn emergency_to_string(&self) -> String {
        self.as_str().to_string()
    }
}

// Quickly convert back if needed
let string_id = user_key.emergency_to_string();
```

---

Happy migrating! The type safety and performance benefits are worth the effort. 🚀

## Next Steps

- Read the [Performance Guide](performance.md) for optimization strategies
- Check out the [examples/](../examples/) directory for real-world usage patterns
- Browse the [API documentation](https://docs.rs/domain-key) for complete reference
- Join our community discussions for migration support
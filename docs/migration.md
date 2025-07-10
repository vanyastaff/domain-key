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

// User domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UserDomain;

impl KeyDomain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
    const MAX_LENGTH: usize = 32;
}

// Session domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SessionDomain;

impl KeyDomain for SessionDomain {
    const DOMAIN_NAME: &'static str = "session";
    const MAX_LENGTH: usize = 64;
    
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

// Product domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProductDomain;

impl KeyDomain for ProductDomain {
    const DOMAIN_NAME: &'static str = "product";
    const MAX_LENGTH: usize = 48;
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
        UserKey::new(s)
    }
    
    fn to_string_key(&self) -> String {
        self.as_str().to_string()
    }
}

impl StringKeyConversion<SessionKey> for SessionKey {
    fn from_string_key(s: String) -> Result<SessionKey, domain_key::KeyParseError> {
        SessionKey::new(s)
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
            id: UserKey::new(id.clone())?,
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
        let user_key = UserKey::new(id)?;
        self.inner.get_user(user_key)
    }
    
    pub fn create_user_from_string(
        &mut self, 
        id: String, 
        email: String
    ) -> Result<User, Error> {
        let user_key = UserKey::new(id)?;
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
        let user_key = UserKey::new(&user_id).unwrap();
        
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
```

## Performance Considerations

### Memory Usage

domain-key can be more memory-efficient than String:

```rust
use std::mem::size_of;

// String: typically 24 bytes (ptr + len + capacity)
println!("String size: {}", size_of::<String>());

// UserKey: optimized size (typically 32 bytes including cached hash)
println!("UserKey size: {}", size_of::<UserKey>());
```

### Hash Performance

domain-key provides cached hashing:

```rust
use std::collections::HashMap;

// Before: String keys hash every time
let mut string_map: HashMap<String, Data> = HashMap::new();
string_map.insert("user_123".to_string(), data); // Hashes "user_123"
let value = string_map.get("user_123"); // Hashes "user_123" again

// After: Cached hash in domain keys
let mut key_map: HashMap<UserKey, Data> = HashMap::new();
let key = UserKey::new("user_123")?; // Hash computed once
key_map.insert(key.clone(), data); // Uses cached hash
let value = key_map.get(&key); // Uses cached hash
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

### Migration Checklist

- [ ] Identify all key types in your application
- [ ] Define corresponding domains
- [ ] Create conversion helpers
- [ ] Start with leaf functions (no dependencies)
- [ ] Work backwards to API boundaries
- [ ] Update database interactions
- [ ] Test thoroughly at each step
- [ ] Remove compatibility layer after migration
- [ ] Update documentation

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

Happy migrating! The type safety benefits are worth the effort. 🚀
//! Web application example with session management and caching
#![allow(dead_code)]

use domain_key::{Key, KeyDomain, KeyParseError};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Session domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct SessionDomain;

impl KeyDomain for SessionDomain {
    const DOMAIN_NAME: &'static str = "session";
    const MAX_LENGTH: usize = 64;
    const TYPICALLY_SHORT: bool = false;

    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        // Session IDs should be alphanumeric for security
        if !key.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Session IDs must be alphanumeric",
            ));
        }

        if key.len() < 16 {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Session IDs must be at least 16 characters",
            ));
        }

        Ok(())
    }
}

// Cache domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CacheDomain;

impl KeyDomain for CacheDomain {
    const DOMAIN_NAME: &'static str = "cache";
    const MAX_LENGTH: usize = 128;

    fn normalize_domain(key: std::borrow::Cow<'_, str>) -> std::borrow::Cow<'_, str> {
        // Normalize cache keys for consistency
        if key.contains(' ') || key.contains(':') {
            let normalized = key.replace(' ', "_").replace(':', "_");
            std::borrow::Cow::Owned(normalized)
        } else {
            key
        }
    }
}

// Request ID domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct RequestDomain;

impl KeyDomain for RequestDomain {
    const DOMAIN_NAME: &'static str = "request";
    const MAX_LENGTH: usize = 36; // UUID format
    const TYPICALLY_SHORT: bool = true;
}

// User domain (reused from other examples)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct UserDomain;

impl KeyDomain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
    const MAX_LENGTH: usize = 32;
}

// Type aliases
type SessionKey = Key<SessionDomain>;
type CacheKey = Key<CacheDomain>;
type RequestKey = Key<RequestDomain>;
type UserKey = Key<UserDomain>;

// Web application entities
#[derive(Debug, Clone)]
struct Session {
    id: SessionKey,
    user_id: UserKey,
    created_at: u64,
    last_accessed: u64,
    data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    key: CacheKey,
    value: String,
    expires_at: u64,
}

#[derive(Debug, Clone)]
struct RequestContext {
    id: RequestKey,
    session_id: Option<SessionKey>,
    user_id: Option<UserKey>,
    ip_address: String,
    user_agent: String,
}

// Web application service
struct WebAppService {
    sessions: HashMap<SessionKey, Session>,
    cache: HashMap<CacheKey, CacheEntry>,
    active_requests: HashMap<RequestKey, RequestContext>,
}

impl WebAppService {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            cache: HashMap::new(),
            active_requests: HashMap::new(),
        }
    }

    fn create_session(&mut self, user_id: UserKey) -> Result<SessionKey, KeyParseError> {
        // Generate session ID (in real app, use crypto-secure random)
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let session_id =
            SessionKey::new(format!("sess_{:016x}_{:08x}", user_id.hash(), timestamp))?;

        let session = Session {
            id: session_id.clone(),
            user_id,
            created_at: timestamp,
            last_accessed: timestamp,
            data: HashMap::new(),
        };

        self.sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    fn get_session(&mut self, session_id: &SessionKey) -> Option<&mut Session> {
        if let Some(session) = self.sessions.get_mut(session_id) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            session.last_accessed = now;
            Some(session)
        } else {
            None
        }
    }

    fn set_cache(
        &mut self,
        namespace: &str,
        key: &str,
        value: String,
        ttl_seconds: u64,
    ) -> Result<CacheKey, KeyParseError> {
        let cache_key = CacheKey::from_parts(&[namespace, key], ":")?;
        let expires_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + ttl_seconds;

        let entry = CacheEntry {
            key: cache_key.clone(),
            value,
            expires_at,
        };

        self.cache.insert(cache_key.clone(), entry);
        Ok(cache_key)
    }

    fn get_cache(&self, cache_key: &CacheKey) -> Option<&str> {
        if let Some(entry) = self.cache.get(cache_key) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if entry.expires_at > now {
                Some(&entry.value)
            } else {
                None // Expired
            }
        } else {
            None
        }
    }

    fn start_request(
        &mut self,
        ip_address: String,
        user_agent: String,
        session_id: Option<SessionKey>,
    ) -> Result<RequestKey, KeyParseError> {
        let request_id = RequestKey::new(format!(
            "req_{:016x}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))?;

        let user_id = session_id
            .as_ref()
            .and_then(|sid| self.sessions.get(sid))
            .map(|session| session.user_id.clone());

        let context = RequestContext {
            id: request_id.clone(),
            session_id,
            user_id,
            ip_address,
            user_agent,
        };

        self.active_requests.insert(request_id.clone(), context);
        Ok(request_id)
    }

    fn cleanup_expired(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Remove expired cache entries
        self.cache.retain(|_, entry| entry.expires_at > now);

        // Remove old sessions (24 hour expiry)
        self.sessions
            .retain(|_, session| session.last_accessed + 86400 > now);
    }

    fn get_user_session_cache_key(&self, user_id: &UserKey) -> Result<CacheKey, KeyParseError> {
        CacheKey::from_parts(&["user_data", user_id.as_str()], ":")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Web Application Example ===\n");

    let mut app = WebAppService::new();

    // Simulate user registration/login
    let alice_id = UserKey::new("alice_123")?;
    let bob_id = UserKey::new("bob_456")?;

    println!("Users:");
    println!("  Alice: {}", alice_id.as_str());
    println!("  Bob: {}", bob_id.as_str());

    // Create sessions
    let alice_session = app.create_session(alice_id.clone())?;
    let bob_session = app.create_session(bob_id.clone())?;

    println!("\nSessions created:");
    println!("  Alice session: {}", alice_session.as_str());
    println!("  Bob session: {}", bob_session.as_str());

    // Cache user data
    let alice_cache_key = app.set_cache(
        "user_profile",
        alice_id.as_str(),
        "Alice's profile data".to_string(),
        3600,
    )?;
    let bob_cache_key = app.set_cache(
        "user profile",
        bob_id.as_str(),
        "Bob's profile data".to_string(),
        3600,
    )?;

    println!("\nCache entries:");
    println!("  Alice cache: {}", alice_cache_key.as_str());
    println!("  Bob cache: {} (normalized)", bob_cache_key.as_str());

    // Simulate web requests
    let request1 = app.start_request(
        "192.168.1.100".to_string(),
        "Mozilla/5.0 Chrome/91.0".to_string(),
        Some(alice_session.clone()),
    )?;

    let request2 = app.start_request(
        "10.0.0.50".to_string(),
        "Mozilla/5.0 Firefox/89.0".to_string(),
        Some(bob_session.clone()),
    )?;

    println!("\nActive requests:");
    println!("  Request 1: {}", request1.as_str());
    println!("  Request 2: {}", request2.as_str());

    // Access session data
    if let Some(session) = app.get_session(&alice_session) {
        session
            .data
            .insert("last_page".to_string(), "/dashboard".to_string());
        println!("\nUpdated Alice's session data");
    }

    // Access cached data
    if let Some(cached_data) = app.get_cache(&alice_cache_key) {
        println!("Retrieved from cache: {}", cached_data);
    }

    // Demonstrate cache key generation
    let user_cache_key = app.get_user_session_cache_key(&alice_id)?;
    println!("Generated cache key: {}", user_cache_key.as_str());

    // Show active sessions
    println!("\n📊 Application state:");
    println!("  Active sessions: {}", app.sessions.len());
    println!("  Cache entries: {}", app.cache.len());
    println!("  Active requests: {}", app.active_requests.len());

    // Cleanup demonstration
    println!("\nCleaning up expired entries...");
    app.cleanup_expired();
    println!("Cleanup completed");

    // Type safety demonstration
    println!("\n🛡️ Type safety in action:");
    println!("  Session keys can only be used for sessions");
    println!("  Cache keys can only be used for caching");
    println!("  User keys can only be used for users");

    // These would not compile:
    // app.get_session(&alice_cache_key); // Wrong key type!
    // app.get_cache(&alice_session);     // Wrong key type!

    println!("\n✅ Web application example completed successfully!");
    Ok(())
}

//! Basic usage example demonstrating core domain-key functionality

use domain_key::{Key, KeyDomain};

// Define a simple domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct UserDomain;

impl KeyDomain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
    const MAX_LENGTH: usize = 32;
}

// Create a type alias for convenience
type UserKey = Key<UserDomain>;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Basic domain-key Usage ===\n");

    // Create keys
    let user1 = UserKey::new("alice")?;
    let user2 = UserKey::new("bob_smith")?;

    println!("Created users:");
    println!("  User 1: {} (domain: {})", user1.as_str(), user1.domain());
    println!("  User 2: {} (domain: {})", user2.as_str(), user2.domain());

    // Key properties
    println!("\nKey properties:");
    println!("  User 1 length: {}", user1.len());
    println!("  User 1 hash: 0x{:x}", user1.hash());

    // String operations
    println!("\nString operations:");
    println!("  Starts with 'alice': {}", user1.starts_with("alice"));
    println!("  Contains 'bob': {}", user2.contains("bob"));

    // Create keys from parts
    let composite_key = UserKey::from_parts(&["user", "123", "profile"], "_")?;
    println!("  Composite key: {}", composite_key.as_str());

    // Split operations
    let parts: Vec<&str> = composite_key.split('_').collect();
    println!("  Split parts: {:?}", parts);

    // Error handling
    match UserKey::new("") {
        Ok(_) => println!("Unexpected success"),
        Err(e) => println!("\nExpected error for empty key: {}", e),
    }

    // Demonstrate type safety (this would not compile):
    // let comparison = user1 == some_other_domain_key; // Compile error!

    println!("\n✅ Basic usage completed successfully!");
    Ok(())
}

//! Multi-tenant SaaS application example

use domain_key::{Key, KeyDomain, KeyParseError};
use std::borrow::Cow;
use std::collections::HashMap;

// Tenant domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct TenantDomain;

impl KeyDomain for TenantDomain {
    const DOMAIN_NAME: &'static str = "tenant";
    const MAX_LENGTH: usize = 32;
    const HAS_CUSTOM_VALIDATION: bool = true;
    const HAS_CUSTOM_NORMALIZATION: bool = true;

    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        if !key.starts_with("tenant_") {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Tenant keys must start with 'tenant_'",
            ));
        }

        let suffix = &key[7..]; // Remove "tenant_" prefix
        if suffix.is_empty()
            || !suffix
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Tenant suffix must be alphanumeric with underscores",
            ));
        }

        Ok(())
    }

    fn normalize_domain(key: Cow<'_, str>) -> Cow<'_, str> {
        if key.chars().any(|c| c.is_ascii_uppercase()) {
            Cow::Owned(key.to_ascii_lowercase())
        } else {
            key
        }
    }
}

// User domain (scoped within tenant)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct UserDomain;

impl KeyDomain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
    const MAX_LENGTH: usize = 64; // Longer to accommodate tenant prefix

    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        // Users should include tenant context
        if !key.contains('@') {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "User keys must include tenant context (user@tenant format)",
            ));
        }
        Ok(())
    }
}

// Resource domain (tenant-scoped resources)
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ResourceDomain;

impl KeyDomain for ResourceDomain {
    const DOMAIN_NAME: &'static str = "resource";
    const MAX_LENGTH: usize = 80;
}

// Type aliases
type TenantKey = Key<TenantDomain>;
type UserKey = Key<UserDomain>;
type ResourceKey = Key<ResourceDomain>;

// Entities
#[derive(Debug, Clone)]
struct Tenant {
    id: TenantKey,
    name: String,
    plan: String,
    active: bool,
}

#[derive(Debug, Clone)]
struct User {
    id: UserKey,
    tenant_id: TenantKey,
    name: String,
    role: String,
}

#[derive(Debug, Clone)]
struct Resource {
    id: ResourceKey,
    tenant_id: TenantKey,
    name: String,
    resource_type: String,
    data: String,
}

// Multi-tenant service
struct MultiTenantService {
    tenants: HashMap<TenantKey, Tenant>,
    users: HashMap<UserKey, User>,
    resources: HashMap<ResourceKey, Resource>,
}

impl MultiTenantService {
    fn new() -> Self {
        Self {
            tenants: HashMap::new(),
            users: HashMap::new(),
            resources: HashMap::new(),
        }
    }

    fn create_tenant(&mut self, name: String, plan: String) -> Result<TenantKey, KeyParseError> {
        let tenant_slug = name.to_lowercase().replace(' ', "_");
        let tenant_id = TenantKey::new(format!("TENANT_{}", tenant_slug))?;

        let tenant = Tenant {
            id: tenant_id.clone(),
            name,
            plan,
            active: true,
        };

        self.tenants.insert(tenant_id.clone(), tenant);
        Ok(tenant_id)
    }

    fn create_user(
        &mut self,
        tenant_id: TenantKey,
        username: String,
        name: String,
        role: String,
    ) -> Result<UserKey, Box<dyn std::error::Error>> {
        // Verify tenant exists
        if !self.tenants.contains_key(&tenant_id) {
            return Err("Tenant not found".into());
        }

        let user_id = UserKey::new(format!("{}@{}", username, tenant_id.as_str()))?;

        let user = User {
            id: user_id.clone(),
            tenant_id,
            name,
            role,
        };

        self.users.insert(user_id.clone(), user);
        Ok(user_id)
    }

    fn create_resource(
        &mut self,
        tenant_id: TenantKey,
        name: String,
        resource_type: String,
        data: String,
    ) -> Result<ResourceKey, Box<dyn std::error::Error>> {
        // Verify tenant exists
        if !self.tenants.contains_key(&tenant_id) {
            return Err("Tenant not found".into());
        }

        let resource_id = ResourceKey::from_parts(
            &[tenant_id.as_str(), &resource_type, &name.replace(' ', "_")],
            "_",
        )?;

        let resource = Resource {
            id: resource_id.clone(),
            tenant_id,
            name,
            resource_type,
            data,
        };

        self.resources.insert(resource_id.clone(), resource);
        Ok(resource_id)
    }

    fn get_tenant_users(&self, tenant_id: &TenantKey) -> Vec<&User> {
        self.users
            .values()
            .filter(|user| &user.tenant_id == tenant_id)
            .collect()
    }

    fn get_tenant_resources(&self, tenant_id: &TenantKey) -> Vec<&Resource> {
        self.resources
            .values()
            .filter(|resource| &resource.tenant_id == tenant_id)
            .collect()
    }

    fn authorize_user_access(&self, user_id: &UserKey, resource_id: &ResourceKey) -> bool {
        if let (Some(user), Some(resource)) =
            (self.users.get(user_id), self.resources.get(resource_id))
        {
            // Users can only access resources in their tenant
            user.tenant_id == resource.tenant_id
        } else {
            false
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Multi-Tenant SaaS Example ===\n");

    let mut service = MultiTenantService::new();

    // Create tenants
    let acme_tenant = service.create_tenant("ACME Corp".to_string(), "enterprise".to_string())?;
    let startup_tenant = service.create_tenant("Startup Inc".to_string(), "basic".to_string())?;

    println!("Created tenants:");
    println!("  ACME Corp: {}", acme_tenant.as_str());
    println!("  Startup Inc: {}", startup_tenant.as_str());

    // Create users
    let alice_id = service.create_user(
        acme_tenant.clone(),
        "alice".to_string(),
        "Alice Johnson".to_string(),
        "admin".to_string(),
    )?;

    let bob_id = service.create_user(
        acme_tenant.clone(),
        "bob".to_string(),
        "Bob Smith".to_string(),
        "user".to_string(),
    )?;

    let charlie_id = service.create_user(
        startup_tenant.clone(),
        "charlie".to_string(),
        "Charlie Brown".to_string(),
        "admin".to_string(),
    )?;

    println!("\nCreated users:");
    println!("  Alice (ACME): {}", alice_id.as_str());
    println!("  Bob (ACME): {}", bob_id.as_str());
    println!("  Charlie (Startup): {}", charlie_id.as_str());

    // Create resources
    let acme_db = service.create_resource(
        acme_tenant.clone(),
        "customer database".to_string(),
        "database".to_string(),
        "postgres://acme-db/customers".to_string(),
    )?;

    let acme_api = service.create_resource(
        acme_tenant.clone(),
        "API Gateway".to_string(),
        "api".to_string(),
        "https://api.acme.com".to_string(),
    )?;

    let startup_db = service.create_resource(
        startup_tenant.clone(),
        "user data".to_string(),
        "database".to_string(),
        "sqlite://data.db".to_string(),
    )?;

    println!("\nCreated resources:");
    println!("  ACME DB: {}", acme_db.as_str());
    println!("  ACME API: {}", acme_api.as_str());
    println!("  Startup DB: {}", startup_db.as_str());

    // Test tenant isolation
    println!("\n🔒 Tenant isolation tests:");

    // Alice (ACME) can access ACME resources
    println!(
        "  Alice -> ACME DB: {}",
        service.authorize_user_access(&alice_id, &acme_db)
    );

    // Alice (ACME) cannot access Startup resources
    println!(
        "  Alice -> Startup DB: {}",
        service.authorize_user_access(&alice_id, &startup_db)
    );

    // Charlie (Startup) can access Startup resources
    println!(
        "  Charlie -> Startup DB: {}",
        service.authorize_user_access(&charlie_id, &startup_db)
    );

    // Charlie (Startup) cannot access ACME resources
    println!(
        "  Charlie -> ACME DB: {}",
        service.authorize_user_access(&charlie_id, &acme_db)
    );

    // Display tenant summary
    let acme_users = service.get_tenant_users(&acme_tenant);
    let acme_resources = service.get_tenant_resources(&acme_tenant);

    println!("\n📊 ACME Corp summary:");
    println!("  Users: {}", acme_users.len());
    println!("  Resources: {}", acme_resources.len());

    let startup_users = service.get_tenant_users(&startup_tenant);
    let startup_resources = service.get_tenant_resources(&startup_tenant);

    println!("\n📊 Startup Inc summary:");
    println!("  Users: {}", startup_users.len());
    println!("  Resources: {}", startup_resources.len());

    // Demonstrate type safety
    println!("\n🛡️ Type safety:");
    println!("  Different domains cannot be compared at compile time");
    // if alice_id == acme_db { } // Would not compile!

    println!("\n✅ Multi-tenant example completed successfully!");
    Ok(())
}

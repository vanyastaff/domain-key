//! E-commerce domain example showing multiple related domains
#![allow(dead_code)]

use domain_key::{Key, KeyDomain, KeyParseError};
use std::borrow::Cow;
use std::collections::HashMap;

// User domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct UserDomain;

impl KeyDomain for UserDomain {
    const DOMAIN_NAME: &'static str = "user";
    const MAX_LENGTH: usize = 32;

    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        if key.starts_with("admin_") && !key.ends_with("_verified") {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Admin users must have '_verified' suffix",
            ));
        }
        Ok(())
    }
}

// Product domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct ProductDomain;

impl KeyDomain for ProductDomain {
    const DOMAIN_NAME: &'static str = "product";
    const MAX_LENGTH: usize = 48;
    const HAS_CUSTOM_NORMALIZATION: bool = true;

    fn normalize_domain(key: Cow<'_, str>) -> Cow<'_, str> {
        // Normalize product keys to lowercase with underscores
        if key
            .chars()
            .any(|c| c.is_ascii_uppercase() || c == '-' || c == ' ')
        {
            let normalized = key.to_ascii_lowercase().replace(['-', ' '], "_");
            Cow::Owned(normalized)
        } else {
            key
        }
    }
}

// Order domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct OrderDomain;

impl KeyDomain for OrderDomain {
    const DOMAIN_NAME: &'static str = "order";
    const MAX_LENGTH: usize = 36; // UUID format

    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        // Simple UUID format validation
        if key.len() == 36 && key.chars().filter(|&c| c == '-').count() == 4 {
            Ok(())
        } else {
            Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME,
                "Order IDs must be in UUID format",
            ))
        }
    }
}

// Cart domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct CartDomain;

impl KeyDomain for CartDomain {
    const DOMAIN_NAME: &'static str = "cart";
    const MAX_LENGTH: usize = 64;
}

// Type aliases
type UserKey = Key<UserDomain>;
type ProductKey = Key<ProductDomain>;
type OrderKey = Key<OrderDomain>;
type CartKey = Key<CartDomain>;

// Domain entities
#[derive(Debug, Clone)]
struct User {
    id: UserKey,
    name: String,
    email: String,
}

#[derive(Debug, Clone)]
struct Product {
    id: ProductKey,
    name: String,
    price: u32, // cents
}

#[derive(Debug, Clone)]
struct Order {
    id: OrderKey,
    user_id: UserKey,
    items: Vec<(ProductKey, u32)>, // (product_id, quantity)
    total: u32,
}

#[derive(Debug, Clone)]
struct Cart {
    id: CartKey,
    user_id: UserKey,
    items: HashMap<ProductKey, u32>, // product_id -> quantity
}

// E-commerce service
struct ECommerceService {
    users: HashMap<UserKey, User>,
    products: HashMap<ProductKey, Product>,
    orders: HashMap<OrderKey, Order>,
    carts: HashMap<CartKey, Cart>,
}

impl ECommerceService {
    fn new() -> Self {
        Self {
            users: HashMap::new(),
            products: HashMap::new(),
            orders: HashMap::new(),
            carts: HashMap::new(),
        }
    }

    fn add_user(&mut self, name: String, email: String) -> Result<UserKey, KeyParseError> {
        let user_id = UserKey::new(format!("user_{}", self.users.len() + 1))?;
        let user = User {
            id: user_id.clone(),
            name,
            email,
        };
        self.users.insert(user_id.clone(), user);
        Ok(user_id)
    }

    fn add_product(&mut self, name: String, price: u32) -> Result<ProductKey, KeyParseError> {
        // Normalize product name for ID
        let product_id = ProductKey::new(format!("PRODUCT-{}", name.replace(' ', "-")))?;
        let product = Product {
            id: product_id.clone(),
            name,
            price,
        };
        self.products.insert(product_id.clone(), product);
        Ok(product_id)
    }

    fn create_cart(&mut self, user_id: UserKey) -> Result<CartKey, KeyParseError> {
        let cart_id = CartKey::from_parts(&["cart", user_id.as_str()], "_")?;
        let cart = Cart {
            id: cart_id.clone(),
            user_id,
            items: HashMap::new(),
        };
        self.carts.insert(cart_id.clone(), cart);
        Ok(cart_id)
    }

    fn add_to_cart(
        &mut self,
        cart_id: &CartKey,
        product_id: ProductKey,
        quantity: u32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let cart = self.carts.get_mut(cart_id).ok_or("Cart not found")?;

        *cart.items.entry(product_id).or_insert(0) += quantity;
        Ok(())
    }

    fn create_order(&mut self, cart_id: &CartKey) -> Result<OrderKey, Box<dyn std::error::Error>> {
        let cart = self.carts.get(cart_id).ok_or("Cart not found")?;

        if cart.items.is_empty() {
            return Err("Cannot create order from empty cart".into());
        }

        // Generate UUID-like order ID
        let order_id = OrderKey::new("550e8400-e29b-41d4-a716-446655440000")?;

        let mut total = 0;
        let items: Vec<(ProductKey, u32)> = cart
            .items
            .iter()
            .map(|(product_id, &quantity)| {
                if let Some(product) = self.products.get(product_id) {
                    total += product.price * quantity;
                }
                (product_id.clone(), quantity)
            })
            .collect();

        let order = Order {
            id: order_id.clone(),
            user_id: cart.user_id.clone(),
            items,
            total,
        };

        self.orders.insert(order_id.clone(), order);

        // Clear cart
        self.carts.get_mut(cart_id).unwrap().items.clear();

        Ok(order_id)
    }

    fn get_user_orders(&self, user_id: &UserKey) -> Vec<&Order> {
        self.orders
            .values()
            .filter(|order| &order.user_id == user_id)
            .collect()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== E-commerce Domain Example ===\n");

    let mut service = ECommerceService::new();

    // Add users
    let alice_id =
        service.add_user("Alice Johnson".to_string(), "alice@example.com".to_string())?;
    let bob_id = service.add_user("Bob Smith".to_string(), "bob@example.com".to_string())?;

    println!("Created users:");
    println!("  Alice ID: {}", alice_id.as_str());
    println!("  Bob ID: {}", bob_id.as_str());

    // Add products (note normalization)
    let laptop_id = service.add_product("Gaming Laptop".to_string(), 129_999)?; // $1299.99
    let mouse_id = service.add_product("wireless-mouse".to_string(), 4999)?; // $49.99

    println!("\nCreated products:");
    println!("  Laptop ID: {} (normalized)", laptop_id.as_str());
    println!("  Mouse ID: {} (normalized)", mouse_id.as_str());

    // Create shopping cart
    let alice_cart = service.create_cart(alice_id.clone())?;
    println!("\nCreated cart: {}", alice_cart.as_str());

    // Add items to cart
    service.add_to_cart(&alice_cart.clone(), laptop_id.clone(), 1)?;
    service.add_to_cart(&alice_cart.clone(), mouse_id.clone(), 2)?;

    println!("Added items to Alice's cart");

    // Create order
    let order_id = service.create_order(&alice_cart)?;
    println!("Created order: {}", order_id.as_str());

    // Display user's orders
    let alice_orders = service.get_user_orders(&alice_id);
    println!("\nAlice's orders:");
    for order in alice_orders {
        println!(
            "  Order {}: ${:.2} ({} items)",
            order.id.as_str(),
            f64::from(order.total) / 100.0,
            order.items.len()
        );
    }

    // Demonstrate type safety
    println!("\n🛡️ Type safety demonstration:");
    println!("  User key domain: {}", alice_id.domain());
    println!("  Product key domain: {}", laptop_id.domain());
    println!("  Order key domain: {}", order_id.domain());

    // This would not compile (type safety):
    // if alice_id == laptop_id { } // Compile error!
    // service.add_to_cart(order_id, laptop_id, 1); // Compile error!

    println!("\n✅ E-commerce example completed successfully!");
    Ok(())
}

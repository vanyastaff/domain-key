# Security Policy

## 🛡️ Supported Versions

We release patches for security vulnerabilities in the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | ✅ Current         |
| < 0.1   | ❌ Not supported   |

## 🚨 Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to: **[your-security-email@example.com]**

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

### What to Include

Please include the requested information listed below (as much as you can provide) to help us better understand the nature and scope of the possible issue:

- **Type of issue** (e.g. buffer overflow, DoS, injection, etc.)
- **Full paths of source file(s)** related to the manifestation of the issue
- **Location of the affected source code** (tag/branch/commit or direct URL)
- **Special configuration required** to reproduce the issue
- **Step-by-step instructions** to reproduce the issue
- **Proof-of-concept or exploit code** (if possible)
- **Impact of the issue**, including how an attacker might exploit it

This information will help us triage your report more quickly.

## 🔒 Security Features

domain-key includes several security features:

### Hash Algorithm Security

- **DoS Protection**: Use the `secure` feature for AHash with HashDoS resistance
- **Cryptographic Security**: Use the `crypto` feature for Blake3 cryptographic hashing
- **Fast but Secure**: GxHash with `fast` feature (requires modern CPU with AES-NI)

```toml
[dependencies]
# For DoS protection
domain-key = { version = "0.1", features = ["secure"] }

# For cryptographic security
domain-key = { version = "0.1", features = ["crypto"] }
```

### Input Validation

- **Length Limits**: Configurable maximum key lengths per domain
- **Character Validation**: Customizable allowed character sets
- **Structure Validation**: Prevention of malformed key patterns
- **Domain Rules**: Custom validation logic per business domain

### Memory Safety

- **No Unsafe Code**: The library forbids unsafe code (`#![forbid(unsafe_code)]`)
- **Bounds Checking**: All array/slice accesses are bounds-checked
- **Memory Efficient**: Smart string allocation prevents excessive memory usage

## 🔍 Security Considerations

### Key Length Attacks

domain-key protects against excessively long keys that could cause:
- Memory exhaustion
- CPU exhaustion during validation
- Hash collision attacks

Each domain can configure appropriate length limits:

```rust
impl KeyDomain for MyDomain {
    const MAX_LENGTH: usize = 64; // Reasonable limit
}
```

### Hash Algorithm Selection

Choose the appropriate hash algorithm for your threat model:

- **Development/Testing**: Default hasher (fast, no security guarantees)
- **Production Web Apps**: `secure` feature (DoS resistant)
- **Cryptographic Applications**: `crypto` feature (cryptographically secure)
- **High-Performance Apps**: `fast` feature (requires hardware AES support)

### Validation Bypass

Custom domain validation should be implemented carefully:

```rust
impl KeyDomain for SecureDomain {
    fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
        // ✅ Good: Explicit validation
        if key.contains("../") {
            return Err(KeyParseError::domain_error(
                Self::DOMAIN_NAME, 
                "Path traversal not allowed"
            ));
        }
        
        // ❌ Bad: Regex without limits
        // let regex = Regex::new(r".*").unwrap(); // DoS risk
        
        Ok(())
    }
}
```

### Serialization Security

When using serde integration:

```rust
// ✅ Good: Validate after deserializing
let key: MyKey = serde_json::from_str(untrusted_input)?;
// Key is automatically validated during deserialization

// ❌ Bad: Using unsafe constructors
// let key = unsafe { MyKey::from_static_unchecked(untrusted_input) };
```

## 🛠️ Security Testing

We maintain comprehensive security testing:

### Automated Security Checks

- **Dependency Audit**: `cargo audit` in CI/CD
- **Clippy Security Lints**: Enhanced security-focused linting
- **Fuzzing**: Property-based testing with edge cases
- **MIRI**: Memory safety validation

### Manual Security Reviews

- Code review for all security-sensitive changes
- Regular security audits of validation logic
- Performance testing for DoS resistance

## 🔄 Security Update Process

1. **Vulnerability Assessment**: Evaluate severity and impact
2. **Patch Development**: Create minimal fix addressing the issue
3. **Testing**: Comprehensive testing including regression tests
4. **Coordinated Disclosure**:
    - Notify affected users
    - Publish security advisory
    - Release patched version
5. **Documentation**: Update security documentation and guidelines

## 📋 Security Checklist for Contributors

When contributing security-sensitive code:

- [ ] No unsafe code blocks
- [ ] Input validation for all user-provided data
- [ ] Bounds checking for all array/slice access
- [ ] Appropriate error handling (no panics on invalid input)
- [ ] DoS resistance (limits on resource usage)
- [ ] Constant-time operations for security-sensitive comparisons
- [ ] Documentation of security assumptions
- [ ] Tests covering security edge cases

## 🌟 Security Best Practices

### For Library Users

1. **Choose Appropriate Features**:
   ```toml
   # Production web application
   domain-key = { version = "0.1", features = ["secure"] }
   
   # Cryptographic application
   domain-key = { version = "0.1", features = ["crypto"] }
   ```

2. **Validate Domain Rules**:
   ```rust
   impl KeyDomain for MyDomain {
       fn validate_domain_rules(key: &str) -> Result<(), KeyParseError> {
           // Add security-relevant validation
           if key.len() > 100 {
               return Err(KeyParseError::TooLong { /* ... */ });
           }
           Ok(())
       }
   }
   ```

3. **Handle Errors Appropriately**:
   ```rust
   match MyKey::new(user_input) {
       Ok(key) => process_key(key),
       Err(e) => {
           // Log but don't expose internal details to users
           log::warn!("Key validation failed: {}", e);
           return Err("Invalid key format");
       }
   }
   ```

4. **Keep Dependencies Updated**:
   ```bash
   cargo audit
   cargo update
   ```

### For Application Developers

1. **Rate Limiting**: Implement rate limiting for key creation endpoints
2. **Input Sanitization**: Sanitize user input before key creation
3. **Logging**: Log security-relevant events (failed validations, etc.)
4. **Monitoring**: Monitor for unusual patterns in key usage

## 📞 Contact Information

- **Security Email**: [your-security-email@example.com]
- **Primary Maintainer**: [@vanyastaff](https://github.com/vanyastaff)
- **GPG Key**: [Optional: Include GPG public key for encrypted communication]

## 🏆 Security Hall of Fame

We'd like to thank the following researchers for responsibly disclosing security issues:

*No vulnerabilities reported yet - be the first!*

---

**Remember**: Security is a shared responsibility. If you use domain-key in a security-sensitive application, please implement appropriate additional security measures at the application level.
# Security Policy

## 🛡️ Supported Versions

We release patches for security vulnerabilities in the following versions:

| Version | Supported          | Status |
| ------- | ------------------ | ------ |
| 0.1.x   | ✅ Current         | Active support with security patches |
| < 0.1   | ❌ Not supported   | Pre-release versions |

## 🚨 Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to: **security@yourdomain.com**

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

domain-key includes several security features designed to protect against common attack vectors:

### Hash Algorithm Security

- **DoS Protection**: Use the `secure` feature for AHash with HashDoS resistance
- **Cryptographic Security**: Use the `crypto` feature for Blake3 cryptographic hashing
- **Fast but Secure**: GxHash with `fast` feature (requires modern CPU with AES-NI)

```toml
[dependencies]
# For DoS protection in web applications
domain-key = { version = "0.1", features = ["secure"] }

# For cryptographic security
domain-key = { version = "0.1", features = ["crypto"] }

# For maximum performance with hardware security
domain-key = { version = "0.1", features = ["fast"] }
```
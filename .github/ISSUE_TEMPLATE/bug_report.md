---
name: Bug report
about: Create a report to help us improve domain-key
title: '[BUG] '
labels: 'bug'
assignees: ''
---

**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Create a domain with '...'
2. Create a key with '...'
3. Call method '...'
4. See error

**Expected behavior**
A clear and concise description of what you expected to happen.

**Code sample**
```rust
// Minimal reproducible example
use domain_key::{Key, KeyDomain};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct TestDomain;

impl KeyDomain for TestDomain {
    const DOMAIN_NAME: &'static str = "test";
}

type TestKey = Key<TestDomain>;

fn main() {
    // Your code that demonstrates the bug
    let key = TestKey::new("example")?;
    // ... issue occurs here
}
```

**Error output**
If applicable, paste the full error message:
```
Error message here
```

**Environment:**
- OS: [e.g. Linux, Windows, macOS]
- Rust version: [e.g. 1.70.0]
- domain-key version: [e.g. 0.1.0]
- Features enabled: [e.g. max-performance, secure]

**Additional context**
Add any other context about the problem here.

**Possible solution**
If you have an idea of what might be causing the issue or how to fix it, please describe it here.
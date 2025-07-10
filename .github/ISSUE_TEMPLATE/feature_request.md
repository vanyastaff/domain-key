---
name: Feature request
about: Suggest an idea for domain-key
title: '[FEATURE] '
labels: 'enhancement'
assignees: ''
---

**Is your feature request related to a problem? Please describe.**
A clear and concise description of what the problem is. Ex. I'm always frustrated when [...]

**Describe the solution you'd like**
A clear and concise description of what you want to happen.

**Describe alternatives you've considered**
A clear and concise description of any alternative solutions or features you've considered.

**Use case**
Describe the specific use case for this feature:
- What domain/industry are you working in?
- How would this feature be used in practice?
- What is the expected frequency of use?

**API design**
If you have ideas about how the API should look, please provide an example:

```rust
// Example of how the feature might be used
use domain_key::{Key, KeyDomain};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct MyDomain;

impl KeyDomain for MyDomain {
    const DOMAIN_NAME: &'static str = "my";
    // New feature configuration here
}

type MyKey = Key<MyDomain>;

fn example_usage() {
    // How the new feature would be used
}
```

**Performance considerations**
- Should this feature prioritize performance or safety?
- Are there any performance requirements or constraints?
- Should this be behind a feature flag?

**Backwards compatibility**
- Should this be a breaking change or backwards compatible?
- If breaking, what migration path would you suggest?

**Additional context**
Add any other context, screenshots, or examples about the feature request here.

**Priority**
How important is this feature to you?
- [ ] Nice to have
- [ ] Would improve my workflow
- [ ] Blocking my project
- [ ] Critical for production use
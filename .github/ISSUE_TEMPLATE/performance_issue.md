---
name: Performance issue
about: Report a performance problem or regression
title: '[PERFORMANCE] '
labels: 'performance'
assignees: ''
---

**Performance issue description**
A clear and concise description of the performance problem.

**Benchmark results**
If you have benchmark results, please include them:

```
// Before (expected):
test key_creation ... bench: 100 ns/iter

// After (actual):
test key_creation ... bench: 500 ns/iter (5x slower!)
```

**Test setup**
Describe your benchmarking setup:
- Hardware: [e.g. Intel i7-9700K, AMD Ryzen 5 3600]
- OS: [e.g. Linux Ubuntu 20.04, Windows 11]
- Rust version: [e.g. 1.70.0]
- domain-key version: [e.g. 0.1.0]
- Features enabled: [e.g. max-performance, secure]
- Compilation flags: [e.g. --release, RUSTFLAGS="-C target-cpu=native"]

**Reproducible example**
Provide code that reproduces the performance issue:

```rust
use domain_key::{Key, KeyDomain};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct TestDomain;

impl KeyDomain for TestDomain {
    const DOMAIN_NAME: &'static str = "test";
}

type TestKey = Key<TestDomain>;

fn main() {
    let start = Instant::now();
    
    // Performance test code here
    for i in 0..100000 {
        let key = TestKey::new(format!("key_{}", i)).unwrap();
        // Do something with key
    }
    
    let duration = start.elapsed();
    println!("Time taken: {:?}", duration);
}
```

**Expected performance**
What performance did you expect? Reference any documentation claims or previous measurements.

**Regression information**
If this is a performance regression:
- Last known good version: [e.g. 0.0.9]
- First bad version: [e.g. 0.1.0]
- Suspected cause: [e.g. new hash algorithm, validation changes]

**Profiling results**
If you've done any profiling, please include relevant results:
- Flamegraphs
- `perf` output
- Memory usage measurements
- CPU usage analysis

**Additional context**
Add any other context about the performance issue here.

**Proposed solution**
If you have ideas for performance improvements, please describe them.
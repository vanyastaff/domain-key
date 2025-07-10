# Pull Request

## Description
Brief description of the changes in this PR.

## Type of change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Performance improvement
- [ ] Code cleanup/refactoring

## Related issues
Fixes #(issue number)

## Testing
- [ ] I have added tests that prove my fix is effective or that my feature works
- [ ] I have run the existing tests and they pass
- [ ] I have tested this change with different feature combinations
- [ ] I have run the benchmarks and verified no performance regressions

**Test commands run:**
```bash
cargo test --all-features
cargo test --no-default-features  
cargo bench --features max-performance
cargo clippy --all-features -- -D warnings
cargo fmt --check
```

## Documentation
- [ ] I have updated the documentation accordingly
- [ ] I have added examples for new features
- [ ] I have updated the CHANGELOG.md
- [ ] I have added inline code documentation

## Performance impact
- [ ] No performance impact
- [ ] Performance improvement (include benchmark results)
- [ ] Potential performance regression (justified and documented)

**Benchmark results** (if applicable):
```
Before: test_name ... bench: 100 ns/iter
After:  test_name ... bench: 90 ns/iter (10% improvement)
```

## Breaking changes
If this PR contains breaking changes, describe:
- What breaks
- Migration path for users
- Justification for the breaking change

## Checklist
- [ ] My code follows the style guidelines of this project
- [ ] I have performed a self-review of my own code
- [ ] I have commented my code, particularly in hard-to-understand areas
- [ ] My changes generate no new warnings
- [ ] Any dependent changes have been merged and published in downstream modules

## Additional context
Add any other context about the PR here.
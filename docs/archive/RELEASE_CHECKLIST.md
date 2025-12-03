# Release Checklist

## Pre-Release

- [ ] All tests pass: `cargo make verify`
- [ ] All examples compile and run
- [ ] Clippy passes: `cargo clippy -- -D warnings`
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] Version numbers updated in `Cargo.toml`

## Release Process

- [ ] Create release branch: `git checkout -b release/v3.x.x`
- [ ] Run final checks: `cargo make release-check`
- [ ] Update CHANGELOG.md with release date
- [ ] Commit: `git commit -m "Release v3.x.x"`
- [ ] Tag: `git tag -a v3.x.x -m "Release v3.x.x"`
- [ ] Push: `git push origin release/v3.x.x && git push --tags`

## Post-Release

- [ ] Verify GitHub Release created automatically
- [ ] Verify crates.io publication successful
- [ ] Check docs.rs documentation updated
- [ ] Merge release branch to main

## Publishing to crates.io

**Note**: Macros crate must be published before main crate.

### Using cargo-make (Recommended)

```bash
cargo make publish-all  # Complete workflow: checks + publish both crates
```

### Step-by-step

```bash
cargo make release-check        # Run all checks
cargo make publish-dry-run-macros  # Verify macros metadata
cargo make publish-macros          # Publish macros crate
cargo make publish-dry-run         # Verify main crate metadata
cargo make publish                 # Publish main crate
cargo make verify-publish          # Verify publication
```

### Manual (if needed)

```bash
cargo publish --dry-run --manifest-path clap-noun-verb-macros/Cargo.toml
cargo publish --manifest-path clap-noun-verb-macros/Cargo.toml
cargo publish --dry-run
cargo publish
```

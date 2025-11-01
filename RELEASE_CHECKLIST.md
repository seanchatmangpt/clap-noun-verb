# Release Checklist

Use this checklist when preparing a new release of clap-noun-verb.

## Pre-Release

- [ ] All tests pass locally
- [ ] All examples compile and run
- [ ] Clippy passes with no warnings
- [ ] Documentation is up to date
- [ ] CHANGELOG.md is updated with new features/changes
- [ ] Version number updated in Cargo.toml
- [ ] README.md is accurate and up to date

## Release Process

- [ ] Create release branch: `git checkout -b release/v1.x.x`
- [ ] Final test: `cargo test --all-features`
- [ ] Build: `cargo build --release`
- [ ] Verify examples: `cargo run --example basic`
- [ ] Update CHANGELOG.md with release date
- [ ] Commit changes: `git commit -m "Release v1.x.x"`
- [ ] Tag release: `git tag -a v1.x.x -m "Release v1.x.x"`
- [ ] Push branch and tags: `git push origin release/v1.x.x && git push --tags`

## Post-Release

- [ ] Verify GitHub Release was created automatically
- [ ] Verify crates.io publication was successful
- [ ] Check docs.rs documentation is updated
- [ ] Announce release (if applicable)
- [ ] Merge release branch to main: `git checkout main && git merge release/v1.x.x`

## Publishing to crates.io

1. Ensure you have a crates.io account token
2. Set up the token: `cargo login <token>`
3. Verify metadata: `cargo publish --dry-run`
4. Publish: `cargo publish`

## Manual Publishing Steps

If GitHub Actions publishing fails:

```bash
# Verify everything is ready
cargo publish --dry-run

# Publish to crates.io
cargo publish

# Verify publication
cargo search clap-noun-verb
```


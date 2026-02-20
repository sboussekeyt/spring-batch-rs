# Documentation Rules — spring-batch-rs

## Two Documentation Systems

This project maintains **two distinct doc systems** that must stay in sync:

| System | Location | Command | Audience |
|---|---|---|---|
| Rustdoc (API ref) | `src/**/*.rs` | `make doc` | Library users, crates.io |
| Website (guides) | `website/src/content/` | `make website-serve` | All users |

## Rustdoc Rules (see also `01-rustdoc.md`)

Run after every change:
```bash
cargo doc --no-deps --all-features 2>&1 | grep -E "warning|error"
```

Zero warnings policy: all rustdoc warnings are treated as errors.

## Website Content Rules

The website uses Astro + DocKit. Content lives in `website/src/content/docs/`.

### When to Update the Website

Update website content when:
- A new reader/writer is added (add to the item-readers-writers section)
- A new example is added (add to the examples section)
- A breaking change is made (update Getting Started + migration guide)
- A new feature flag is introduced (update the features table in README and website)

### Frontmatter Requirements

Every `.mdx` page must have:
```mdx
---
title: Clear, Short Title
description: One sentence for search engines and nav previews.
sidebar:
  order: <number>
---
```

### Code Blocks in Website

Always specify the language and add a title when showing a file:
````mdx
```rust title="examples/csv_processing.rs"
// code here
```
````

Always include a `cargo run` command after code examples:
```mdx
```bash
cargo run --example csv_processing --features csv
```
```

## README.md

The README must always reflect:
- Current version (matches `Cargo.toml`)
- All feature flags (table must be complete)
- Quick start example that compiles with current API
- Links to website and docs.rs

When adding a feature flag:
1. Add row to the features table in README
2. Add row to `Cargo.toml` features section
3. Add entry to the website features page

## CHANGELOG

Location: to be created as `CHANGELOG.md` at project root.

Format (Keep a Changelog):
```markdown
# Changelog

## [Unreleased]

### Added
- ...

### Fixed
- ...

### Changed
- ...

## [0.3.0] - 2024-XX-XX
```

## docs/ Directory

The `docs/` directory is for **internal design documents** only — not user-facing:
- Migration guides
- Architecture decision records (ADRs)
- Builder design rationale

Do NOT put user-facing documentation in `docs/`. It belongs in the website.

## Sync Checklist

When adding a new public type or feature:

- [ ] Rustdoc on the struct/trait/fn
- [ ] Rustdoc on all public methods
- [ ] Module-level `//!` updated
- [ ] At least one doc-test that runs
- [ ] `README.md` features table updated (if new feature flag)
- [ ] Website page updated or created
- [ ] Example added to `examples/`
- [ ] `Cargo.toml` `[[example]]` entry added

## Verification Commands

```bash
# Verify rustdoc builds clean
cargo doc --no-deps --all-features

# Verify all doc-tests pass
cargo test --doc --all-features

# Start website dev server
make website-serve

# Check for broken links in rustdoc
cargo doc --no-deps --all-features 2>&1 | grep "unresolved link"
```

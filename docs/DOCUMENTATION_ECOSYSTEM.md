# Documentation Ecosystem Overview

**Complete map of documentation resources in clap-noun-verb.**

---

## Documentation Pyramid

```
                         ▲
                        /│\
                       / │ \
                      /  │  \
                     / Theory\                  DOCUMENTATION_GUIDE.md
                    /    &    \                 (Comprehensive Reference)
                   / Explanation\
                  /_____________\
                 ▲         ▲         ▲
                /│\       /│\       /│\
               / │ \     / │ \     / │ \       DOCUMENTATION_SKILLS.md
              / How│ \   / Ref\   /Tut \       (Task-Oriented Skills)
             /  -To │ \ /erence \ / orial\
            /__________\________\/______\
           ▲           ▲          ▲        ▲
          /│\         /│\        /│\      /│\
         / │ \   QUICK/│ \ REF  / │ \ EX / │ \
        / Doc \REFERENCE/ API  /Exam\AMPLES/ \    DOCUMENTATION_QUICK_REFERENCE.md
       /__Code_\________/______/ples______\___\   (Cheat Sheet)
```

---

## The Three Documentation Guides

### 1. **DOCUMENTATION_GUIDE.md** — Comprehensive Reference

**Best for:** Complete understanding of all documentation practices

- **Length:** ~1,400 lines
- **Scope:** Everything about documentation
- **Use when:** You need thorough reference material
- **Contains:**
  - Rustdoc generation and publishing
  - Doc comment patterns (all types)
  - Creating guides for new features
  - Managing examples directory
  - Writing ADRs
  - README updates
  - Quality checklists
  - Common mistakes
  - Getting help

**Location:** [docs/DOCUMENTATION_GUIDE.md](DOCUMENTATION_GUIDE.md)

---

### 2. **DOCUMENTATION_SKILLS.md** — Task-Oriented Workflows

**Best for:** Learning specific documentation skills by doing

- **Length:** ~1,820 lines
- **Scope:** Discrete, learnable skills
- **Use when:** You're learning documentation practices
- **Organized as:**
  - Skill: Generating & Publishing Rustdoc
  - Skill: Documentation Comment Patterns
  - Skill: Creating Feature Documentation
  - Skill: Managing Examples
  - Skill: Writing ADRs
  - Skill: Updating README
  - Quality Assurance Workflows
  - Integration with Development

**Each skill includes:**
- Quick reference (commands at a glance)
- Detailed walkthrough (step-by-step)
- Common patterns (reusable templates)
- Validation checklist (quality gates)
- Troubleshooting (how to fix issues)

**Location:** [docs/DOCUMENTATION_SKILLS.md](DOCUMENTATION_SKILLS.md)

---

### 3. **DOCUMENTATION_QUICK_REFERENCE.md** — One-Pager

**Best for:** Quick lookup during active documentation work

- **Length:** ~430 lines
- **Scope:** Most essential information
- **Use when:** You need quick answers
- **Contains:**
  - Commands at a glance
  - Comment templates
  - Example checklist
  - Diataxis quick reference
  - Doc test patterns
  - Project structure map
  - Common mistakes
  - Validation commands

**Location:** [docs/DOCUMENTATION_QUICK_REFERENCE.md](DOCUMENTATION_QUICK_REFERENCE.md)

---

## How to Use This Ecosystem

### Scenario 1: "I'm New to Documenting This Project"

**Path:**
1. Start with [DOCUMENTATION_QUICK_REFERENCE.md](DOCUMENTATION_QUICK_REFERENCE.md) — Get overview
2. Read [DOCUMENTATION_SKILLS.md](DOCUMENTATION_SKILLS.md) — Pick a relevant skill section
3. Follow the step-by-step walkthrough for your task
4. Use templates and checklists provided

**Time:** 15-30 minutes to understand, then apply

---

### Scenario 2: "I Need Comprehensive Documentation Standards"

**Path:**
1. Read [DOCUMENTATION_GUIDE.md](DOCUMENTATION_GUIDE.md) — Full reference
2. Keep [DOCUMENTATION_QUICK_REFERENCE.md](DOCUMENTATION_QUICK_REFERENCE.md) as bookmark
3. Reference specific sections as needed
4. Use templates for consistent quality

**Time:** 1-2 hours for thorough understanding

---

### Scenario 3: "I'm Writing a How-To Guide Right Now"

**Path:**
1. Open [DOCUMENTATION_QUICK_REFERENCE.md](DOCUMENTATION_QUICK_REFERENCE.md) for quick commands
2. Jump to [DOCUMENTATION_SKILLS.md](DOCUMENTATION_SKILLS.md) → "Skill: Creating Feature Documentation"
3. Follow Step 2: "Write a How-To Guide"
4. Use the template provided
5. Check off validation checklist before committing

**Time:** 5-10 minutes for reference, then your writing time

---

### Scenario 4: "I'm Creating Doc Tests"

**Path:**
1. Check [DOCUMENTATION_QUICK_REFERENCE.md](DOCUMENTATION_QUICK_REFERENCE.md) → "Doc Test Special Syntax"
2. See examples of ✅ Good vs ❌ Bad
3. Follow patterns in [DOCUMENTATION_SKILLS.md](DOCUMENTATION_SKILLS.md) → "Skill: Documentation Comment Patterns"
4. Test with `cargo test --doc`

**Time:** 5 minutes

---

### Scenario 5: "I'm Publishing a Release"

**Path:**
1. Review [DOCUMENTATION_SKILLS.md](DOCUMENTATION_SKILLS.md) → "Skill: Generating & Publishing Rustdoc" → Step 5: Publish
2. Follow checklist in [DOCUMENTATION_GUIDE.md](DOCUMENTATION_GUIDE.md) → "README Updates"
3. Use commands from [DOCUMENTATION_QUICK_REFERENCE.md](DOCUMENTATION_QUICK_REFERENCE.md) to validate

**Time:** 10-15 minutes

---

## Content Map

### By Task

| Task | Quick Ref | Skills | Full Guide |
|------|-----------|--------|-----------|
| Generate rustdoc | ✅ | ✅ | ✅ |
| Write doc comments | ✅ | ✅ | ✅ |
| Create examples | ✅ | ✅ | ✅ |
| Write ADRs | ✅ | ✅ | ✅ |
| Update README | — | ✅ | ✅ |
| Create feature docs | — | ✅ | ✅ |
| Quality checklists | ✅ | ✅ | ✅ |

### By Documentation Type

| Type | Quick Ref | Skills | Full Guide |
|------|-----------|--------|-----------|
| Tutorials | — | — | ✅ |
| How-To Guides | — | ✅ | ✅ |
| Reference | ✅ | ✅ | ✅ |
| Explanations | — | — | ✅ |
| Doc comments | ✅ | ✅ | ✅ |
| Examples | ✅ | ✅ | ✅ |
| ADRs | ✅ | ✅ | ✅ |

---

## Complementary Resources

### In This Repository

| Document | Purpose |
|----------|---------|
| [README.md](../README.md) | User-facing project overview |
| [CONTRIBUTING.md](../CONTRIBUTING.md) | Code contribution guidelines |
| [CHANGELOG.md](../CHANGELOG.md) | Version history and changes |
| [CLAUDE.md](../CLAUDE.md) | Development workflow & architecture |
| [examples/README.md](../examples/README.md) | Example directory navigation |
| [docs/tutorial/](tutorial/) | Step-by-step learning guides |
| [docs/howto/](howto/) | Task-oriented guides |
| [docs/reference/](reference/) | API reference documentation |
| [docs/explanation/](explanation/) | Architecture & design explanations |

### External References

| Resource | For |
|----------|-----|
| [Diataxis Framework](https://diataxis.fr/) | Documentation structure philosophy |
| [Rust Documentation Guide](https://doc.rust-lang.org/rustdoc/) | Rustdoc syntax and features |
| [ADR Guide](https://adr.github.io/) | Architecture Decision Records |
| [Cargo Book](https://doc.rust-lang.org/cargo/) | Cargo features and publishing |

---

## The Three Levels of Documentation Detail

### Level 1: Quick Reference (5 minutes)

**What:** One-pager with essential commands and templates

**When:** During active documentation work, need quick lookup

**File:** `DOCUMENTATION_QUICK_REFERENCE.md`

**Examples:**
- "What's the syntax for doc test special syntax?"
- "Show me the module doc template"
- "How do I run doc tests?"

---

### Level 2: Task-Oriented Skills (15-30 minutes)

**What:** Focused skill sections with full walkthrough

**When:** Learning a documentation practice, need guidance

**File:** `DOCUMENTATION_SKILLS.md`

**Examples:**
- "Walk me through creating a feature"
- "How do I write an ADR?"
- "Show me how to manage examples"

---

### Level 3: Comprehensive Reference (1-2 hours)

**What:** Complete guide covering all aspects

**When:** Need thorough understanding, studying practices deeply

**File:** `DOCUMENTATION_GUIDE.md`

**Examples:**
- "What are all the documentation patterns?"
- "Show me complete error handling docs"
- "What are common mistakes?"

---

## Integration Points

### With Code Changes

```
Code change in src/
  ↓
Update doc comments (same file)
  ↓
Test with: cargo test --doc
  ↓
If new API/feature:
  ├→ Create/update examples
  ├→ Create/update howto guide
  ├→ Create/update reference page
  └→ Update examples/README.md
  ↓
Update README.md if user-facing
  ↓
Run: cargo make doc
  ↓
Commit: docs: [type] [subject]
```

### With Releases

```
1. Update version in Cargo.toml
2. Update CHANGELOG.md
3. Update README.md "What's New"
4. Update feature table if applicable
5. Test examples: cargo run --example <name>
6. Verify all links work
7. Run: cargo make doc
8. Publish: cargo make publish-macros && cargo make publish
```

### With Architecture Changes

```
Architectural decision made
  ↓
Document in ADR (docs/adr/NNNN-title.md)
  ↓
Link from related ADRs
  ↓
Update docs/explanation/ if affects understanding
  ↓
Create/update examples as needed
  ↓
Update README.md with context
```

---

## Quality Gates

All documentation passes through:

```
cargo test --doc          # Doc tests compile and pass
cargo make format-check   # Formatting standard
cargo make lint           # No linting errors
cargo make build-examples # Examples compile
cargo make doc            # Docs generate cleanly
```

---

## Tips for Navigating

### Finding Information Fast

**Q: How do I write doc comments?**
- Quick answer: DOCUMENTATION_QUICK_REFERENCE.md → "Documentation Comment Templates"
- Full examples: DOCUMENTATION_GUIDE.md → "Documentation Comment Patterns"
- Walkthrough: DOCUMENTATION_SKILLS.md → "Skill: Documentation Comment Patterns"

**Q: How do I create a new example?**
- Quick answer: DOCUMENTATION_QUICK_REFERENCE.md → "Example Quality Checklist"
- Full details: DOCUMENTATION_GUIDE.md → "Managing Examples"
- Step-by-step: DOCUMENTATION_SKILLS.md → "Skill: Managing Examples"

**Q: How do I write an ADR?**
- Template: DOCUMENTATION_QUICK_REFERENCE.md → "ADR Quick Template"
- Full guide: DOCUMENTATION_GUIDE.md → "Writing Architecture Decision Records"
- Walkthrough: DOCUMENTATION_SKILLS.md → "Skill: Writing ADRs"

### Best Practices

1. **Start small:** Use quick reference for initial lookup
2. **Learn by doing:** Follow skills walkthrough while working
3. **Reference as needed:** Use full guide for comprehensive understanding
4. **Keep checklists:** Use validation checklists before committing
5. **Ask questions:** If stuck, check troubleshooting sections

---

## Maintenance Schedule

| Document | Update Frequency | Trigger |
|----------|-----------------|---------|
| Quick Reference | Per quarter | New major patterns |
| Skills Guide | Per quarter | New skills/workflows |
| Full Guide | Per quarter | New practices |
| Examples README | Per example added | New example |
| README.md | Per release | New features |

---

## Contributing to Documentation

When improving these guides:

1. **Quick Reference:** Keep ≤500 lines, update most essential info first
2. **Skills Guide:** Add new skills in same format as existing ones
3. **Full Guide:** Add comprehensive material with examples

All updates should pass:
```bash
cargo make format-check  # Formatting
cargo make lint          # Linting
```

---

## Version Compatibility

This documentation ecosystem applies to **clap-noun-verb 26.6.13+**.

For older versions, check the specific version's documentation.

---

**Last Updated:** 2026-06-14

**Scope:** clap-noun-verb 26.6.13+

See also:
- [DOCUMENTATION_GUIDE.md](DOCUMENTATION_GUIDE.md) — Comprehensive reference
- [DOCUMENTATION_SKILLS.md](DOCUMENTATION_SKILLS.md) — Task-oriented workflows
- [DOCUMENTATION_QUICK_REFERENCE.md](DOCUMENTATION_QUICK_REFERENCE.md) — Quick lookup

# Book v2.0 Rewrite Plan: Comprehensive Update

## Overview

Complete rewrite plan for all book documents to ensure they reflect ggen v2.0 architecture perfectly. This plan addresses all critical issues identified in the pre-implementation checklist.

**Status**: Plan created - awaiting approval before implementation  
**Files to Update**: 8 markdown source files  
**Critical Issues to Address**: Async/sync compatibility, v2.0 architecture, command renames

---

## Critical Issues to Address

### 1. Async/Sync Compatibility ⚠️ **CRITICAL**
- **Issue**: All book examples show sync functions, but ggen has 94 async functions
- **Required**: Add async/sync wrapper pattern examples throughout
- **Impact**: All command examples need updating

### 2. v2.0 Architecture Patterns
- **Required**: Update examples to show pure RDF-driven templates
- **Required**: Show business logic separation
- **Required**: Update command syntax (`market` → `marketplace`, `doctor` → `utils doctor`)
- **Required**: Remove `--var` flags, add `--rdf` flag

### 3. Error Handling
- **Required**: Remove all `unwrap()` / `expect()` from examples
- **Required**: Show proper error handling patterns

### 4. Code Examples
- **Required**: All examples must compile
- **Required**: All examples must reflect v2.0 architecture
- **Required**: Add async wrapper pattern examples

---

## File-by-File Rewrite Plan

### 1. `introduction.md` - Update Overview & Examples

**Current State**: Introduces ggen porting, shows basic v3.0.0 examples

**Required Changes**:

1. **Add Async/Sync Pattern Section**
   ```rust
   // Add new section explaining async/sync compatibility
   ```

2. **Update Command Examples**
   - Change `market` → `marketplace`
   - Change root commands → `utils` noun
   - Remove `--var` flags
   - Add `--rdf` flag examples

3. **Add Business Logic Separation Examples**
   ```rust
   // Show CLI layer vs business logic separation
   ```

4. **Update All Code Examples**
   - Ensure no `unwrap()` / `expect()`
   - Add proper error handling
   - Add async wrapper examples

**Sections to Update**:
- "About ggen" section (commands)
- "Example: After" section (complete rewrite)
- "Benefits" section (add async/sync, business logic separation)

---

### 2. `getting-started.md` - Add Async/Sync Setup

**Current State**: Shows basic setup, type inference, auto-inference

**Required Changes**:

1. **Add Async/Sync Compatibility Section** ⚠️ **CRITICAL**
   ```rust
   // NEW SECTION: Async/Sync Compatibility
   // Explain that clap-noun-verb is sync-only
   // Show how to wrap async business logic
   ```

2. **Add Async Wrapper Pattern Example**
   ```rust
   // Show sync CLI wrapper with tokio::runtime::Runtime::block_on()
   ```

3. **Update Project Structure Section**
   - Show business logic separation
   - Show domain/ structure for business logic

4. **Update Examples**
   - Add async business logic examples
   - Show sync CLI wrapper pattern
   - Remove any `unwrap()` / `expect()`

**New Section Required**:
```markdown
## Async/Sync Compatibility

ggen has async functions, but clap-noun-verb v3.0.0 is sync-only. 
We handle this by creating sync CLI wrappers that spawn async runtimes.
```

---

### 3. `analyzing-structure.md` - Update Command Mapping

**Current State**: Maps ggen commands to noun-verb pattern

**Required Changes**:

1. **Update Command Names**
   - `market` → `marketplace` (all occurrences)
   - `doctor` → `utils doctor`
   - `help-me` → `utils help-me`
   - `ggen gen` → `ggen template generate`

2. **Add Async Business Logic Examples**
   - Show async business logic functions
   - Show sync CLI wrappers

3. **Update Command Mapping Examples**
   - Remove `--var` flags
   - Add `--rdf` flag requirements
   - Show proper error handling

4. **Add Business Logic Separation**
   - Show domain/ structure
   - Show separation pattern

**Sections to Update**:
- "Overview of ggen commands" (command names)
- "Identifying nouns" (market → marketplace)
- "Mapping commands to noun-verb structure" (all examples)

---

### 4. `porting-commands.md` - Complete Rewrite for v2.0

**Current State**: Shows before/after examples for each command group

**Required Changes**:

1. **Add Async/Sync Pattern to All Examples** ⚠️ **CRITICAL**
   ```rust
   // Every command example needs:
   // 1. Async business logic function
   // 2. Sync CLI wrapper with runtime
   ```

2. **Update All Before Examples**
   - Show current ggen async pattern
   - Show how to extract business logic

3. **Update All After Examples**
   - Show sync CLI wrapper
   - Show async business logic delegation
   - Remove `unwrap()` / `expect()`

4. **Add Business Logic Separation Examples**
   ```rust
   // Show:
   // - CLI layer (sync, thin wrapper)
   // - Business logic (async, in domain/)
   ```

5. **Update Command Names**
   - `market` → `marketplace`
   - Root commands → `utils`

6. **Update Argument Examples**
   - Remove `--var` flags
   - Add `--rdf` flag
   - Show RDF-driven approach

**Sections Requiring Major Updates**:
- All "Before" examples (show async pattern)
- All "After" examples (show sync wrapper + async business logic)
- AI commands section
- Marketplace commands section
- Template commands section

---

### 5. `advanced-patterns.md` - Add Async Patterns

**Current State**: Shows nested commands, error handling, custom implementations

**Required Changes**:

1. **Add Async/Sync Patterns Section** ⚠️ **NEW SECTION**
   ```markdown
   ## Async/Sync Compatibility Patterns
   
   How to handle async business logic with sync CLI layer
   ```

2. **Add Async Wrapper Patterns**
   ```rust
   // Show different async patterns:
   // - Simple wrapper
   // - Error handling wrapper
   // - Multiple async calls
   ```

3. **Update Error Handling Section**
   - Remove all `unwrap()` / `expect()`
   - Add async error handling examples
   - Show error propagation through async

4. **Add Business Logic Separation Patterns**
   ```rust
   // Show how to organize async business logic
   ```

5. **Update All Examples**
   - Ensure they compile
   - Ensure proper error handling
   - Show async/sync patterns

**New Section Required**:
```markdown
## Async/Sync Compatibility

Full section on handling async business logic with sync CLI layer
```

---

### 6. `testing-validation.md` - Add Async Testing

**Current State**: Shows testing strategies for CLI commands

**Required Changes**:

1. **Add Async Testing Section** ⚠️ **NEW SECTION**
   ```markdown
   ## Testing Async Business Logic
   
   How to test async functions in sync CLI wrappers
   ```

2. **Update Testing Examples**
   - Add async test examples
   - Show how to test sync wrappers
   - Show how to test async business logic

3. **Add Mock Patterns**
   ```rust
   // Show how to mock async operations
   ```

4. **Update Integration Test Examples**
   - Show async integration tests
   - Show runtime spawning in tests

**New Section Required**:
```markdown
## Testing Async/Sync Compatibility

Complete section on testing async business logic with sync CLI layer
```

---

### 7. `migration-checklist.md` - Add v2.0 Migration Steps

**Current State**: General migration checklist

**Required Changes**:

1. **Add Async/Sync Migration Steps** ⚠️ **NEW SECTION**
   ```markdown
   ## Async/Sync Compatibility Migration
   
   Steps for migrating async commands to sync CLI wrappers
   ```

2. **Add v2.0-Specific Checklist Items**
   - Async/sync wrapper creation
   - Business logic extraction
   - RDF-driven template migration
   - Command rename migration

3. **Update Common Pitfalls**
   - Add async/sync pitfalls
   - Add business logic separation pitfalls
   - Add RDF-driven pitfalls

4. **Update Best Practices**
   - Add async/sync best practices
   - Add business logic separation best practices

**New Sections Required**:
- "Async/Sync Compatibility Migration"
- "Business Logic Separation Migration"
- "RDF-Driven Template Migration"

---

### 8. `SUMMARY.md` - Update Navigation

**Current State**: Lists all chapters

**Required Changes**:

1. **Add New Sections** (if needed)
   - Async/Sync Compatibility (if we add dedicated chapter)

2. **Update Chapter Descriptions**
   - Reflect v2.0 architecture focus
   - Mention async/sync patterns
   - Mention business logic separation

**No Major Rewrite Needed**: Just ensure navigation is accurate

---

## Common Changes Across All Files

### 1. Command Names
- ✅ `market` → `marketplace` (all occurrences)
- ✅ `doctor` → `utils doctor`
- ✅ `help-me` → `utils help-me`
- ✅ `ggen gen` → `ggen template generate`

### 2. Remove `unwrap()` / `expect()`
- ✅ Replace all with proper error handling
- ✅ Use `Result<T>` types
- ✅ Show proper error propagation

### 3. Add Async/Sync Examples
- ✅ Show async business logic functions
- ✅ Show sync CLI wrappers
- ✅ Show runtime spawning pattern

### 4. Update Argument Examples
- ✅ Remove `--var` flags
- ✅ Add `--rdf` flag
- ✅ Show RDF-driven approach

### 5. Add Business Logic Separation
- ✅ Show domain/ structure
- ✅ Show CLI vs business logic separation
- ✅ Show delegation pattern

### 6. Ensure Examples Compile
- ✅ All examples must be valid Rust
- ✅ All imports must be correct
- ✅ All types must be defined

---

## Implementation Order

### Phase 1: Critical Updates (Async/Sync)
1. `introduction.md` - Add async/sync overview
2. `getting-started.md` - Add async/sync setup section
3. `porting-commands.md` - Update all examples with async wrappers

### Phase 2: Command Updates
4. `analyzing-structure.md` - Update command names
5. `porting-commands.md` - Update command names in examples

### Phase 3: Architecture Updates
6. All files - Add business logic separation examples
7. All files - Remove `--var`, add `--rdf`
8. All files - Remove `unwrap()` / `expect()`

### Phase 4: Advanced Patterns
9. `advanced-patterns.md` - Add async/sync patterns section
10. `testing-validation.md` - Add async testing section
11. `migration-checklist.md` - Add v2.0 migration steps

### Phase 5: Final Review
12. All files - Verify examples compile
13. All files - Verify consistency
14. All files - Final review

---

## Validation Checklist

After rewriting each file:

- [ ] All command names updated (`market` → `marketplace`, etc.)
- [ ] All examples show async/sync pattern
- [ ] No `unwrap()` / `expect()` in examples
- [ ] All examples compile
- [ ] Business logic separation shown
- [ ] RDF-driven approach shown
- [ ] Error handling proper
- [ ] Consistent with v2.0 architecture

---

## Success Criteria

**All book documents are perfect for v2.0 when**:

1. ✅ Every command example shows async/sync wrapper pattern
2. ✅ All command names reflect v2.0 renames
3. ✅ All examples show business logic separation
4. ✅ All examples use RDF-driven approach
5. ✅ Zero `unwrap()` / `expect()` in examples
6. ✅ All examples compile
7. ✅ All examples consistent with v2.0 architecture
8. ✅ All cross-references accurate

---

## Next Steps

1. **Review this plan** - Confirm all changes needed
2. **Approve implementation** - Allow edits to begin
3. **Implement Phase 1** - Critical async/sync updates
4. **Implement Phase 2** - Command name updates
5. **Implement Phase 3** - Architecture updates
6. **Implement Phase 4** - Advanced patterns
7. **Implement Phase 5** - Final review

---

**Last Updated**: Book v2.0 rewrite plan created


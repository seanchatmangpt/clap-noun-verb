# ggen Implementation Strategy: Direct Refactoring

## Principle: Clean Implementation

**Strategy**: Direct implementation to v2.0.0 architecture. No migration needed - project has no users. Clean break with all breaking changes implemented directly.

**Benefits**:
- **Cleaner codebase** - No legacy code to maintain
- **Simpler implementation** - No alias routing logic
- **Faster development** - Focus on new structure only
- **Better UX** - Consistent noun-verb pattern everywhere

---

## Breaking Changes Summary

### 1. Command Renames (16 commands affected)

#### Marketplace Rename (14 commands)
```bash
# OLD → NEW
ggen market search      → ggen marketplace search
ggen market add         → ggen marketplace add
ggen market remove      → ggen marketplace remove
ggen market list        → ggen marketplace list
ggen market update      → ggen marketplace update
ggen market info        → ggen marketplace info
ggen market recommend   → ggen marketplace recommend
ggen market offline     → ggen marketplace offline
ggen market cache       → ggen marketplace cache
ggen market sync        → ggen marketplace sync
ggen market categories  → ggen marketplace categories
ggen market publish     → ggen marketplace publish
ggen market unpublish   → ggen marketplace unpublish
ggen market natural     → ggen marketplace natural
```

#### Utils Grouping (2 commands)
```bash
# OLD → NEW
ggen doctor             → ggen utils doctor
ggen help-me            → ggen utils help-me
```

#### Legacy Command Removal (1 command)
```bash
# OLD → NEW
ggen gen <template>     → ggen template generate <template>
                        # OR
                        → ggen project gen <template>
                        # (Remove entirely if redundant)
```

### 2. API Changes (Breaking)

#### CLI Library API
- **Old**: Manual enum-based command registration
- **New**: Auto-discovery with `#[verb]` attributes
- **Migration**: Rewrite all command handlers using `#[verb]` attributes

#### Business Logic API
- **Old**: Mixed CLI and business logic
- **New**: Separated business logic with pure functions
- **Migration**: Extract business logic from CLI handlers

#### Output Format
- **Old**: Plain text output by default
- **New**: JSON output by default
- **Migration**: Update all scripts to parse JSON instead of text

---

## Implementation Notes

**No Migration Tools Needed**: Since the project has no users, we can directly implement all breaking changes. No migration tools, no compatibility layer - just clean implementation.

---

## Implementation Timeline

### Phase 1: Preparation (Week 1)

**Goal**: Plan and document

**Tasks**:
- [ ] Document all breaking changes
- [ ] Create command mapping table
- [ ] Update documentation templates
- [ ] Plan implementation order

**Deliverables**:
- Command mapping reference
- Breaking changes list
- Implementation plan

### Phase 2: Implementation (Week 2-4)

**Goal**: Implement new CLI structure

**Tasks**:
- [ ] Migrate all commands to `#[verb]` attributes
- [ ] Rename `market` → `marketplace` (all 14 commands)
- [ ] Group `doctor` → `utils.doctor`
- [ ] Group `help-me` → `utils.help-me`
- [ ] Remove legacy `ggen gen` command
- [ ] Update all command handlers
- [ ] Implement JSON output for all commands
- [ ] Extract business logic from CLI handlers

**Deliverables**:
- Complete CLI migration
- All 69+ commands migrated
- Business logic separated
- JSON output implemented

### Phase 3: Testing (Week 4-5)

**Goal**: Comprehensive testing

**Tasks**:
- [ ] Unit tests for all commands
- [ ] Integration tests for command parsing
- [ ] JSON output validation tests
- [ ] End-to-end workflow tests
- [ ] Performance benchmarks

**Deliverables**:
- All tests passing
- Performance benchmarks
- Test coverage report

### Phase 4: Release (Week 5)

**Goal**: Documentation and release

**Tasks**:
- [ ] Update all documentation
- [ ] Update examples
- [ ] Final testing
- [ ] Release v2.0.0

**Deliverables**:
- Complete documentation
- Final v2.0.0 release

---

## Command Mapping Reference

### Complete Command Map

| Old Command | New Command | Notes |
|------------|------------|-------|
| `ggen market search` | `ggen marketplace search` | All 14 marketplace commands renamed |
| `ggen market add` | `ggen marketplace add` | |
| `ggen market remove` | `ggen marketplace remove` | |
| `ggen market list` | `ggen marketplace list` | |
| `ggen market update` | `ggen marketplace update` | |
| `ggen market info` | `ggen marketplace info` | |
| `ggen market recommend` | `ggen marketplace recommend` | |
| `ggen market offline` | `ggen marketplace offline` | |
| `ggen market cache` | `ggen marketplace cache` | |
| `ggen market sync` | `ggen marketplace sync` | |
| `ggen market categories` | `ggen marketplace categories` | |
| `ggen market publish` | `ggen marketplace publish` | |
| `ggen market unpublish` | `ggen marketplace unpublish` | |
| `ggen market natural` | `ggen marketplace natural` | |
| `ggen doctor` | `ggen utils doctor` | Root-level → grouped under utils |
| `ggen help-me` | `ggen utils help-me` | Root-level → grouped under utils |
| `ggen gen` | `ggen template generate` | Legacy command → full noun-verb |

### All Other Commands (Unchanged)

All other commands already follow noun-verb pattern and remain unchanged:
- `ggen ai.*` - No changes (10 verbs)
- `ggen project.*` - No changes (10 verbs)
- `ggen template.*` - No changes (6 verbs, except `gen` removal)
- `ggen hook.*` - No changes (5 verbs)
- `ggen graph.*` - No changes (7 verbs)
- `ggen audit.*` - No changes (3 verbs)
- `ggen ci.*` - No changes (4 verbs)
- `ggen lifecycle.*` - No changes (9 verbs)
- `ggen shell.*` - No changes (1 verb)

**Total Unchanged**: 55 verbs (no migration needed)

---

## Implementation Checklist

### For Developers

- [ ] Review breaking changes list
- [ ] Update all code examples
- [ ] Update all tests
- [ ] Update documentation
- [ ] Implement all commands with `#[verb]` attributes
- [ ] Extract business logic from CLI handlers
- [ ] Implement JSON output for all commands
- [ ] Update all examples
- [ ] Prepare release notes

---

## Risk Mitigation

### Risk 1: Breaking Changes Not Documented

**Mitigation**:
- Comprehensive breaking changes list
- Clear command mapping reference
- Updated documentation
- Helpful error messages

### Risk 2: Implementation Incomplete

**Mitigation**:
- Complete checklist
- Comprehensive testing
- Code review
- Performance benchmarks

### Risk 3: API Inconsistencies

**Mitigation**:
- Consistent noun-verb pattern
- Type inference throughout
- JSON output for all commands
- Clear documentation

---

## Success Metrics

### Implementation Success

- **Command Migration**: 100% of commands migrated to `#[verb]` attributes
- **Test Coverage**: 90%+ test coverage maintained
- **Performance**: <5% performance regression
- **Documentation**: 100% documentation updated

### Quality Metrics

- **Code Quality**: All breaking changes implemented cleanly
- **API Consistency**: 100% noun-verb pattern adherence
- **JSON Output**: All commands return structured JSON
- **Type Safety**: All arguments type-inferred

---

## Next Steps

1. **Review this strategy** - Confirm breaking changes approach
2. **Begin implementation** - Start Phase 2 implementation
3. **Update documentation** - Document all changes as we implement
4. **Testing** - Comprehensive testing throughout
5. **Release** - Final v2.0.0 release

---

**Last Updated**: Implementation strategy updated for direct refactoring. No migration needed - clean implementation.


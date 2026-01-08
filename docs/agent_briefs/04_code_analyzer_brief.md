# Code Analyzer Brief - v6.0.0 Backward Compatibility

**Agent ID**: code-analyzer-v6
**Memory Key**: backward_compatibility_analysis
**Dependencies**: Awaits v6_specification from Specification Agent
**Timeline**: Start at +10 min

## Mission
Deep-dive analysis of backward compatibility impact, document all public API breaking changes, and create detailed migration guide for v5.5.0 → v6.0.0.

## Work Steps

1. **Poll v6_specification** (2 min)
   - Retrieve breaking changes from Specification Agent
   - Extract detailed API change requirements
   - Get acceptance criteria for testing

2. **Analyze Public API Surface** (8 min)
   - Review current public APIs:
     - Exported types, functions, traits, macros
     - Current error handling approach
     - Current builder patterns
   - Map to breaking changes specified

3. **Document Migration Paths** (10 min)
   - For each breaking change, create:
     - Detailed migration guide (step-by-step)
     - Code examples: old code → new code
     - Common pitfalls and how to avoid
     - Deprecation warnings (if gradual migration possible)
   - Create comprehensive migration document

4. **Identify Unsafe Patterns** (5 min)
   - Find any unsafe code that needs review
   - Check for unwrap/panic patterns
   - Identify error handling gaps
   - Flag for Security Officer and Code Reviewer

5. **Store in Memory** (1 min)
   - Save backward_compatibility_analysis findings
   - Include full migration guide
   - Code examples ready for Production Validator

## Deliverables

### Public API Changes Document
```
## Breaking Change 1: [Name]
- Old API: [signature]
- New API: [signature]
- Reason: [justification]
- Migration: [step-by-step guide]
- Examples: [before/after code]

## Breaking Change 2: [Name]
[same structure]
```

### Migration Guide
- Complete v5.5.0 → v6.0.0 migration instructions
- Organized by feature area
- Include troubleshooting section
- Real-world examples

### Unsafe Code Audit
- List all unsafe blocks with justification
- Identify patterns needing fixing
- Security concerns flagged

## Constraints
- Migration guide must be clear to library users
- Examples must compile and run correctly
- No misleading or incomplete guides
- Identify ALL public API breaking changes

## Success Criteria
- ✅ All breaking changes documented with migration paths
- ✅ Step-by-step migration guide created
- ✅ Code examples for migration complete
- ✅ Unsafe patterns identified and flagged
- ✅ Memory key backward_compatibility_analysis populated
- ✅ Clear for Production Validator to use in documentation

## Dependencies
- **Awaits**: v6_specification (needs breaking change details)
- **Provides**: Detailed migration guide for Production Validator
- **Provides**: Code examples for Release Notes

## Notes
- Completeness is CRITICAL - users depend on clear migration paths
- Every breaking change needs a migration example
- This guide becomes the official migration documentation
- Review actual code in /home/user/clap-noun-verb/src/ to ensure accuracy

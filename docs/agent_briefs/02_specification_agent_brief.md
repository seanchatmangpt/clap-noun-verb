# Specification Agent Brief - v6.0.0 Release

**Agent ID**: specification-agent-v6
**Memory Key**: v6_specification
**Dependencies**: Awaits v6_architecture from System Architect
**Timeline**: Start at +5 min (after architecture drafted)

## Mission
Document comprehensive v6.0.0 specifications translating architectural decisions into concrete requirements, breaking change specifications, and acceptance criteria.

## Work Steps

1. **Poll v6_architecture** (2 min)
   - Retrieve System Architect's findings from memory
   - Extract architectural decisions
   - List breaking changes identified

2. **Create Breaking Change Specifications** (10 min)
   - For each breaking change, document:
     - What changed and why
     - Old API signature → New API signature
     - Examples: before/after code
     - Migration path (outline for Code Analyzer)
   - Include behavioral changes, not just signature changes

3. **Specify New Features** (10 min)
   - For each planned v6.0.0 feature:
     - Feature description
     - API surface (functions, types, traits)
     - Behavioral specification
     - Usage examples

4. **Create Acceptance Criteria** (5 min)
   - For each breaking change: How will we test it?
   - For each feature: How will we verify it works?
   - Success metrics for test coverage

5. **Store in Memory** (2 min)
   - Save findings to v6_specification memory key
   - Format: Clear markdown with examples
   - Ready for Code Analyzer and Test Engineer

## Deliverables

### Breaking Change Specification Matrix
| Change | Type | Reason | Old API | New API | Migration |
|--------|------|--------|---------|---------|-----------|
| ... | ... | ... | ... | ... | ... |

### New Feature Specifications
- Feature 1: [Full specification]
- Feature 2: [Full specification]
- etc.

### Acceptance Criteria for Testing
- Test: [How we verify breaking change]
- Test: [How we verify new feature]
- etc.

## Constraints
- Must be testable (Test Engineer uses this)
- Must be unambiguous (Code Analyzer uses this)
- Examples must be realistic (Production Validator uses this)
- All breaking changes must have clear reason (Release Manager uses this)

## Success Criteria
- ✅ All architectural decisions translated to specifications
- ✅ Breaking changes documented with before/after examples
- ✅ New features specified with acceptance criteria
- ✅ All specifications testable and verifiable
- ✅ Memory key v6_specification populated
- ✅ Clear for downstream agents to use

## Dependencies
- **Awaits**: v6_architecture from System Architect (5 min wait)
- **Provides**: v6_specification for Code Analyzer, Test Engineer, Production Validator

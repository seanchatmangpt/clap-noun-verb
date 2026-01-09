# Production Validator Brief - v6.0.0 Release Documentation

**Agent ID**: production-validator-v6
**Memory Key**: release_documentation
**Dependencies**: Awaits backward_compatibility_analysis from Code Analyzer
**Timeline**: Start at +15 min

## Mission
Create comprehensive, production-ready v6.0.0 release documentation including breaking change guide, migration instructions, changelog, and release notes suitable for publication.

## Work Steps

1. **Poll backward_compatibility_analysis** (2 min)
   - Retrieve Code Analyzer's migration guide
   - Extract breaking changes documentation
   - Get code examples

2. **Create Breaking Change Guide** (8 min)
   - Professional documentation of all breaking changes
   - Format: Clear, concise, well-structured
   - For each change:
     - What changed and why
     - How it affects users
     - Step-by-step migration instructions
     - Code examples (before/after)
   - Include tips and best practices

3. **Create Migration Guide** (10 min)
   - Complete step-by-step upgrade guide
   - Organized by user scenarios (not features)
   - Include troubleshooting section
   - Common mistakes and solutions
   - Testing recommendations after upgrade

4. **Generate CHANGELOG** (8 min)
   - Professional, marketing-ready changelog
   - Format: Grouped by feature area
   - Each item includes:
     - What changed
     - Impact on users
     - Link to migration guide if breaking
   - Keep tone professional but friendly

5. **Create Release Notes** (5 min)
   - Marketing-focused release announcement
   - Key improvements and features
   - Performance improvements (if any)
   - Security fixes (if any)
   - Gratitude to contributors
   - Links to guides

6. **Store in Memory** (2 min)
   - Save release_documentation findings
   - Include all 4 documents above
   - Ready for Release Manager's publication

## Deliverables

### 1. Breaking Changes Guide
- File: `/docs/v6_BREAKING_CHANGES.md`
- Professional, complete reference
- All breaking changes with context

### 2. Migration Guide
- File: `/docs/v6_MIGRATION_GUIDE.md`
- Step-by-step instructions
- Real-world examples
- Troubleshooting

### 3. CHANGELOG
- File: `CHANGELOG.md` (update for v6.0.0)
- Standard changelog format
- All features, fixes, breaking changes
- Proper attribution

### 4. Release Notes
- File: `/docs/v6_RELEASE_NOTES.md`
- Marketing-friendly announcement
- Key highlights
- Download/upgrade instructions

## Quality Standards
- ✅ All docs are complete and self-contained
- ✅ Examples are accurate and tested
- ✅ Professional grammar and formatting
- ✅ Clear for end users (not just developers)
- ✅ Links between docs work
- ✅ Ready to publish on website

## Success Criteria
- ✅ Breaking changes guide complete
- ✅ Migration guide step-by-step and clear
- ✅ CHANGELOG entries for all v6.0.0 items
- ✅ Release notes professionally written
- ✅ Memory key release_documentation populated
- ✅ All docs ready for publication on GitHub/website

## Dependencies
- **Awaits**: backward_compatibility_analysis (needs migration guide)
- **Awaits**: v6_specification (needs feature descriptions)
- **Provides**: Complete documentation for Release Manager
- **Provides**: Content ready to publish

## Notes
- This documentation is what users see first
- Quality and clarity directly impact adoption
- Include examples - they help users understand
- Be empathetic to users upgrading from v5.5.0
- Test all examples before publishing

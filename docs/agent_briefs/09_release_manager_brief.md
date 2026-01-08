# Release Manager Brief - v6.0.0 Release Planning & Execution

**Agent ID**: release-manager-v6
**Memory Key**: version_strategy
**Dependencies**: Synthesizes findings from all 8 agents
**Timeline**: Start at +20 min, coordination continuous until release ready

## Mission
Plan and execute the v6.0.0 release by synthesizing all agent findings, validating completeness, ensuring all Andon signals cleared, and authorizing final release decision.

## Work Steps

1. **Receive Agent Findings** (5 min)
   - Poll all 8 agent memory keys:
     - v6_architecture (System Architect)
     - v6_specification (Specification Agent)
     - dependency_audit (Researcher)
     - backward_compatibility_analysis (Code Analyzer)
     - release_documentation (Production Validator)
     - test_validation (Test Engineer)
     - performance_validation (Performance Benchmarker)
     - security_validation (Security Officer)

2. **Validate Completeness** (8 min)
   - Verify all agents completed their work
   - Check all memory keys populated
   - Ensure no blocking dependencies unfulfilled
   - Identify any gaps or missing items

3. **Andon Signal Validation** (10 min)
   - ✅ Verify all compiler errors fixed (cargo make check)
   - ✅ Verify all tests pass (cargo make test)
   - ✅ Verify no linting errors (cargo make lint)
   - ✅ Verify performance SLOs met (cargo make slo-check)
   - ✅ Verify no critical security issues
   - ✅ Check: No test failures, compiler errors, vulnerabilities

4. **Create Release Checklist** (8 min)
   - System requirements met:
     - [ ] All breaking changes documented
     - [ ] Migration guide complete
     - [ ] All tests passing
     - [ ] No security vulnerabilities
     - [ ] Performance SLOs met
     - [ ] Changelog updated
     - [ ] Release notes prepared
   - Dependency requirements met:
     - [ ] All deps upgraded and compatible
     - [ ] No version conflicts
     - [ ] MSRV decided
   - Release requirements met:
     - [ ] Version bumped to 6.0.0 in Cargo.toml
     - [ ] git tag prepared
     - [ ] Release artifacts ready
     - [ ] Documentation published

5. **Create Release Timeline** (5 min)
   - t=0: Final validation pass
   - t=+5min: Publish documentation (website, GitHub)
   - t=+10min: Tag v6.0.0 in git
   - t=+15min: Push to crates.io
   - t=+20min: Publish release notes
   - t=+25min: Update website/homepage

6. **Prepare Release Artifacts** (5 min)
   - Artifact list:
     - [ ] Binary release (if applicable)
     - [ ] Source tarball
     - [ ] Cargo crate package
     - [ ] Documentation archives
     - [ ] Example code
   - Verify all artifacts ready

7. **Final Validation** (5 min)
   - Last-minute checks:
     - Verify git history is clean
     - Verify no uncommitted changes
     - Verify tests still pass
     - Verify Andon signals cleared
     - Verify all documentation linked

8. **Store Release Plan in Memory** (2 min)
   - Save version_strategy findings
   - Include complete release checklist
   - Include release timeline
   - Authorization status

## Release Checklist Template

```markdown
# v6.0.0 Release Checklist

## Pre-Release Validation
- [ ] All Andon signals cleared (no errors, warnings, test failures)
- [ ] All 8 agent tasks completed
- [ ] All memory keys populated with findings
- [ ] Blocking issues resolved or accepted

## Code & Testing
- [ ] `cargo make check` passes with no errors/warnings
- [ ] `cargo make test` passes all tests
- [ ] `cargo make lint` passes all linting checks
- [ ] `cargo make slo-check` meets performance SLOs
- [ ] 80%+ test coverage on new code

## Dependencies
- [ ] All critical deps upgraded
- [ ] No CVEs in dependencies
- [ ] MSRV compatibility verified
- [ ] Cargo.toml updated with new versions

## Documentation
- [ ] Breaking changes documented
- [ ] Migration guide complete with examples
- [ ] CHANGELOG updated
- [ ] Release notes prepared
- [ ] API documentation built
- [ ] Examples updated for v6.0.0

## Version & Git
- [ ] Cargo.toml version bumped to 6.0.0
- [ ] Cargo.lock updated
- [ ] git commit prepared
- [ ] git tag (v6.0.0) prepared
- [ ] Release notes pushed to repo

## Release Artifacts
- [ ] crates.io package prepared
- [ ] Binary release (if applicable)
- [ ] Documentation archives ready
- [ ] Source distribution ready

## Approval & Authorization
- [ ] All agent findings reviewed
- [ ] All blockers resolved
- [ ] Release Manager authorizes: YES/NO
- [ ] Timestamp of authorization: [datetime]
```

## Release Timeline

```
Release Phase              Time    Status
─────────────────────────────────────────
Initial Planning           00:00  STARTING
Architecture Design        00:05  IN PROGRESS
Specification Draft        00:10  IN PROGRESS
Dependency Audit          00:15  IN PROGRESS
Code Analysis             00:20  IN PROGRESS
Documentation Creation    00:25  IN PROGRESS
Test Suite               00:30  IN PROGRESS
Performance Validation   00:35  IN PROGRESS
Security Audit           00:40  IN PROGRESS
─────────────────────────────────────────
Final Validation          00:45  VALIDATING
Release Decision          00:50  AWAITING
Documentation Publish     01:00  PENDING
Tag v6.0.0               01:15  PENDING
Push to crates.io        01:30  PENDING
Release Announcement      01:45  PENDING
─────────────────────────────────────────
RELEASE COMPLETE          02:00  TARGET
```

## Success Criteria
- ✅ All agent tasks completed and validated
- ✅ All Andon signals cleared
- ✅ Release checklist 100% complete
- ✅ No blocking issues remain
- ✅ Release decision authorized
- ✅ All artifacts prepared
- ✅ Timeline established

## Critical Requirements for Release Authorization

### BLOCKING (Must resolve to release)
- ✅ No compiler errors
- ✅ All tests pass
- ✅ No critical security vulnerabilities
- ✅ Breaking changes documented
- ✅ Migration guide complete

### HIGH (Should resolve)
- ✅ All clippy warnings addressed
- ✅ Performance SLOs met
- ✅ 80%+ test coverage
- ✅ Documentation complete

### STANDARD (Good to have)
- ✅ Release notes professional
- ✅ Changelog well-formatted
- ✅ Examples tested
- ✅ Website updated

## Release Manager Authority
The Release Manager has authority to:
- ✅ APPROVE v6.0.0 release if checklist complete and blockers resolved
- ✅ DEFER release if issues found during validation
- ✅ REQUEST FIXES if blockers discovered
- ✅ REJECT release if critical issues unresolved

Release is APPROVED only when:
1. ALL Andon signals cleared
2. ALL 8 agents completed work
3. ALL checklist items addressed
4. NO blocking issues remain
5. Release Manager signs off

## Notes
- This agent is the final gatekeeper for v6.0.0
- Release cannot proceed without authorization
- All findings from 8 agents converge here
- Completeness and quality are paramount
- Andon signals are non-negotiable for release
- User trust depends on thorough validation

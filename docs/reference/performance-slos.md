# Reference: Performance SLOs

**Version**: 26.9.1

Service Level Objectives for the `clap-noun-verb` framework. These cover the **CLI
framework** itself — build cost, binary size, dispatch overhead — not any particular
downstream application's domain logic.

---

## Build & Size SLOs

| Metric | Target | Measured | Source |
|--------|--------|----------|--------|
| Incremental compilation | ≤ 2 s | 0.66 s | project `CLAUDE.md` |
| Release binary size | ≤ 10 MB | 2.2 MB | project `CLAUDE.md` |
| Full test suite (parallel) | < 1 s | ~0.16 s | `cargo make test-lib` |

The minimalist default build (zero features) is what these targets assume. Enabling
optional features (`repl` pulls in `rustyline`, etc.) increases binary size accordingly.

---

## Runtime Dispatch Characteristics

The framework is designed for zero measurable overhead versus hand-written `clap`:

| Operation | Cost | Notes |
|-----------|------|-------|
| Command registration | Compile-time only | Verbs are collected via `linkme` distributed slices; no runtime registration cost |
| Command routing | O(1) | Hashmap lookup from noun/verb → handler |
| Argument parsing | Same as raw `clap` | No wrapper overhead |
| JSON serialization | O(output size) | `serde_json` over the handler's `Serialize` result |
| Telemetry span (no subscriber) | Negligible | Telemetry is always compiled; emission is gated on `RUST_LOG` |

These are qualitative guarantees from the architecture (see
[API Catalog → Feature Flags / Performance characteristics](api-catalog.md)). They are not
fixed millisecond budgets — actual numbers depend on the host and the command tree size.

---

## Measuring Locally

The crate ships a Criterion benchmark suite. To reproduce numbers on your machine:

```bash
cargo make bench-build                # build the criterion benchmark suite
cargo bench                           # run it (criterion writes target/criterion/ reports)
```

The only benchmark source today is `benches/dispatch.rs` (`bench_build_command`,
`bench_route`) -- there is no separate startup/middleware-chain/telemetry-overhead
suite yet. Treat the SLOs above as regression gates: a change that pushes incremental
compile past 2 s, the binary past 10 MB, or the test suite past 1 s is a defect.

---

## See Also

- [API Catalog](api-catalog.md) — performance characteristics per construct
- [Performance Optimization](../howto/performance-optimization.md) — application-level tuning
- [Error Codes](error-codes.md) — `DeadlineExceeded` / `TimeoutAdjustment` for deadline SLOs

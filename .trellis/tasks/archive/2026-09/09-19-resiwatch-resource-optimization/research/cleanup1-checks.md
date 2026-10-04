# Cleanup1 bounded review and full gate — 2026-09-19

This record covers the frozen query1 plus cleanup1 source. Earlier full gates remain evidence for their own source states, not this follow-up.

## Review scope and findings

Hash comparison against `performance-20260919-layout3-query1/candidate-source-manifest.json` found exactly three changed Rust files: `storage_lifecycle.rs`, `c3/query.rs`, and `bench/corpus.rs`. All other files in that source manifest were unchanged. No new blocking finding was identified, and the checker made no product changes.

- Auxiliary session and receipt cleanup read at most 128 candidates per page and process at most 1,000 candidates per call. Cached statements avoid repeated preparation. The persisted cursor advances only through candidates actually processed; an early yield does not skip the rest of a fetched page. End-of-scan wraps to zero, permitting later reconsideration of protected or previously ineligible rows.
- Cooperative session/receipt yields occur at 100/200 ms within the existing 250 ms hard interrupt budget. Receipt work starts only while the shared elapsed time is below 150 ms. Each processed prefix, its deletes, and the associated cursor or receipt expiry digest commit in the same transaction. Hard interruption retains the existing clear-handler-then-explicit-rollback path. The 24-hour/latest-100,000 receipt union and pending/session protections are unchanged.
- The deterministic session fan-out regression adds 3 ms per deletion, verifies durable partial progress before reopen, and then verifies completion. Receipt-prefix/digest and protected-session cursor regressions cover early yields. Existing cancellation regressions still require unchanged rows/cursors/digests and an idle writer after interruption.
- The raw plan now echoes `totals_raw_attribution`, resolving the prior query1 diagnostic finding. Its regression checks the named constant resolves to the fused SQL and retains the comparison query name.
- The ignored, `cfg(test)` corpus capacity gate now records before/after combined byte totals and remaining expired raw in its evidence file before asserting completion. An incomplete run still fails; recording conservation does not claim completion. No production corpus or deletion gate behavior changes.

The preceding query1 review also checked the fused raw totals/attribution partition, unchanged comparison and period query semantics, and the shared query progress handler's 1,024-opcode stride. This full gate includes those sources and regressions as well as cleanup1.

## Full gate

```powershell
rtk proxy just ci
```

- Started: `2026-09-19T12:43:43.6644452Z`.
- Log: `%TEMP%/resiwatch-cleanup1-integration-check-20260919.log`.
- Result: **PASS**, exit code `0`; elapsed `77.5406845 s`.

| Check | Observed result |
| --- | --- |
| Monitor version alignment | Pass, `0.3.0` |
| Frontend icons, TypeScript, ESLint and Vite build | Pass; build `2.93 s` |
| Frontend tests | 73 files; 300 passed; `4.66 s` |
| Rust formatting and strict workspace/all-target clippy | Pass; clippy `5.31 s` |
| Rust library tests | 527 passed, 0 failed, 2 ignored; `12.03 s` |
| Rust process crash/retry integration tests | 3 passed; `0.42 s` |
| Rust binaries and documentation tests | Pass; zero tests |
| Root syntax checks and tests | Pass; 139 passed; `1.8109823 s` |
| Monitor and root template secret scans | Pass |

The full run includes the query1 fusion equivalence and active-SQL cancellation tests, the named-SQL regression, all three new cooperative-yield tests, and both existing cancellation rollback regressions. No integration fix was needed. The locked dependency refresh again reported one existing high-severity advisory; dependency definitions and lockfiles were unchanged.

The gate covers version alignment, locked dependency refresh, frontend icons/typecheck/lint/tests/build, Rust formatting and strict workspace/all-target clippy, Rust workspace tests, root syntax/tests, and template secret scans. No capacity tests or performance workloads run concurrently.

## Boundaries

The checker changed only this evidence file. No installed application, production database, capacity fixture, credentials, dependency definitions, commit, push, or memory note was changed. The ignored full-capacity fixture and actual Windows Credential Manager write tests were not run here. Automatic deletion remains disabled. Full-scale retention completion, performance and installed-soak acceptance require separate evidence.

The full gate process exited successfully. Product source remains frozen; Cargo ownership was returned to the main agent immediately after the exit for its final release build and isolated retention measurements.

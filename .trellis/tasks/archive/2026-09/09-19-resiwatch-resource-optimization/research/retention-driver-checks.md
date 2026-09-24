# Retention driver review and full gate — 2026-09-19

## Scope and findings

Hash comparison against `performance-20260919-cleanup1/candidate-source-manifest.json` found only `src-tauri/src/bench/corpus.rs` changed. The production storage, retention, report and schema sources in that manifest were unchanged. The checker found no blocking issue and made no product or helper edits.

- Retry classification uses the existing production `ReportError` mapping: only `DeadlineExceeded` and `StorageBusy` are eligible, and only if the writer is already in autocommit and the cancellation flag is false. Cancellation, low space, integrity/I/O failures and other terminal errors stop the maintenance loop. A writer with an open transaction is never retried.
- Each retry consumes the original `max_chunks` budget and records the error, classification, writer state and elapsed time. An unsuccessful final attempt cannot leave `complete=true`. This accelerated fixture driver explicitly does not emulate the production scheduler's 60-second error interval.
- Successful auxiliary cleanup and retryable auxiliary-cleanup errors both observe persisted session/receipt cursors. This preserves a session scan completion when its transaction committed before a receipt step timed out. A failed retention step does not itself run auxiliary cleanup.
- Independent end-of-scan observations remain latched while raw/coverage/dictionary work is pending. The latches are consumed only when that work reports no pending chunks, then reset before an independent full lifecycle inventory. Nonzero reclaimable sessions or eligible receipts prevent completion and require another scan cycle. A stale or initially zero cursor can request an inventory check, but cannot alone produce a passing capacity result.
- The isolated ignored capacity test still requires completion, byte conservation, zero remaining expired raw, zero reclaimable sessions/eligible receipts and `quick_check=ok`. It preserves evidence before assertions. The production automatic-deletion flag remains false; the test-only gate remains isolated.

Focused source regressions cover retryable versus terminal SQLite mappings, cancellation after classification, open-writer refusal, asynchronous cursor completion while other work remains, latch consumption/reset, and failed cursor reads. The full gate below executes the ordinary helper regressions but not the ignored capacity workload.

## Full gate

```powershell
rtk proxy just ci
```

- Started: `2026-09-19T13:23:00.2182053Z`.
- Log: `%TEMP%/resiwatch-retention-driver-integration-check-20260919.log`.
- Result: **PASS**, exit code `0`; elapsed `81.269926 s`.

| Check | Observed result |
| --- | --- |
| Monitor version alignment | Pass, `0.3.0` |
| Frontend icons, TypeScript, ESLint and Vite build | Pass; build `3.16 s` |
| Frontend tests | 73 files; 300 passed; `4.27 s` |
| Rust formatting and strict workspace/all-target clippy | Pass |
| Rust library tests | 529 passed, 0 failed, 2 ignored; `13.48 s` |
| Rust process crash/retry integration tests | 3 passed; `0.40 s` |
| Rust binaries and documentation tests | Pass; zero tests |
| Root syntax checks and tests | Pass; 139 passed; `2.2450368 s` |
| Monitor and root template secret scans | Pass |

Retry classification, durable scan-end observation and asynchronous completion-latch regressions passed in the workspace run. No integration fix was needed. The locked dependency refresh again reported one existing high-severity advisory; dependency definitions and lockfiles were unchanged.

The gate includes version alignment, locked dependency refresh, frontend icons/typecheck/lint/tests/build, Rust formatting and strict workspace/all-target clippy, Rust workspace tests, root syntax/tests, and secret scans. No performance or full-capacity workload runs concurrently.

## Boundaries

No installed application, production database, capacity fixture, credentials, dependency definitions, commit, push or memory note was changed. This check does not replace remaining full-capacity, performance, memory or installed-soak evidence. Earlier measurement results remain tied to their recorded executable hashes.

The gate process exited successfully and all source remained frozen. Cargo and the heavy-work slot were returned to the main agent immediately after exit for the final release build. The checker changed only this evidence file.

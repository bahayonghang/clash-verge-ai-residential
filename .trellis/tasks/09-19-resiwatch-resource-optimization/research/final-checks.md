# Integration check — 2026-09-19

Scope: the approved resource-optimization implementation in the shared `dev` checkout. The checker owned only required in-scope product fixes and this evidence file. The main agent owns task/specification updates and subsequent performance measurements. The unrelated `.gitignore` change was preserved.

Result: full `just ci` passed, followed by passing formatting, strict all-target clippy and all normal benchmark tests for the final harness-only edit. Product source is frozen and no checker-owned Cargo process remains.

## Gate runs

| Command | Outcome | Wall time / evidence |
| --- | --- | --- |
| `rtk proxy just ci` (sandbox) | Stopped at `npm ci`: Windows `spawn EPERM`, before source checks | 17.8762459 s; `%TEMP%/resiwatch-integration-check-20260919.log` |
| `rtk proxy just ci` (approved process permission) | Frontend, Rust fmt/clippy passed; Rust library tests: 508 passed, 4 failed, 2 ignored | 70.2478422 s overall; library tests 12.49 s; `%TEMP%/resiwatch-integration-check-20260919-elevated.log` |
| `rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib cancellation_during_ -- --nocapture` | Both auxiliary cleanup cancellation regressions passed after rollback correction | 2 passed, 512 filtered out; 0.39 s test execution |
| `rtk cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib -- --nocapture` | All library tests passed after all corrections and the added raw-deletion regression | 513 passed, 2 ignored; 11.96 s test execution |
| `rtk proxy just ci` (first post-fix rerun) | All monitor checks passed; one unchanged root test hit a temporary-file rename `EPERM` | 83.2081973 s overall; library 513 passed/2 ignored in 11.71 s, process integration 3 passed; `%TEMP%/resiwatch-integration-check-20260919-final.log` |
| `rtk proxy node --test --test-name-pattern='home_proxy 凭据缺键不自动补全' tests/sync-local-config.test.js` followed by `rtk proxy npm run ci` | The failed root test and complete root gate passed without any source edit | Focused 1/1; root 139/139, 1.6568388 s root test execution |
| `rtk proxy just ci` (confirmation) | PASS: frontend 73 files/300 tests, Rust library 513 passed/2 ignored, process integration 3 passed, root 139 passed; fmt, clippy, builds and secret scans passed | 61.2856637 s overall; library tests 12.13 s; process tests 0.95 s; `%TEMP%/resiwatch-integration-check-20260919-confirmed.log` |
| `rtk proxy cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check`; `rtk proxy cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`; `rtk proxy cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib bench::` | PASS on the final frozen corpus harness edit: 10 benchmark tests passed, 1 intentionally ignored, 505 filtered out | 32.6974398 s overall; test execution 3.56 s; `%TEMP%/resiwatch-corpus-final-check-20260919.log` |

The corpus owner froze its final `corpus.rs` optimization after the confirmed full gate had already compiled Rust. Therefore that edit is covered by the explicit final fmt/clippy/benchmark follow-up, not represented as part of the earlier full-gate snapshot. It adds one normal regression, bringing the current library inventory to 516 tests (514 normal and 2 intentionally ignored); the new normal test was executed by the follow-up.

## Findings fixed

1. `storage_lifecycle.rs`: persistent progress-handler cancellation could also interrupt the transaction destructor's `ROLLBACK`, leaving uncommitted auxiliary deletion/boundary changes visible on the shared writer. After clearing the handler, failed maintenance now explicitly rolls back a still-open owned transaction. A non-idle writer is rejected before starting maintenance. Both SQL-trigger cancellation tests now verify restoration; receipt cleanup also asserts autocommit.
2. `c3/retention_staged.rs` and `c3/retention.rs`: the corresponding staged-retention and chain-repair error tails now finish an interrupted owned rollback after clearing the handler. They preserve caller-owned transactions by recording initial autocommit. `retention_day.rs` asserts an idle writer after cancellation and adds a real raw-delete trigger regression proving all four raw rows survive cancellation and subsequent retry deletes them successfully. Archive purge uses one implicit-transaction statement and has no matching RAII rollback tail.
3. `c3/archive.rs`: rebuilding candidates after a backward clock jump preserved the old future `retry_after` for jobs never attempted, delaying ready work. Only actual failures now carry retry backoff across refresh; unattempted work is ready at the current timestamp. The existing clock-rewind/timezone and failure-backoff regressions pass.
4. `c2/facade.rs`: the new zero-traffic ID-reuse test incorrectly expected the reused connection's cumulative seven bytes to be counted on the first frame of its new generation. It now checks zero baseline traffic followed by the next real five-byte delta, while retaining assertions that durable identity reuse creates a new generation/session.

The main agent synchronized the explicit-owned-rollback contract into `storage/sqlite-contract.md`. No unresolved source findings remain within this integration-check scope.

## Evidence boundaries

The final full gate includes version alignment, the locked frontend dependency refresh, icon check, TypeScript, ESLint, Vitest, Vite build, Rust fmt/clippy/workspace tests, root syntax/tests and secret scans. Docs-site build is independent and was not run by this checker. Package Markdown documentation does not alter the VitePress site.

The normal Rust run intentionally excludes `bench::corpus::tests::isolated_corpus_retention_capacity_gate` and the real Windows Credential Manager CRUD test. This checker did not run timed A/B, capacity, peak or installed-soak measurements, install an application, modify the production ledger/credentials, add dependencies, commit, push, archive, or write a duplicate memory checkpoint. `AUTO_DELETE_ENABLED` stays false. Passing automated checks does not satisfy the remaining performance/capacity/installed acceptance criteria.

`npm ci` reported one high-severity dependency advisory. No manifest/lockfile changes or automatic dependency fixes were made; the warning is separate from the source gate.

The unchanged root rename failure occurred once and passed immediately both alone and in two subsequent complete root runs. Its underlying Windows lock source was not established; it is recorded as a transient environment observation, not a fixed product defect.

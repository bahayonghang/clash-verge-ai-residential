# Retention statement-cache review and full gate — 2026-09-19

## Scope and findings

Hash comparison against `performance-20260919-final-local/candidate-source-manifest.json` found exactly `src-tauri/src/c3/retention.rs` and `src-tauri/src/c3/retention_staged.rs` changed. Other manifest entries, including schema, storage lifecycle, query code and the capacity driver, were unchanged. No blocking finding was identified, and the checker made no product changes.

The bounded review covered seven conversions to cached preparation: the lookup, next-ID lookup and insert in `intern_one`; the per-row dimension-ID lookup in `raw_rows`; the core and dimension writes in `write_actual`; and the lookup in `read_actual`.

- SQL remains parameterized. The dictionary operations bind the same kind/value/ID fields; core writes bind six values and dimension writes bind eight. Readback uses the same core or dimension key and returns the same four exact aggregate values.
- Cached statements are local to the same connection and are dropped/reset before the surrounding chunk completes. No statement, rows iterator or transaction is stored beyond the existing function/chunk lifetime. Table names still come from the existing fixed `source` mapping.
- The dictionary existence check, empty/unknown identity exclusion, optional missing-row handling, post-write equality check, publication/audit comparisons and stage membership checks remain active. Cached preparation does not bypass SQLite triggers or the actual readback query.
- Transaction ownership, chunk size, raw retention boundaries, deletion gate, progress handler and rollback tail are unchanged. Preparation and execution failures still propagate through their existing error mappings into the enclosing chunk failure path.

The owner-reported focused retention test pass is not treated as a substitute for the independently run full gate below. No extra synthetic cache test was added for this API-level preparation change; the existing correctness, corruption, cancellation and restart regressions exercise the affected paths.

## Full gate

```powershell
rtk proxy just ci
```

- Initial run started: `2026-09-19T13:45:17.4186742Z`.
- Initial log: `%TEMP%/resiwatch-retention-cache-integration-check-20260919.log`.
- Initial result: exit `1` after `64.1855449 s`. The monitor gate passed (frontend 300; Rust 529 plus 3 process tests, two expected ignored). Root tests had 138 passes and one failure: Windows rejected a temporary-fixture rename with `EPERM` in `tests/sync-local-config.test.js:527`, through `scripts/sync-local-config.js:503`. No retention code appeared in the failure path.
- Focused rerun: `rtk proxy node --test --test-name-pattern='本地 openai_core = false' tests/sync-local-config.test.js` passed, one test, `181.4147 ms` test duration. The rename failure did not reproduce, and no source was modified.
- Full rerun started: `2026-09-19T13:47:17.8698076Z`; log `%TEMP%/resiwatch-retention-cache-integration-recheck-20260919.log`.
- Full rerun result: exit `1` after `47.2374588 s`. All monitor checks passed again. Root tests had 137 passes and two failures in different temporary fixtures (`tests/sync-local-config.test.js:94` and `:913`), both Windows `EPERM` during the same atomic rename path. This did not justify modifying unrelated root source.

The first alternate-temp diagnostic used the ignored `residential-monitor/src-tauri/target/integration-temp-retention-cache` directory. It exited `1` in `3.3860066 s`, but this was an invalid diagnostic setup: generated `.js` fixtures inherited the frontend package's `type: module` scope, and their expected CommonJS `constants` exports were undefined. Those errors are neither retention failures nor evidence that the rename error persisted. No source or global settings were changed; its tool output was not separately logged.

The corrected isolated-temp commands ran from the repository root, preserving its CommonJS scope:

```powershell
$checkTemp = Join-Path (Get-Location).Path 'bench-data/integration-temp-retention-cache'
New-Item -ItemType Directory -Path $checkTemp -Force | Out-Null
$env:TEMP = $checkTemp
$env:TMP = $checkTemp
rtk proxy npm run ci
# After that succeeds, in a command with the same local TEMP/TMP:
rtk proxy just ci
```

- Corrected root-only diagnostic: exit `0`, all 139 tests plus syntax and secret checks passed; `3.7740039 s` command time and `1478.691 ms` test time. Log: `C:/Users/lyh/AppData/Local/Temp/resiwatch-retention-cache-root-isolated-20260919.log`.
- Final full gate started `2026-09-19T13:52:49.6752022Z`: exit `0` after `51.4945317 s`. Log: `C:/Users/lyh/AppData/Local/Temp/resiwatch-retention-cache-full-isolated-20260919.log`.
- Frontend: icons, TypeScript and ESLint passed; 73 test files / 300 tests passed in `3.90 s`; production Vite build passed in `1.61 s`.
- Rust: formatting and strict clippy passed (`0.90 s` clippy); 529 library tests passed, two expected tests ignored, in `11.62 s`; all three process crash/retry tests passed in `0.37 s`; binary and doc-test targets passed with zero tests.
- Root: syntax check, all 139 tests (`1469.7028 ms`) and both secret-scan invocations passed. Docs build was not part of this gate.

The passing gate is conditional on that command-local temporary directory. It supports an environment-dependent fixture failure but does not establish the root cause of default-temp `EPERM` (contention remains a hypothesis). No retry or other workaround was added to product or test source. The existing dependency audit output reported one high-severity advisory; dependency changes were outside this review and none were made.

The gate includes version alignment, locked dependency refresh, frontend icons/typecheck/lint/tests/build, Rust formatting and strict workspace/all-target clippy, Rust workspace tests, root syntax/tests, and secret scans. No capacity workload, release build or timing experiment runs concurrently.

## Evidence boundaries

The interrupted A1000 first-day capacity run and all earlier measurements remain evidence for their original source and executable identities. This gate cannot establish a retention speedup or completed capacity run. The main agent owns the subsequent frozen release binaries and measurements.

No installed application, production database, retained capacity fixture, credentials, dependency definitions, commit, push or memory note was changed. Automatic deletion remains disabled, and the ignored capacity and Windows Credential Manager tests are excluded from this ordinary full gate.

Product source remained frozen throughout this review. The completed gate process exited, and the checker handed the Cargo/heavy-work slot back to the main agent before finalizing this evidence file.

# Auxiliary cleanup and raw-series review — 2026-09-19

## Scope and result

Independent source review found no blocking defect in the declared cleanup-completion and per-bucket raw-series changes. The initial review performed no product edits or workloads. After both source owners froze their changes and the main agent handed over the exclusive Cargo/heavy-work slot, the checker ran the full product gate below. No integration fixes were necessary; the main agent retains ownership of release builds, measurements and acceptance evidence.

Comparison against `performance-20260919-retention-cache/candidate-source-manifest.json` found eight changed entries: `bench/corpus.rs`, `c2/facade.rs`, `c3/retention.rs`, `c3/retention_day.rs`, `c3/retention_staged.rs`, `c3/service.rs`, `c3/sql.rs` and `storage_lifecycle.rs` (all under `residential-monitor/src-tauri/src/`). Schema and dependency files in that manifest remain unchanged. The manifest provides old hashes, not old file contents; review therefore combined this scope check, current code, HEAD diff and the already reviewed retention-cache behavior. It is not presented as a byte-for-byte patch against an unavailable source snapshot.

Reviewed service SHA256: `F7EF789DC3527DFBA2A42D4925ACBEB23BEEA7FAC42C3D7DC6459DD858D4A32E`.
Reviewed SQL SHA256: `6FA426D2DEDB18C58634FB153A06D3EA41AAAC2523E6DFD563DF999D5C9560E5`.

## Auxiliary completion

- `c3/retention_staged.rs:157` and `:192`: final clear/discard and dimension-prune completion mark the existing dictionary/coverage cursors dirty (`-1`) before committing. Reference release and invalidation share one transaction. Failed transactions cannot durably advertise completion while releasing references.
- `storage_lifecycle.rs:271`: successful session deletion marks the dictionary cursor dirty in the same transaction as chain/attribute/session deletion and the processed-session cursor. No marker is written when no session was deleted. Active/pending/raw-reference protections and the existing cancellation cleanup remain intact.
- `c3/retention_day.rs:172`: auxiliary pending checks only the two existing cursor keys. `-1` starts a fresh sweep; a positive cursor continues an ordered page; `0` denotes the last completed sweep. The existing 128-source-row pages advance even when every row remains protected.
- `c3/retention_staged.rs:206`, `c2/facade.rs:2240` and `bench/corpus.rs:349`: chunk completion includes auxiliary pending, then callers check again after ledger cleanup can release more dictionary references. The driver still requires its completed session/receipt scans and full lifecycle inventory before reporting completion.
- Production `AUTO_DELETE_ENABLED` remains false (`c3/query.rs:21`). Gate-off/materialize-only paths do not spin on dirty deletion cursors. The single writer, source page limits, 1-second retention deadline, 250-ms ledger deadline, cooperative yields and explicit rollback after handler removal are unchanged.

The two new source regressions exercise final multi-day pruning with more than one dictionary page, and reference release after an already completed sweep with protected session/category references, a closed deletion gate and eventual convergence. Existing cancellation regressions continue to cover rollback and reusable/autocommit writer state. The main agent reported that the storage cursor contract was synchronized.

## Per-bucket raw series

- `c3/service.rs:1378`: one prepared statement is rebound for disjoint clipped minute ranges. Every granularity has a positive width. Rust integer division truncates toward zero like the old SQLite expression: negative buckets end at `bucket + 1`, and the zero bucket spans negative and positive remainders. The final upper boundary is clipped to `end_min`, and each iteration advances. The quotient-times-width operation cannot increase the absolute cursor value; the positive upper addition saturates. The conversion of returned bucket minutes to seconds retains the old `* 60` behavior and does not introduce a new extreme-timestamp policy.
- `c3/sql.rs:87`: joins and filter predicates match the previous grouped query; values remain bound. `HAVING count(*) > 0` omits empty buckets while preserving observed zero-upload/download buckets. Each bucket independently counts distinct session and minute keys, maintaining connection-count and active-duration semantics without summing per-row counts.
- `c3/service.rs:159` and `:194`: the report installs one progress handler with its original start time and complete-query limit before opening its read transaction. `load_raw_series` never installs, clears or restarts that handler. Cancellation/deadline classification and reader cleanup remain at the existing outer boundary; there is no new budget per bucket.
- `c3/query.rs:991` and `c3/sql.rs:418`: `series_raw` still names the actual production template. `explain_named` derives its parameter count dynamically, so the changed placeholder count is reflected. Other series loaders, comparison totals and period usage retain their existing routes. The repository search found no remaining caller using the old parameter layout with the new template.
- `c3/sql.rs:72` preserves the old full-window grouped SQL under `cfg(test)`. The equivalence regression (`service.rs:3151`) compares five result fields using all seven granularities, six ranges and eleven filter selections, including negative boundaries, missing metadata, residential fallback, multiple filters, empty ranges and zero-byte observations. The oracle partitions rows in SQLite rather than repeating the Rust bucket loop. Separate tests check the minute-range access plan and shared cancellation/deadline across many buckets.

## Verification status and limits

The implementation owner reported that the three focused release series tests passed in `1.97 s`, and that a paired A1000 retained-29-day series probe returned 696 rows with an exact result-digest match. Those are owner-reported results, not independently rerun by this reviewer. The reported series-stage reduction does not establish the production 10-second full-report deadline: the owner also reported slower totals/rank stages remain unresolved.

Earlier retention-cache gate results and capacity timings remain attached to their original source/executable hashes. No automatic-deletion approval or full-task completion follows from this review.

## Independent full product gate

The main agent authorized the gate after source freeze. No extra targeted test was added: the existing cross-bucket regression exercises reuse of the statement and the registered handler; the implementation neither installs a fresh handler nor grants a new deadline when moving across sparse buckets. The loop retains the existing maximum-range and whole-query budget contracts.

Executed from the repository root with temporary variables scoped to that command process:

```powershell
$checkTemp = Join-Path (Get-Location).Path 'bench-data/integration-temp-aux-series'
New-Item -ItemType Directory -Path $checkTemp -Force | Out-Null
$env:TEMP = $checkTemp
$env:TMP = $checkTemp
rtk proxy just ci
```

The isolated temporary directory preserves the root CommonJS package scope and avoids the default-temp fixture rename failures observed in the preceding gate. No global environment or test/product source was changed.

- Started: `2026-09-19T15:59:46.8407225Z`; completed after local midnight on September 20. Exit `0`; elapsed `77.6075539 s`.
- Complete command output: `C:/Users/lyh/AppData/Local/Temp/resiwatch-auxiliary-series-full-isolated-20260919.log`.
- Frontend: icon check, TypeScript and ESLint passed; 73 files / 300 tests passed in `3.69 s`; Vite production build passed in `2.27 s`.
- Rust: formatting passed; strict workspace/all-target clippy passed in `5.55 s`; test build `16.24 s`; 534 library tests passed in `10.54 s`, including the two cleanup and three series regressions. Three tests were intentionally ignored: isolated capacity, native Credential Manager and the SQL stage probe. All three process crash/retry tests passed in `0.36 s`; binary/doc-test targets passed with zero tests.
- Root: syntax checks, 139 tests (`1610.1935 ms`) and both secret-scan invocations passed. The dependency audit reported the existing one high-severity advisory; no dependency definitions changed. Docs build is outside `just ci` and was not run by this checker.

No capacity workload, stage measurement, release build or other Cargo command ran concurrently with this gate. No installed application, production database, preserved capacity fixture, credentials, dependency definition, commit, archive or memory note was changed. Product source remains frozen. After the gate process exited, the Cargo/heavy-work slot was handed back to the main agent before this evidence update.

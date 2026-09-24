# Layout2 source review and integration gate — 2026-09-19

This evidence is separate from `final-checks.md`, which records the first implementation and its benchmark-harness follow-up. Those earlier passing checks do not validate layout2.

## Source review

The review first ran read-only while the copied first-candidate executables were being measured. It executed no Cargo, SQL, tests or measurements and made no edits during that interval.

Hash comparison against `performance-20260919/candidate-source-manifest.json` found exactly six changed Rust files: `storage.rs`, `storage_lifecycle.rs`, `c3/schema.rs`, `c3/service.rs`, `dbcli/mod.rs`, and `bench/corpus.rs`. Cargo manifests, lockfile, build script and all other Rust files in that manifest were unchanged.

| Concern | Source evidence and conclusion |
| --- | --- |
| Receipt migration and rollback | `c3/schema.rs` rebuilds the receipt table with `data_version INTEGER PRIMARY KEY` and unique `(writer_epoch, bundle_seq)`, copying existing versions, identities and hashes. Unknown legacy commit timestamps remain NULL. In `storage::migrate`, copy/drop/rename, contiguous-prefix reconstruction, checksum and user-version publication share the v5 transaction. Duplicate legacy versions fail without renumbering receipts. |
| Durable version ownership | `storage::durable_data_version` reads the maximum of the legacy singleton floor and latest receipt. Commit allocation is inside the existing immediate writer transaction, checks integer exhaustion, and no longer rewrites the singleton per commit. Retry lookup happens before allocation and returns the original version/hash outcome. Writer reservation, storage health, report, internal period usage and CLI readers use the same helper. |
| Receipt expiry | Pruning retains the latest 100,000 receipts as well as the recent time window, so it cannot remove the global maximum receipt version. Per-epoch expiration boundaries/digests remain atomic with deletion. Unknown-time legacy receipts still require owner retirement plus the observation window. |
| Coverage tail | The explicit `(kind, reason, interval_id)` index selects one latest controller-sample interval. The update changes only `ended_utc`, extends only a contiguous same-UTC-day interval, and otherwise appends a new interval. Clock rollback does not scan or overwrite older matching intervals. Cross-day spans remain split. Frozen-range checks precede persistence. |
| Reader and fixture consistency | Public reports and internal period usage read the version within their business read transaction. The corpus generator leaves the legacy floor at zero; its receipt rows supply the durable maximum. Added tests cover floor preservation, unknown-result retry/reopen, reader snapshot consistency, version exhaustion, migration rollback, receipt-pruning preservation and bounded coverage-tail lookup. |

No blocking source finding was identified in this bounded review. Old experimental v5 databases with the earlier checksum intentionally fail closed; released v1–v4 migration paths remain the supported history.

## Integration gate

After the main agent confirmed all timed measurements had finished and handed over the Cargo slot, the checker started:

```powershell
rtk proxy just ci
```

- Started: `2026-09-19T11:32:01.5944567Z`.
- Log: `%TEMP%/resiwatch-layout2-integration-check-20260919.log`.
- Status: **PASS**, exit 0 on the first run, **81.1169409 seconds** overall.

| Check | Result |
| --- | --- |
| Version alignment, locked dependency refresh, icons | Passed; product version 0.3.0 |
| TypeScript, ESLint, Vitest, Vite build | Passed; 73 test files, 300 tests; 4.42 seconds test execution |
| Rust formatting and workspace/all-target clippy with `-D warnings` | Passed |
| Rust workspace tests | Library: 520 passed, 2 intentionally ignored, 12.04 seconds; isolated crash/retry process integration: 3 passed, 0.39 seconds; binary/doc targets passed with zero tests |
| Root syntax/tests and secret scans | Passed; 139 root tests, 1.8205871 seconds test execution; both secret scans passed |

All new layout2 tests ran in the full workspace gate, including migration failure atomicity, preserved legacy timestamps/prefixes, version floor and retry/reopen, same-snapshot readers, integer exhaustion, receipt pruning, report/period version propagation, corpus floor consistency and bounded coverage-tail behavior. No product or test fixes were necessary in this gate iteration. The checker wrote only this evidence file.

The source was frozen and Cargo handed back to the main agent after completion. This passing run covers layout2; the earlier first-candidate gate remains separately recorded. `npm ci` again reported one high-severity advisory; no manifest, lockfile or dependency change was made.

## Boundaries

The gate refreshes only existing locked workspace dependencies and runs version alignment, frontend icons/typecheck/lint/tests/build, Rust fmt/clippy/workspace tests, root syntax/tests and secret scans. It does not run the ignored full-capacity test or Windows Credential Manager write test. No production database, installed application, dependencies, credentials, commit, push or memory checkpoint is modified by this checker.

Passing this gate establishes automated correctness checks for layout2, not the write-reduction target, full capacity, peak-load or installed-soak acceptance. `AUTO_DELETE_ENABLED` remains false.

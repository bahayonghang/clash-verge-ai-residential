# Layout3 bounded review and full gate — 2026-09-19

This record covers the frozen layout3 source. Earlier evidence in `final-checks.md` and `layout2-checks.md` remains specific to its respective source state.

## Review scope and findings

Hash comparison against `performance-20260919-layout2/candidate-source-manifest.json` found exactly three changed Rust files: `c3/schema.rs`, `storage.rs`, and `bench/corpus.rs`. All other Rust files, Cargo manifests, lockfile and build script in that manifest were unchanged. No blocking finding was identified.

- The new v5 migration drops `idx_connection_minute_utc`, whose `(utc_minute, session_pk)` key order duplicates `sqlite_autoindex_connection_minute_1`. The primary-key index still enforces identity and supports ordered minute-range access. Source search found no query pinned to the removed name. Published v1–v4 DDL remains unchanged, and the drop executes inside the existing transactional v5 migration with checksum `ledger-lifecycle-v5-layout3`.
- The v4 fixture recreates the legacy duplicate index. The new migration regression checks both original index key orders, verifies only the duplicate is removed, checks the range query uses the primary-key index without a temporary sort, and verifies ordered row values survive migration.
- The 64 MiB cache setting applies only to the temporary production-corpus generator's connection. It does not change product connection defaults, WAL/FULL settings or transaction boundaries. The manifest exposes `generation_cache_kib: 65536` and states that generator memory samples do not represent production runtime memory.

The old experimental v5 checksum policy remains fail-closed. This review does not authorize or claim an upgrade of experimental layout1/layout2 fixture databases to layout3.

## Full gate

Command:

```powershell
rtk proxy just ci
```

- Started: `2026-09-19T11:53:21.9062844Z`.
- Log: `%TEMP%/resiwatch-layout3-integration-check-20260919.log`.
- Result: **PASS**, exit code `0`; elapsed `79.6558262 s`.

| Check | Observed result |
| --- | --- |
| Monitor version alignment | Pass, `0.3.0` |
| Frontend icons, TypeScript, ESLint and Vite build | Pass |
| Frontend tests | 73 files; 300 passed; `4.31 s` |
| Rust formatting and strict workspace/all-target clippy | Pass; clippy `4.85 s` |
| Rust library tests | 521 passed, 0 failed, 2 ignored; `11.81 s` |
| Rust process crash/retry integration tests | 3 passed; `0.40 s` |
| Rust binaries and documentation tests | Pass; zero tests |
| Root syntax checks and tests | Pass; 139 passed; `1.6000817 s` |
| Monitor and root template secret scans | Pass |

The new `v4_migration_drops_duplicate_minute_index_without_range_sort` regression passed in this full run. No product source fix or fixture adjustment was necessary. The locked dependency refresh reported one existing high-severity advisory; no dependency definition or lockfile was changed by this check.

The gate includes version alignment, locked dependency refresh, frontend icons/typecheck/lint/tests/build, Rust fmt and strict all-target clippy, Rust workspace tests, root syntax/tests and secret scans. The ignored full-capacity fixture and real Credential Manager write tests are excluded. No performance workloads run concurrently.

## Scope boundaries

The checker changed only this evidence file for layout3. No installed application, production database, credentials, dependency definitions, commit, push or memory note was changed. The main agent owns documentation, specifications, task evidence and subsequent immutable build identities/measurements. Passing checks alone does not establish performance, capacity, peak-load or installed-soak acceptance. Automatic deletion remains disabled.

The gate process has exited and the product source remains frozen. Cargo ownership was returned to the main agent immediately after the successful exit, before its subsequent build and measurements.

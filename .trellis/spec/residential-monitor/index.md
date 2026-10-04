# Residential Monitor Development Guidelines

This Trellis package covers ResiWatch in `residential-monitor/`: a Windows 11
NSIS current-user desktop app (React frontend, Tauri/Rust backend, SQLite
ledger). It does not cover the pasteable Clash Verge extension or the VitePress
docs site. Root `.trellis/spec/frontend/` is the extension and local renderer
only.

Do not migrate layer files. Read the layer that matches the code you change.

## Guidelines Index

| Layer | Path | Focus |
|---|---|---|
| [Backend](./backend/index.md) | `residential-monitor/src-tauri/` | Rust ownership of collection, storage, credentials, tray, query, report, and recovery |
| [Frontend](./frontend/index.md) | `residential-monitor/src/` | React 19 desktop shell; IPC only in hooks |
| [Storage](./storage/index.md) | SQLite ledger | Authoritative ledger; JSON is for small non-secret prefs or export |

## Pre-Development Checklist

- Read the layer `index.md` that matches the files you will change.
- Backend: read `backend/index.md`, then `modules-and-errors.md` and
  `secrets-and-cancellation.md`.
- Frontend: read `frontend/index.md`, then `dto-and-decoding.md` and
  `view-state.md`. Root `.trellis/spec/frontend/` does not apply.
- Storage: read `storage/index.md`, then `sqlite-contract.md`. Performance
  numbers must come from monitor-bench measurements.
- Do not put real credentials in the public extension template. Do not commit
  or hand-edit `*.local.toml` or `*.local.js`. ResiWatch does not read those
  files.

## Quality Check

- Follow [backend Quality Check](./backend/index.md#quality-check).
- Follow [frontend Quality Check](./frontend/index.md#quality-check).
- Follow [storage Quality Check](./storage/index.md#quality-check).
- Local monitor gate: `just monitor-check`. Full local product gate: `just ci`
  (`monitor-check` then `npm run ci`). Neither command builds docs.
- 30-day corpus, 30-minute peaks, 24-hour soak, and installed Windows evidence
  stay manual. CI short samples must not stand in for those gates.

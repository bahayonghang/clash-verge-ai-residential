# residential-monitor backend

Rust 拥有采集、存储、凭据、托盘、查询、报告与恢复。C3 代码在 `src-tauri/src/c3/`。C4 在 `src-tauri/src/c4/`。C5 在 `src-tauri/src/c5/`。

## Pre-Development Checklist

- 读 `modules-and-errors.md`
- 读 `secrets-and-cancellation.md`
- 新依赖必须服务 C0/C1 已批准能力，禁止为演示引入额外运行时。

## Quality Check

- `cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`

## Scenario: owner-window activation recovery

### 1. Scope / Trigger

- Trigger: the tray, a second instance, or a background launch makes the existing owner window visible.
- Scope: `DesktopRuntime::open_window`, `AppFacade::open_main_window`, and the Windows named activation event in `src-tauri/src/lib.rs`.

### 2. Signatures

- `DesktopRuntime::open_window() -> bool` returns `true` only for an owner `hidden -> visible` transition.
- `AppFacade::open_main_window() -> Option<MonitorStreamMessage>` is the single lifecycle recovery seam.
- Existing Tauri command `reconnect_now` remains the explicit manual recovery command; no second collector loop is created.

### 3. Contracts

- Recovery is allowed only in `BootBranch::NormalReady` with a persisted, parseable loopback `settings.address`.
- An empty address must never substitute the UI suggestion `127.0.0.1:9097`.
- `SessionStatus::Cancelled` transitions to `Connecting`; a paused collector resumes through the existing lifecycle input.
- Reopening an already-visible window is a no-op, preserving an explicit disconnect or pause.
- On Windows, a second process signals the owner named activation event and exits before constructing a second Tauri runtime.

### 4. Validation & Error Matrix

| Condition | Result |
| --- | --- |
| owner, hidden, valid loopback | reconnect or resume; publish lifecycle message |
| owner, hidden, empty/non-loopback/invalid address | show window only; remain cancelled/paused |
| owner, already visible | show/focus only; no recovery |
| `RecoveryOnly` or shutdown | show/focus policy only; no collector recovery |
| existing Windows instance | signal owner event; do not open SQLite/Tauri |

### 5. Good/Base/Bad Cases

- Good: `127.0.0.1:9097` persisted, tray reopen reconnects one collector.
- Base: first install has no address, UI may suggest a port but no probe starts.
- Bad: use a remote address or create a timer/collector on every render or window focus.

### 6. Tests Required

- Unit: hidden-to-visible transition reports once; visible window preserves manual cancellation.
- Contract: empty and non-loopback addresses fail closed; persisted loopback survives a fresh `AppFacade::boot` and is eligible on the first `plan_tick`.
- Workspace: fmt, clippy with `-D warnings`, and all Rust tests.
- Manual evidence: installed Windows second-instance, tray, WebView, real controller and Credential Manager remain separate `UNVERIFIED` checks.

### 7. Wrong vs Correct

#### Wrong

```rust
fn open_main_window(&mut self) {
    self.reconnect_now(); // runs on every focus/render and overrides disconnect
}
```

#### Correct

```rust
if self.desktop.open_window() && self.has_valid_controller_address() {
    // recover only once at hidden -> visible, through the existing collector
}
```

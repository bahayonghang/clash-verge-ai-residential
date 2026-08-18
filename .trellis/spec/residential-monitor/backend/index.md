# residential-monitor backend

Rust 拥有采集、存储、凭据、托盘和查询。C0 只交付骨架与 spike。

## Pre-Development Checklist

- 读 `modules-and-errors.md`
- 读 `secrets-and-cancellation.md`
- 新依赖必须服务 C0/C1 已批准能力，禁止为演示引入额外运行时。

## Quality Check

- `cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check`
- `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`

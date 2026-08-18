# residential-monitor storage

权威账本是 SQLite。JSON 只用于小型非敏感偏好或导出。

## Pre-Development Checklist

- 读 `sqlite-contract.md`
- 性能数字必须来自 monitor-bench 实测，不得沿用研究文档估算。

## Quality Check

- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml sqlite_probe`
- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml sqlite_fault`
- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c3::`
- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c4::`
- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib c5::`
- 完整 30 天库与 30 分钟峰值属于手工证据 gate，不得由 CI 短样本冒充。
- C3 自动 DELETE 与完整 `A=50/250/1000` 重跑不得由 CI 短样本冒充完成。
- C5 fixture 并发 / soak smoke / `c5-baseline` 不得写成 30 天容量、24 小时 soak 或已通过的 C0 升级。

# residential-monitor storage

权威账本是 SQLite。JSON 只用于小型非敏感偏好或导出。

## Pre-Development Checklist

- 读 `sqlite-contract.md`
- 性能数字必须来自 monitor-bench 实测，不得沿用研究文档估算。

## Quality Check

- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml sqlite_probe`
- `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml sqlite_fault`
- 完整 30 天库与 30 分钟峰值属于手工证据 gate，不得由 CI 短样本冒充。

# C4 Gate 状态

生成时间：2026-08-18。完整 30 天 `A=50/250/1000` 重跑、24 小时 soak、代码签名、GitHub Release、NSIS 安装态通知与 Focus Assist 真机验收未执行。

## Gate

| Gate | 结果 | 证据 |
|---|---|---|
| 1 C3 复用契约 | 通过 | `c4::period` 只调用 `ReportService` 与 `local_day_bounds` / `local_month_bounds`；schema 无第二套小时 / 日 / 月表 |
| 2 前向 schema | 通过 | `user_version=3` / checksum `c4-alert-v3`；C1 `c1-core-v1` 与 C3 `c3-report-v2` 未改 |
| 3 AlertEngine | 通过 | `rate_needs_three_consecutive_hits`；`hysteresis_keeps_active_inside_band`；`gap_is_not_zero_rate`；`health_dedups_same_root_cause` |
| 4 周期用量 | 通过 | `period_observation_matches_c3_report`；能力不足 / 未知时区返回 `not_evaluable`，观测值不是零 |
| 5 原子提交 | 通过 | `kill_after_alerts_rolls_back_facts_and_outbox`；`retry_same_bundle_does_not_duplicate_event` |
| 6 outbox | 通过 | lease 互斥、stale reclaim、退避、永久失败可见 |
| 7 通知与告警中心 | 部分 | Fake / Windows seam 与告警页已交付。未做 NSIS 普通用户 / Focus Assist 真机发送 |
| 8 脱敏诊断 | 通过 | `diagnostics_omit_secret_and_full_host`；导出扫描 `bearer ` / `password=` / `secret=` |
| 9 性能 | fixture | 80 规则 × 30 帧热路径 SQL=0，耗时 < 500 ms。未重跑 10k×30 分钟完整库 |
| 10 独立验证 | 见 `c4-ac-evidence.md` | `just monitor-check` 与 `just ci` 记录于对话 |

## 暂停项

- 未执行 `just tinstall`、登录自启动、Credential Manager 真机写入或系统通知真机发送。
- 未重跑完整 30 天三档库、24 小时 soak、代码签名、GitHub Release。
- 安装态普通用户通知 / Focus Assist 真机验收未做。

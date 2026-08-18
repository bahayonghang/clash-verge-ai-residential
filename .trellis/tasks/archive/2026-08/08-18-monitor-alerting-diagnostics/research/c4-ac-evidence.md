# C4 验收证据

| 项 | 命令或文件 | 结果 |
|---|---|---|
| C4-AC1 依赖与口径 | C3 `c3-gate-status.md` 已独立验收；`c4::period` 复用 `ReportService`；`C4_DDL` 无 `traffic_hourly` | 通过。C3-AC10 完整三档 30 天库未重跑，本任务不扩大容量声明 |
| C4-AC2 健康闭环 | `c4::engine::health_dedups_same_root_cause`；默认 7 条 health 规则 | 通过。同根因第二次评估不重复 Activated |
| C4-AC3 速率规则 | `rate_needs_three_consecutive_hits`；`hysteresis_keeps_active_inside_band`；`gap_is_not_zero_rate` | 通过。第 1、2 次不触发；滞回带不恢复；缺口不是零速率 |
| C4-AC4 周期规则 | `period_observation_matches_c3_report`；`local_day_bounds_handle_dst_spring_forward`；`local_month_bounds_follow_calendar_length` | 通过。DST 春拨日 23h；2 月 28 天。能力不足可见且观测值为空 |
| C4-AC5 原子性 | `kill_after_alerts_rolls_back_facts_and_outbox`；`retry_same_bundle_does_not_duplicate_event` | 通过。事务失败后 receipt / event / outbox 均为 0 |
| C4-AC6 outbox 恢复 | `lease_then_sent`；`retry_uses_backoff_then_stale_reclaim`；`double_claim_same_tick_is_exclusive`；`permanent_failure_is_visible` | 通过 |
| C4-AC7 通知与告警中心 | `notify_seam_tests`；alerts 路由可用；`test_notification` 不写告警历史 | 部分。真实 NSIS / Focus Assist 未执行（暂停条件） |
| C4-AC8 脱敏诊断 | `diagnostics_omit_secret_and_full_host` | 通过 |
| C4-AC9 性能与有界性 | `many_rules_do_not_issue_sql_on_hot_path`；沿用已归档 C1 replay p95=21ms | fixture 通过。未重跑 10k×30 分钟完整组合压测 |
| C4-AC10 独立验证 | `just monitor-check` 退出码 0；`just ci` 退出码 0 | 自动质量门通过。安装态通知 smoke 未跑 |

## 未执行

- 完整 30 天 `A=50 / 250 / 1000` 库重跑
- 24 小时 soak、代码签名、GitHub Release
- 本机 NSIS 安装、登录自启动、Credential Manager 真机写入、系统通知真机发送

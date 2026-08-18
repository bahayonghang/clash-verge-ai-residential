# C2 验收证据

| 项 | 命令或文件 | 结果 |
|---|---|---|
| C2-AC1 依赖门 | `c2_only_names_frozen_c1_owners`；`normal_boot_does_not_create_report_tables`；C2 无 rusqlite / create table | 通过。C1 gate 见归档 `c1-gate-status.md` |
| C2-AC2 生命周期 | `desktop_lifecycle_tests` | 自动化通过。Windows 11 安装态托盘 / 自启动 / 单实例未跑（暂停：不写本机安装与登录自启动） |
| C2-AC3 凭据与引导 | `settings_workflow_tests`；secret scan | Fake 与进程内临时 secret 通过。本机 Credential Manager 未写入 |
| C2-AC4 原子订阅 | Rust `channel_contract_tests`；TS `reducer.test.ts` | bootstrap / resync / gap / 重复 / 旧订阅迟到消息通过 |
| C2-AC5 实时口径 | `LiveOverview` 分字段；前端概览分别渲染 meter / attributed / 分类 / 其他 / gap / over / coverage / health | 缺口字段为 `null`，显示“未知”，不写零 |
| C2-AC6 连接控制 | `random_order_does_not_change_identity_page`；`missing_id_204_is_accepted_until_remove`；fixture DELETE 204 | 通过。无关闭全部入口 |
| C2-AC7 10k 性能 | `peak_10k_1800_frames_stay_bounded_and_fast_to_filter`（CI：30 帧×10k）；`c2-peak-ui.json`（1800 帧） | 短时实时路径。不是 30 天容量 |
| C2-AC8 Recovery Shell | `future_schema_enters_recovery_without_writer`；`restore_available=false` | 通过。restore 标为 C3 |
| C2-AC9 应用壳 seam | `reports_and_alerts_are_unavailable_until_child`；`file_dialog_returns_only_injected_path`；`operation_progress_can_cancel_fixture`；`routes.test.ts` | 通过 |
| C2-AC10 可用性与安全 | CSP 仍为本地资源；capability 无 fs / SQL / opener；secret 不进 DTO；键盘焦点与高对比 CSS | 走查覆盖静态壳。真机高 DPI / 安装态未跑 |
| C2-AC11 独立回滚 | C2 不拥有 migration；`MIGRATION_CHECKSUM` 仍为 `c1-core-v1` | 可停用 UI 保留 C1 内核 |

## 未执行

- 本机 NSIS 安装、登录自启动写入、Credential Manager 真机 CRUD、系统通知
- Clash Verge 真机、24 小时 soak、代码签名、GitHub Release

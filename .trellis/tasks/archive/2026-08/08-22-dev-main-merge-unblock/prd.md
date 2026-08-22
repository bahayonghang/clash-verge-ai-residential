# 解封 dev 合并 main：生产端口与恢复后状态

父任务。子任务各自独立验收。

## 来源

Cursor 对 `dev` → `main` 的合并审查（2026-08-22）。本任务只承接核对后成立且构成合并门槛的条目。

`dev` 相对 `main`：862 文件、+96,654 / −397 行。按目录分：`residential-monitor` 64,791 行（67%）、`.trellis/tasks` 25,177 行（26%）、`clash-verge-ai-residential.js` +362 / −74。`residential-monitor` 首次落到 `main`。

## 审查条目核对结果

| 审查条目                                                                                | 结果           | 处置                                                    |
| --------------------------------------------------------------------------------------- | -------------- | ------------------------------------------------------- |
| 生产 `boot()` 装配 `FakeFileDialog`，`pick_file` 恒返回 `None`                          | 成立           | 子任务 `08-22-prod-shell-ports-real`                    |
| `WindowsNotificationSink` 未在 `boot()` 构造，`FakeNotificationSink` 类型名进入产品文案 | 成立           | 同上。已定：接 `tauri-plugin-notification` 发真实 toast |
| restore / vacuum / 删除只改 `storage` + `branch`，不重跑存储侧启动                      | 成立           | 子任务 `08-22-storage-reboot-after-recovery`            |
| 路由脚本无条件写 `find-process-mode: always`                                            | 不构成缺陷     | 不处理，见下                                            |
| `AppFacade` 2469 行、35 个 `pub` 字段，C3/C4/C5 经它转发                                | 成立           | 不在本任务，见下                                        |
| `lib.rs` 1374 行、约 50 条命令 + 托盘 + 单实例                                          | 成立           | 不在本任务                                              |
| 两份 `decodeSettings`、10 处 `as unknown as`                                            | 成立           | 不在本任务                                              |
| `useSettings` 708 行                                                                    | 成立           | 不在本任务                                              |
| `FakeAutostart` 在生产字段                                                              | 成立但无调用点 | 不在本任务                                              |
| `start_operation` 走 `start_fixture`                                                    | 成立，命名误导 | 不在本任务                                              |

### 路由脚本 `find-process-mode: always` 不处理的依据

该行为是已归档任务 `08-22-process-lookup-observation`（父任务 `08-22-process-attribution`）的交付物，决策记录在父任务 PRD 的 Background（Grill，2026-08-22）：

- R1 已写明「查找进程与进程路由分开」。`routing.ai_process_fallback` 仍默认 false，不注入 `PROCESS-NAME` / `PROCESS-PATH`。
- 覆盖用户既有值（含 `off`）是 AC1 的显式要求，不是遗漏。
- 「新 TOML 开关」在该任务的 Out of scope 内。
- R2 要求的文档已交付：`docs/configuration.md:65,90`、`docs/local-configuration.md:87`、`CHANGELOG.md:19`。
- `always` 是目标所需。`strict` 只在规则需要时查找进程，不会为未匹配 PROCESS 规则的连接提供 process identity，无法满足监控读取 identity 的要求。

审查把该决策判定为回归，依据不成立。若要为独立粘贴脚本的用户提供退出方式，属新需求，另开任务。

### 结构条目不在本任务的依据

`AppFacade` 拆分、`lib.rs` 拆分、IPC 单一解码器、`useSettings` 拆分：核对成立。这些是行数与所有权问题，不产生用户可见的错误行为。`.github/workflows/ci.yml` 只有测试作业，无 release 或产物发布作业，合并 `main` 不分发桌面应用二进制。因此这些条目不构成合并门槛，另开重构任务。

审查提出的「生产 `facade.rs` < 1000 行」「`lib.rs` < 1000 行」为行数阈值，不作为验收条件。

## 子任务

| 目录                                  | 交付                                                                                            |
| ------------------------------------- | ----------------------------------------------------------------------------------------------- |
| `08-22-prod-shell-ports-real`         | `boot()` 交付真实文件对话框与通知 sink；测试替身只在测试构造                                    |
| `08-22-storage-reboot-after-recovery` | restore / vacuum / 删除后重跑存储侧启动，刷新 `writer_epoch`、告警规则与实例、targets、settings |

两个子任务都改 `c2/facade.rs` 的 `boot()` 与 `recovery_only()`。先做 `prod-shell-ports-real`，它移除字段；再做 `storage-reboot-after-recovery`，它提取启动序列。反序会产生一次重复修改。

## 跨子任务验收

- [x] PAC1：两个子任务合并后 `just ci` 通过。
- [x] PAC2：两个子任务合并后 CI 的 `monitor` 作业全部步骤通过：`npm --prefix residential-monitor run check`、`cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings`、`cargo test --workspace`、`npm run check:secrets`。
- [x] PAC3：`residential-monitor/src-tauri/src/c2/facade.rs` 的 `AppFacade` 结构体不再有类型为 `Fake*` 的字段（`FakeAutostart` 除外，见 Out of scope）。
- [x] PAC4：`clash-verge-ai-residential.js`、`scripts/sync-local-config.js`、`tests/regression.test.js` 无改动。
- [x] PAC5：`src-tauri/capabilities/default.json` 若因通知插件改动，其 description 同步更新，且仍不含文件系统、opener、SQL、凭据权限。文件对话框不得引入文件系统权限。

## Out of scope

- 路由脚本、`scripts/sync-local-config.js`、`tests/regression.test.js` 的任何改动。
- `AppFacade` / `lib.rs` / `useSettings` / `dto.ts` 拆分，IPC 解码器合并。
- `FakeAutostart` 与 `start_operation` / `start_fixture`。
- `FakeCredentialStore`（`facade.rs:331`、`:388`）。`lib.rs:311-325` 的 `attach_windows_credentials` 已在 Windows 上用 `credential::windows_cm::WindowsCredentialManager` 换掉它，不是交付到生产的测试替身。非 Windows 构建会用到它，v1 只支持 Windows 11。
- `NotifyCapability.reason_zh` 等 `*_zh` DTO 字段在英文 locale 下返回中文的 i18n 缺陷（`c4/notify.rs:109,117,124` 硬编码中文，未过 `t()`）。属既有跨端契约问题，另开任务。
- 告警静默。`src/i18n/zh.ts:503` 的文案「静默只抑制通知，不删除事件」已向用户承诺该功能，`src-tauri/src/c4/` 无对应实现，全仓搜索 `silence` / `mute` / `suppress` 无结果。真实 toast 上线后用户无应用内关闭入口，只能用环境变量。该缺口另开任务。
- `src/i18n/zh.ts:542` 的 `alerts.notify_on` 文案「通知 seam 可用」把内部术语 seam 显示给用户。
- 实际执行 `dev` → `main` 的合并。

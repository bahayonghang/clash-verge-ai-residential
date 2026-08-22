# 实施计划：家宽监控本机日志

## 启动前门禁

- [ ] 用户已批准本任务最新规划摘要。
- [ ] 已 `task.py start`，状态为 `in_progress`。
- [ ] 已读 `trellis-before-dev` 与 residential-monitor backend / frontend / storage checklist。
- [ ] 未获确认前不跑 `just tinstall`、Credential Manager 真机写入或登录自启动写入。真机走查用 `just tdev`。

## 执行顺序

### 1. `redact` 与 `app_log`

- 新增 `src-tauri/src/redact.rs`：迁入 C4 `FORBIDDEN` 与 `scan_text_for_secrets`。`c4/diagnose.rs` 改为调用它。
- 新增 `src-tauri/src/app_log.rs`：`resolve_dir`、`init`、`dir`、`emit`、轮转、panic hook。测试用 `RESIDENTIAL_MONITOR_LOG_DIR` + 可注入体积上限。
- `lib.rs` / `main.rs` 声明模块。不改 `Cargo.toml` 依赖，除非无法通过 clippy。

**Gate 1**：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --lib redact:: app_log::`（或模块测试名）覆盖脱敏、轮转、DATA_DIR 不影响日志目录。诊断原测试仍通过。

### 2. 启动接线

- `run()` 最先 `init` + `emit boot`。
- 单实例 `FocusExisting` 先 emit 再 return。
- `boot_facade`：记录 `storage_open` class 与 `branch`；打开失败不再静默丢弃类别。
- `AppFacade` 增加 `last_logged_session`；`apply_probe_err` / 成功连上时只在变迁时 `emit session`。
- `pause_collector` / `resume_collector` / `reconnect_now` / `shutdown` 在既有状态机之后 emit。
- C3 备份恢复保留、C4 规则/outbox 永久失败、C5 VACUUM/删除：在现有返回路径上 emit，不改业务结果。

**Gate 2**：库打开失败夹具产生 `storage_open` ERROR 行且无 secret；同一 `tcp_unauthorized` 连续两次只一行 `session`。

### 3. 打开目录、DTO、删除

- `BootstrapDto.logDir`。
- command `open_log_dir`：只 `spawn explorer` 模块目录。
- `preview_delete` / `confirm_delete` 增加 `log_dir`；声明 `logs`。
- invoke_handler 注册 `open_log_dir`。capability 不增加权限。

**Gate 3**：purge 测试覆盖日志目录；错误短语不删日志。

### 4. 前端与文档

- `dto.ts` / `previewBootstrap` / 设置数据区块 / `renderRecovery`：显示 `logDir`，按钮 `open_log_dir`。
- `zh.ts` / `en.ts` 成对键。路径放文本，不拼 `file://`。
- `docs/data-directory.md`、`troubleshooting.md`、`privacy.md` 写路径、轮转、事件与打开方式。

**Gate 4**：`npm --prefix residential-monitor` typecheck / lint / test / build。缺失 `logDir` 时按钮不可用。

### 5. 质量门与真机

- fmt / clippy / cargo test workspace / npm 门 / `npm run check:secrets`。
- `just tdev`：设置页打开资源管理器到 logs；人为坏库或 Recovery 夹具下 Recovery 壳同样可打开。

**Gate 5**：AC1–AC8 在文件与界面可观察。不跑 `tinstall`。

## 验证命令

```
cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check
cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace
npm --prefix residential-monitor run typecheck
npm --prefix residential-monitor run lint
npm --prefix residential-monitor test
npm --prefix residential-monitor run build
npm run check:secrets
```

真机：`just tdev`。不要 `just tinstall`。

## 风险文件与回滚

- `src-tauri/src/lib.rs`：启动顺序与 command 注册。
- `src-tauri/src/c2/facade.rs`：boot 错误分类、`logDir`、session 去重。
- `src-tauri/src/c4/diagnose.rs`：扫描迁出。
- `src-tauri/src/c5/purge.rs`：删除清单。
- `src/main.ts`、`src/dto.ts`、i18n、docs。

回滚：去掉 `app_log` 调用与 UI 按钮，C4 扫描可留在 `redact.rs`。不碰 migration。

## `task.py start` 前检查

- [ ] `prd.md` 无阻塞开放问题，已做收敛。
- [ ] `design.md` 与 `implement.md` 已写。
- [ ] `implement.jsonl` / `check.jsonl` 已有真实 spec 条目。

# Implement：监控数据目录迁移出 Temp

## 顺序清单

1. 新建 `src-tauri/src/data_dir.rs`：
   - `prepare_data_dir() -> PathBuf`（env → exe 默认 → 迁移编排，`app_log` 记 `data_dir_migrated` / `data_dir_migration_failed` / `data_dir_skip`）。
   - 纯函数核 `resolve(explicit, exe_dir, legacy)` 与 `migrate_legacy(legacy, target) -> MigrationOutcome`，全部可注入路径，测试用 tempdir。
   - 单测：整目录 rename、目标已存在走逐项、目标已有库跳过、legacy 无库 no-op、失败保 legacy。
2. `lib.rs`：`mod data_dir;` 声明 + `boot_facade()` 内替换 307-310 四行为 `prepare_data_dir()` 调用。
3. `src-tauri/installer.nsh`：`NSIS_HOOK_PREUNINSTALL` / `NSIS_HOOK_POSTUNINSTALL` 搬出搬回 `$INSTDIR\data`；`tauri.conf.json` `bundle.windows.nsis.installerHooks: "./installer.nsh"`。
4. `docs/data-directory.md`：默认路径改为 `<安装目录>\data`，补迁移与卸载保留说明。
5. 验证：
   - `cargo test`（src-tauri 全量 + 新模块单测）。
   - 迁移守恒实测：以真实 Temp 库副本为 legacy，跑迁移，SQL 对比 `connection_minute` 行数与 upload/download 总量。
   - `npm run tauri:build` 产出安装包 → 安装、卸载，确认 `data\` 保留（A5）。构建工具链不可用时如实报告阻塞。

## 验证命令

```bash
cd residential-monitor/src-tauri && cargo test
cd residential-monitor && npm run tauri:build
```

## 风险文件与回滚点

- `lib.rs`（boot 入口）：改动 4 行，回滚即删 `data_dir.rs` 引用。
- `tauri.conf.json`：只加 `installerHooks` 一键。
- 迁移函数对真实数据只 move 不改写；最坏情况 = 维持现状（Temp）。

## task.py start 前检查

- prd.md / design.md / implement.md 就绪。
- `implement.jsonl` / `check.jsonl` 已含真实条目（lib.rs、facade boot、purge.rs、docs/data-directory.md、findings.md）。

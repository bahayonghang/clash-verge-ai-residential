# Design：监控数据目录迁移出 Temp

## 现状与目标

- 现状：`lib.rs:307-309` 默认数据目录 = `std::env::temp_dir().join(IDENTIFIER)`。
- 目标：默认 = `current_exe()` 同级 `data\` 子目录；既有 Temp 数据首启自动迁移；卸载保留 `data\`。

## 边界

| 决策点 | 选择 | 理由 |
|---|---|---|
| 新模块 | `src-tauri/src/data_dir.rs` | 路径解析 + 迁移是独立生命周期关注点，纯函数化便于测试 |
| 解析优先级 | `RESIDENTIAL_MONITOR_DATA_DIR` > `<exe_dir>\data` > 失败回退 legacy（Temp） | env 覆盖是既有契约（测试/bench 依赖），不变 |
| 迁移时机 | `boot_facade()` 内、`AppFacade::boot` 之前 | 单实例 claim 已完成：`FocusExisting` 已提前退出，迁移时无本应用进程持有旧库，文件未打开，rename 安全 |
| 快路径 | `fs::rename(legacy_dir, target_dir)` 整目录原子改名 | 本机 Temp 与安装目录同卷（C:）；目录内只有应用数据（monitor.sqlite3/-wal/-shm、report-spool/、archive-tick/，已实地核实） |
| 慢路径 | 逐项 move：sqlite 三件套 rename（跨卷时 copy+size 校验+删源），目录项整体 rename；成功后 `remove_dir_all(legacy)` | 目标已存在（如上次部分迁移）时整目录 rename 会失败 |
| 目标已有库 | 不迁移、不动 legacy，记日志 | 无法判定新旧，删除任何一侧都不安全 |
| 失败语义 | 任何错误 → 记 warn、沿用 legacy 目录（现状），下次启动重试 | 迁移幂等；应用永不因迁移失败而不可用 |
| 卸载保留 | NSIS `PREUNINSTALL` 把 `$INSTDIR\data` 搬到 `$TEMP\residential-monitor-data-keep`，`POSTUNINSTALL` 搬回 | Tauri 卸载器结尾 `RMDir /r $INSTDIR`，hook 搬走再搬回是唯一不改内核的保留方式；Tauri 2.11.5 支持 `installerHooks` |

## 数据流

```
boot_facade()
  └─ data_dir::prepare_data_dir()
       ├─ env override → 原样返回（不迁移）
       ├─ exe_dir 不可得 → legacy（Temp）目录
       ├─ legacy 无 monitor.sqlite3 → target = exe_dir\data，仅 create_dir_all
       ├─ target 已有 monitor.sqlite3 → 跳过，沿用 target，legacy 原样保留
       ├─ rename 整目录 → 成功即迁移完成
       └─ 失败 → 逐项 move + 校验 + 清理 legacy；再失败 → 沿用 legacy
```

## 兼容与回滚

- `RESIDENTIAL_MONITOR_DATA_DIR` 行为完全不变；测试与 bench 不受影响。
- 迁移只移动文件，不改 schema、不改 `connection_minute` 内容；守恒校验 = 文件级（存在性 + copy 后 size 相等），行数守恒由验收时 SQL 实测一次。
- 回滚 = 还原 `lib.rs` 三行 + 删除 `data_dir.rs`；已迁移的数据目录对新代码与旧代码同样可用（路径是唯一变化）。
- dev 态：exe 在 `target/debug` 时数据落 `target/debug/data`，可被 `cargo clean` 清除；需要稳定目录时用 env 覆盖（现状已是此模式）。

## 风险

- NSIS hook 依赖 Tauri 模板的 hook 插入点（`NSIS_HOOK_PREUNINSTALL` / `POSTUNINSTALL`）；构建一次安装包手测卸载才能闭环（验收 A5）。
- `$TEMP` 中转目录在卸装后未搬回（如用户卸载后清了 Temp）→ 数据丢失窗口极小，hook 内不做额外兜底。
- 整目录 rename 后 `report-spool` 内旧 token 路径失效：spool 本就是按 `data_dir` 派生的相对内容（`snapshot.rs` 用 `data_dir.join("report-spool")`），无绝对路径存储，安全。

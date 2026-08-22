# 技术设计：家宽监控本机日志

## 边界

进程级模块 `app_log` 拥有目录解析、文件轮转、脱敏写入和「打开日志目录」。C2/C3/C4/C5 只调用 `app_log::emit`，不各自 `OpenOptions`。C5 删除清单调用同一解析函数，避免日志目录与删除项不一致。

`app_log` 不读 SQLite，不解释 mihomo payload，不订阅 Channel。写失败返回 `()`，调用方忽略。

前端只展示 `BootstrapDto.logDir` 并 `invoke("open_log_dir")`。WebView 不读文件，不获 `fs` / opener / log plugin。

## 拒绝 `tauri-plugin-log`

架构原文建议该插件。本任务不用：

1. 插件在 `tauri::Builder` 里初始化，赶不上 `boot_facade` 的库打开失败。
2. 给 WebView 插件权限与最小 capability 冲突；Rust 侧单独用插件仍无法覆盖 Builder 之前。
3. 插件不提供与 C4 相同的禁止子串扫描。

也不把第三方 crate 的 `log` 记录桥进同一文件，避免 Tauri/WebView 内部字段漏进磁盘。

## 目录

```
resolve_log_dir:
  RESIDENTIAL_MONITOR_LOG_DIR 若设置 → 该路径
  否则 Windows：%LOCALAPPDATA%\io.github.bahayonghang.residential-monitor\logs
  测试非 Windows：临时目录下 identifier/logs（产品不验收）
```

`RESIDENTIAL_MONITOR_DATA_DIR` 不参与。缺省 `data_dir` 仍是临时目录（本任务不改）。

文件：`residential-monitor.log`，轮转 `residential-monitor.log.1` … `.4`。当前文件超过 `2 * 1024 * 1024` 字节时：删除 `.4`，`.3`→`.4` … `.1`→`.2`，当前 → `.1`，再建当前文件。测试可注入更小阈值。

## 行格式

UTF-8 单行：

```text
{utc_rfc3339} {LEVEL} {event} {json}
```

`LEVEL` 为 `INFO` / `WARN` / `ERROR`。`event` 为稳定英文码。`json` 对象只含白名单键；值仅为 `string`（事件码、类别、版本）/ `number` / `bool`。编码后跑与 C4 相同的扫描；命中则该值改为 `"<redacted>"`。禁止把 `StorageError` / `rusqlite::Error` 的 `Display` 写入 json。

存储失败只记 `class`：`sqlite` | `closed` | `ok`。

## 事件码

| event | 级别 | 字段 |
|---|---|---|
| `boot` | INFO | `launch`, `branch`, `version` |
| `instance_focus_existing` | INFO | （无） |
| `storage_open` | INFO/ERROR | `class`；失败为 ERROR |
| `collector_pause` | INFO | |
| `collector_resume` | INFO | |
| `reconnect` | INFO | |
| `session` | INFO/WARN | `from`, `to`（`session_status_name`）；`connected` 用 INFO，其余 WARN |
| `shutdown` | INFO | `phase`（`ShutdownPhase` kebab） |
| `backup` / `restore` / `retention` / `vacuum` / `delete` | INFO/ERROR | `ok`；删除可加 `failed` 计数 |
| `alert_rule` | INFO | `id` 的短哈希或稳定规则 id（已是内部 id，不含 secret） |
| `notify_unavailable` | WARN | `class` |
| `outbox_failed` | WARN | `error_class` |
| `open_log_dir` | INFO/ERROR | `ok` |
| `panic` | ERROR | `class=panic`；payload 经扫描，过长截断 |

`session` 只在 `from != to` 时写。`AppFacade` 记下 `last_logged_session: Option<SessionStatus>`。采集成功帧不 emit。

## 初始化顺序

```
run()
  resolve_log_dir + create_dir_all
  app_log::init（打开当前文件、安装 panic hook）
  emit boot（launch 先按 args 解析）
  单实例；FocusExisting → emit 后 return
  boot_facade；按 StorageCoordinator 结果 emit storage_open 与最终 branch
  tauri::Builder …
```

`boot_facade` 今日丢弃 `StorageError`。改为 `open` 结果先分类再 `emit`，仍进入 `RecoveryOnly`。不把错误字符串送前端。

panic hook：扫描后 `emit panic`，再调用默认 hook。`run().expect` 仍会终止进程，但文件里已有行。

## 打开目录

`open_log_dir` command：

1. `create_dir_all(resolve_log_dir())`
2. Windows：`Command::new("explorer").arg(dir).spawn()`。只传模块解析出的 `PathBuf`，不接收前端路径。
3. 不 `wait`（explorer 退出码不稳定）。
4. spawn 失败 → `AppErrorDto` code `open_log_dir`，retryable true，action 为打开数据/日志说明。
5. emit `open_log_dir`。

不新增 FilePurpose，不走 `FileDialogPort`。

## DTO 与前端

`BootstrapDto` 增加 `logDir: string`（serde camelCase）。`schemaVersion` 保持 `1`（与 `uiLocale` / `uiTheme` 相同加法）。Rust 始终填解析后的路径。前端 `previewBootstrap` 与设置/Recovery 模板同步。缺字段时按钮禁用并显示「日志目录未知」，不猜 `%LOCALAPPDATA%`。

设置「数据」区块与 Recovery 面板：只读路径 +「打开日志目录」。中英文键成对。路径进文本节点，不拼 `file://`，不进 `innerHTML` 属性以外的用户输入。

## 删除

`c5::preview_delete` / `confirm_delete` 增加 `log_dir: &Path` 参数。声明项 `id=logs`，`kind=directory`。`AppFacade` 传入 `app_log::dir()`。测试传入 `tempdir`，禁止测到本机 LocalAppData。

删除成功后 logger 可能指向已删文件：下一次 `emit` 重新 `create_dir_all` 并打开当前文件。不在删除过程中保持过期句柄为唯一写入点。

## 脱敏共享

把 `c4/diagnose.rs` 的 `FORBIDDEN` 与 `scan_text_for_secrets` 抽到顶层 `redact.rs`。`diagnose` 与 `app_log` 共用。禁止两份列表。

## 依赖

只增加直接依赖 `log`（可选，若 `emit` 不用 facade 则可零新依赖）。**推荐零新依赖**：`app_log::emit` 自行写文件，不安装全局 `log::Log`。`Cargo.toml` 保持现有集合，除非 clippy 另有要求。

`monitor-bench` 不初始化产品 logger，stdout JSON 不变。

## 测试

`app_log.rs`：

- 解析：有/无 `RESIDENTIAL_MONITOR_LOG_DIR`；DATA_DIR 不影响日志目录
- 脱敏：secret、`password=`、host
- 轮转：阈值 64 字节时文件数 ≤ 5
- 写失败：只读目录或不存在父路径处理后采集测试仍通过（emit 不 panic）
- session 去重：相同 `to` 第二次不追加

`c5::purge`：预览含 `logs`；确认后日志目录文件消失；短语错误不删日志。

前端：`logDir` 缺失时 Recovery/设置按钮不可用；不把路径写进 `innerHTML` 事件处理以外的拼接（沿用文本节点）。

## 回滚

删除 `app_log` / `redact` 接线、`logDir` 字段、打开按钮与 purge 项。C4 诊断扫描改回模块内常量亦可。不碰 SQLite schema。

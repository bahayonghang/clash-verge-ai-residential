# 技术设计：托盘状态与左右键

## 边界

Rust 拥有托盘图标、tooltip、菜单可用性与点击分流。映射函数放在 `c2/desktop.rs`，用 `(collector_running, session, storage_ok)` 纯输入，不依赖 Tauri。`lib.rs` 只负责 Tauri 接线与把 `AppFacade` 字段送进映射。

前端继续只通过 `tray_summary` 读 `collectorRunning` 画实时空态。本任务不扩展该 DTO，不在 WebView 里 `set_icon`。

窗口 / 任务栏图标继续用 `icons/icon.png`。托盘四态资源是另一组文件。

## 数据流

```
collector tick / 暂停 / 继续 / 重连 / 语言切换 / 设置导致的会话变化
        │
        ▼
AppFacade: desktop.collector_running + hub.overview().health
        │
        ▼
tray_chrome(running, session, storage_ok)  →  TrayVisual + tooltip_session
        │
        ├── set_icon（仅 visual 变化）
        ├── set_tooltip（产品名 — health_title）
        └── 重建菜单（仅 collector_running 或 locale 变化）
```

`health.session` 与 `storage_ok` 已由 hub 投影。托盘不读 SQLite，不解释 mihomo payload。

## 映射合同

`TrayVisual`: `Collecting` | `Connecting` | `Paused` | `Fault`。

优先级与 PRD R2 表相同：未采集 → `Paused`；采集中且存储不健康 → `Fault`；`connected` → `Collecting`；`connecting` / `core_restarted` → `Connecting`；其余会话码 → `Fault`。

tooltip 键：`Paused` 用 `paused`；存储不健康用 `storage_failure`；其余用 `session`。`i18n::health_title` 得到标题；空字符串时回落会话码。拼 `"{product.display_name} — {title}"`。

`TraySummary` 保持现有三字段，供前端空态使用。映射测试不走该 DTO。

## Tauri 接线（2.11.5）

- `TrayIconBuilder::show_menu_on_left_click(false)`。默认值是 `true`，这是当前左键弹出菜单的原因。
- `on_tray_icon_event`：`TrayIconEvent::Click { button: Left, button_state: Up, .. }` 与 `DoubleClick { button: Left, .. }` 都调用与菜单 `"open"` 相同的 `open_main_window`。忽略 `Down`、右键、中键。右键菜单由已设置的 `Menu` 弹出。
- `MenuItemBuilder::enabled`：暂停 `enabled(running)`，继续 `enabled(!running)`。
- 打开窗口路径继续：`desktop.open_window()` + `show` + `set_focus`。

## 资源

| 文件 | 用途 |
|---|---|
| `icons/icon.png` 及 ico / 32 / 128 | 窗口与任务栏，无色点 |
| `icons/tray-collecting.png` | 绿点 |
| `icons/tray-connecting.png` | 蓝点 |
| `icons/tray-paused.png` | 黄点 |
| `icons/tray-fault.png` | 红点 |

四枚托盘 PNG 从现有 `icon.png` 派生，画布与源图相同（512×512），白底房子标记不变，右下角实心色点带深色描边。Windows 通知区自行缩小。色点不跟随 `ui_theme`。嵌入用 `tauri::include_image!`。

建议色（白底上可扫读）：绿 `#3D8B3D`，蓝 `#1E66F5`，黄 `#D99A1A`，红 `#D20F39`。

## 刷新

`sync_tray_chrome(app)` 读取 facade，计算 chrome，再应用到 id `main` 的托盘。

调用点：

- `collector_loop_tick` 每次结束（含未取帧），保证暂停后下一拍也一致
- 托盘菜单 pause / resume / reconnect 之后立刻调用，不等下一拍
- `pause_collector` / `resume_collector` / `reconnect_now` command 之后
- `apply_locale_chrome`（语言切换已重建菜单，这里补图标与 tooltip）
- `test_controller` / `disconnect_controller` / `save_settings` 等会改 `session_status` 的入口之后

`DesktopRuntime` 记下上次已应用的 `TrayVisual` 与 `collector_running`。visual 未变则不 `set_icon`。`collector_running` 或 locale 未变则不重建菜单。tooltip 字符串变化才 `set_tooltip`。

Recovery-only 启动时会话常为 `endpoint_missing`，映射为故障红点。不为此分支另做图标。

## 测试

`c2/desktop.rs` 现有 lifecycle 测试旁增加表驱动用例，至少覆盖：

- 采集中 + `connected` + 存储健康 → Collecting，tooltip `connected`
- 采集中 + `connecting` / `core_restarted` → Connecting
- `collector_running == false` + `connected` → Paused，tooltip `paused`
- `collector_running == false` + 故障会话 → 仍 Paused
- 采集中 + `tcp_unauthorized` / `endpoint_missing` / `cancelled` → Fault
- 采集中 + `connected` + `storage_ok == false` → Fault，tooltip `storage_failure`

不在测试里创建真实托盘。点击分流用代码审查 + 真机走查 AC1/AC2。

## 兼容与回滚

- 不改 Channel / bootstrap / `TraySummary` 字段。
- 不改暂停、继续、重连、退出的 `ControllerInput` 映射。
- 回滚：恢复 `show_menu_on_left_click` 默认行为、单枚 `default_window_icon`、固定产品名 tooltip。数据库与凭据不受影响。

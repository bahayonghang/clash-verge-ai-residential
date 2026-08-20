# 设计：设置关于页与侧栏宽度

## 边界

父任务定义跨子任务契约。关于页子任务不改壳宽度。侧栏子任务不改 AboutDto 语义。两者都改 `residential-monitor/src/main.ts` 与 `src/styles.css`，必须串行合入。

C5 继续拥有关于信息。C2 `AppFacade` 继续拥有 `put_setting` 外观键。前端只缓存 DTO 与视图选择。不授予 opener / fs。不注册 updater。

## 关于页

```text
进入 about 分区
    → 若会话缓存为空或上次失败需重试
        → invoke get_about → decodeAbout
    → 成功：定义列表
    → 失败：错误行 + 刷新
刷新按钮：强制再拉一次
```

加载标志只留当前会话，不写入设置。`route === "settings-data"` 的 skip-paint 规则保持：`connectionDelta` 等不整页重绘；关于加载完成需要一次 `paint`，与字体列表相同。

### 展示模型

定义列表（宽屏两列，窄屏一列），每行 `dt` + `dd`：

| 来源 | 行 |
|---|---|
| AboutDto | 产品名、版本、可执行文件、identifier、AUMID、签名、自动更新、Windows Service、Releases URL |
| i18n 静态 | 许可证 MIT、平台 Windows 11 NSIS current-user、数据只留本机且无遥测 |

签名行继续使用 `signatureNoteZh`。英文界面外层标签走 i18n，说明正文保持后端中文字符串（现契约）。`signed === true` 仍解码失败，界面停在错误态。

Releases URL 放在 `dd` 的 `.mono-value` 中，用户可选择复制。`#open-releases` 改为把该节点滚入视口并选中文本，或仅作为已展示地址的重复入口；返回值不得写入 `errorZh`。

末卡 `min-height: 100%` 保留。`.about-body` 用定义列表填满可用高度，顶部对齐，行间用现有卡片间距，不添加空装饰块。

## 侧栏宽度

```text
boot.uiSidebarWidth → parse → --shell-width
pointer/keyboard → clamp 160–352 → 改 .shell style
pointerup 且有变化 → save_ui_sidebar_width
cancel / lostcapture / blur → 回到 startWidth
```

| 键 | 合法值 | 默认 |
|---|---|---|
| `ui_sidebar_width` | 整数 CSS 像素 160–352 | 220 |

`BootstrapDto` 增加可选 camelCase `uiSidebarWidth`。缺字段或非有限数字回落 220。Rust 解析与 `UiFontSize` 同形：非法回落默认并仍 `put_setting` 规范化值。

`.shell` 改为 `width: var(--shell-width); flex: 0 0 var(--shell-width);`。`--shell-width` 设在 `html`。拖动中直接写 style，避免 `paint`。拖动标志对齐 `liveTableDragging`：为真时 `paint()` 直接 return。

手柄放在 `.shell` 右缘，不占导航点击区。键盘：ArrowLeft/Right 步进 8px，Home=160，End=352。松手或键调整结束才保存。

Recovery 无 `storage` 时 `save_ui_sidebar_width` 只改内存。设置二级导航保持现有 grid 列，不挂手柄。

## 共享约束

- 侧栏拖动与实时列宽拖动互斥：同一时刻只允许一个 dragging 标志。
- 动态 `paint` 后按 `id` 恢复焦点与 input 选区；侧栏宽度来自内存变量，不从 DOM 反推。
- 中英 i18n 键集合保持相等。
- 不把宽度或关于缓存写进控制器 JSON。

## 回滚

- 关于页可单独回滚到点击加载 + 三段段落，不影响宽度键。
- 宽度键可删除：`.shell` 回到 13.75rem，Bootstrap 忽略未知字段。
- `signed` 解码与删除部分失败断言必须始终保留。

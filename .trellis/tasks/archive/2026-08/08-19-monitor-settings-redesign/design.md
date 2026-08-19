# 技术设计：设置与数据管理界面

## 边界

保留 `settings-data` 顶级 route、现有 Tauri commands、DTO、四套主题和危险操作语义；只重组 `residential-monitor/src/main.ts` / 新的纯视图模块 / `styles.css` / i18n。不开新 UI 框架、不新增后端能力、不把 SkillPort 专属能力带入产品。

## 视图模型

```ts
type SettingsSection = "appearance" | "connection" | "data" | "about" | "danger";
interface SettingsDraft {
  section: SettingsSection;
  address: string;
  targets: string;
  locale: UiLocale;
  theme: UiTheme;
}
```

`SettingsDraft` 是当前 WebView 会话的视图状态，不写 controller JSON；`settingsSecret` 与 `settingsSecretVisible` 继续由 `main.ts` / `applySecretField` 管理，任何 render string 不含 secret。所有 command 输入在 click 时从 draft / DOM 读取，Rust 继续是校验权威。

## 信息架构

```text
settings-data
├─ 外观与语言     主题卡片、中文/English 分段
├─ 连接与监控     权威状态、地址、secret、重点目标、保存/测试/重连/断开
├─ 数据与备份     日志、备份恢复、保留/汇总、VACUUM
├─ 关于           版本、identifier、签名状态、固定 Releases
└─ 危险区域       删除预览、固定确认短语、分项结果
```

二级导航使用按钮 / 链接语义和 `aria-current`，不是伪造 `tablist`；内容区一次只渲染一个 section，避免整页长滚动。连接 section 默认激活，首屏优先展示 `health.session` 与 `collectorRunning` 的权威组合。

## 交互与安全

- section 切换先同步未提交 address / targets 到 draft，再 paint；paint 后按 id 恢复 focus、secret 显隐和 draft 字段。
- `save-settings` 保持现有 `save_settings`、`save_targets`、`save_ui_locale`、`save_ui_theme` 调用顺序与 session_only=false；保存成功后刷新 bootstrap。
- `test-controller` 明示“测试单帧连接”，成功只更新 probe 与 bootstrap，不把前端状态写成持续监控；`reconnect-controller` 调现有 `reconnect_now`。
- 关于、日志目录、备份、恢复、retention、VACUUM、删除按钮复用现有用途和结果 DTO；Recovery 分支继续只渲染 recovery safe actions。
- secret 字段保持 password 默认，显示 / 隐藏使用独立按钮；动态重绘不得通过 innerHTML 插 secret。

## CSS 结构

- `.settings-layout`：桌面 `grid-template-columns: minmax(11rem, 15rem) minmax(0, 1fr)`，内容最大宽度受控但不牺牲数据密度。
- `.settings-nav`：与全局 sidebar 区分的中性 surface，当前项使用现有 accent；窄窗改为 `overflow-x:auto` 的横向列表或紧凑选择器。
- `.settings-card` / `.settings-row` / `.settings-danger`：使用现有主题变量、divider 和 concentric radius；危险区单独 border / warning 色。
- 表单网格在桌面两列，窄窗一列；按钮组可换行但不遮挡状态。所有 interactive 控件至少 40×40 px。
- `transition-property` 精确列出，按压 `scale(0.96)`；`prefers-reduced-motion` 禁用 transform；动态数字 `tabular-nums`。

## 验证与回滚

纯渲染 / section / draft / secret-safety tests 先行，再做 CSS。四主题抽查 Mocha / Latte，中文 / English 各一轮，Tauri WebView 1200×800 + narrow screenshot。回滚只恢复 settings markup / scoped CSS / section state，不动 Rust commands 或数据库。

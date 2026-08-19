# 实施计划：全局中英

## 启动前门禁

- [ ] 用户已批准父任务规划摘要。
- [ ] 已读 `.trellis/spec/residential-monitor/{backend,frontend}/index.md`。
- [ ] 不改 `identity::PRODUCT_NAME`、`DELETE_CONFIRM_PHRASE`、identifier、Channel `schemaVersion`。

## 执行顺序

### 1. Locale 存储与 DTO

- `UiLocale` + `put_setting("ui_locale")`。
- `BootstrapDto.uiLocale`；前端解码缺省 `zh`。
- 设置页中/英控件，保存后立即生效。

**Gate**：缺键、空串、非法值都回落 `zh`；保存 `en` 后重启仍为 `en`（单元测试用临时库）。

### 2. Rust 目录

- 托盘、错误句、通知标题、路由标题、窗口标题、导出 HTML `lang` / 标题。
- `message_zh` 字段填当前语言。

**Gate**：同一 `code` 在 `zh`/`en` 下句子不同；`code` 本身不变。

### 3. 前端目录

- 抽出 `main.ts` 铬文案。侧栏、五页、Recovery、口号、删除说明跟 `uiLocale`。
- 与 Rust 健康/错误 key 对齐测试。

**Gate**：`npm --prefix residential-monitor test`；切换后侧栏标题不是中英混排。

### 4. 托盘与通知

- 语言变更重建托盘。
- 新通知用当前语言；历史行不回写。

**Gate**：菜单 id 不变，可见标签随语言变。

## 验证

- `npm --prefix residential-monitor run typecheck && npm --prefix residential-monitor run lint && npm --prefix residential-monitor test && npm --prefix residential-monitor run build`
- 针对 i18n / settings 的 `cargo test`
- 不跑 `tinstall`

## 回滚

删除 `ui_locale` 键与目录模块；DTO 去掉 `uiLocale`；托盘恢复写死中文。

## 产品文档

`PRODUCT.md` 能力列表用中文加一条：设置可切换界面中/英。不把该文件改成英文。

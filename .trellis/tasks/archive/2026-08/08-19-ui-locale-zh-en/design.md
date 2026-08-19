# 技术设计：全局中英

## 边界

```text
设置页选择 zh|en
        │
        ▼
put_setting("ui_locale", "zh"|"en")
        │
        ▼
AppFacade 持有 UiLocale
        ├─ build_tray / set_title
        ├─ AppErrorDto / ProbeResult / 通知 payload
        ├─ default_routes 标题
        └─ BootstrapDto.uiLocale + settings
                │
                ▼
前端 i18n 表渲染五页与 Recovery
```

- 安装包 `tauri.conf.json` `productName` 与 `identity::PRODUCT_NAME` 保持「家宽流量监控」。
- 窗口标题、侧栏品牌、托盘提示用显示名：中文「家宽流量监控」，英文 `Residential Traffic Monitor`。
- 口号：中文「观测下界，不是账单。」英文 `Observed lower bound, not a bill.`
- `DELETE_CONFIRM_PHRASE` 仍是 `删除全部本地数据`。

## 持久化

现有 `StorageCoordinator::put_setting` / `get_setting`。键 `ui_locale`，值 `zh` 或 `en`。启动时读取；非法值当 `zh`。恢复模式没有业务库时回落 `zh`，不阻塞 Recovery Shell。

新增 `save_ui_locale(locale)` 或并入现有设置保存。不要把语言塞进控制器 JSON，以免和地址/凭据补偿缠在一起。

`BootstrapDto` 增加 `uiLocale: "zh" | "en"`。前端解码缺字段时当 `zh`。

## 文案目录

Rust 新增小模块（建议 `c2/i18n.rs` 或 `i18n.rs`）：

- `UiLocale { Zh, En }`
- `t(locale, key) -> &'static str`
- 覆盖：托盘五项、产品显示名、口号、健康/会话错误句与下一步、通知标题、HTML 导出标题、`lang` 属性

前端新增 `src/i18n/zh.ts`、`src/i18n/en.ts` 与 `t(locale, key)`。`main.ts` 不再直接写死页面铬文案。健康码与 Rust 使用同一组 key（如 `health.connected`），测试断言两边 key 集合一致。

`AppErrorDto.message_zh` / `title_zh` / `note_zh` 字段名保持不变（现有解码依赖），内容改为当前语言。字段名不表示语言。

## 切换

1. 写入设置。
2. 更新内存 `UiLocale`。
3. `window.set_title`。
4. 重建托盘菜单。
5. 前端重拉 bootstrap 或只更新本地 locale 后 `paint()`。

历史 `alert_event` / outbox 行不回写。新通知按新语言生成。

## 删除确认

C5 `confirm_delete` 仍比较 `DELETE_CONFIRM_PHRASE`。英文页标签说明必须输入该中文句。输入框不预填。

## 风险

- `message_zh` 在英文下仍叫 `messageZh`，解码测试只查非空字符串，不查中文。
- 托盘重建必须在主线程；失败时保持旧菜单并留下一条不带 secret 的错误。
- 导出 HTML 的 `lang` 随当前语言；旧文件不改。

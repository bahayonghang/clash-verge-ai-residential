# 全局中英语言与系统文案

## Goal

用户在设置里选择中文或英文后，窗口、侧栏、五页、Recovery、托盘菜单、系统通知和后端错误立即使用该语言，重启后仍保持。

## Background

父任务：`08-19-live-clash-columns`。本子任务先于 `08-19-live-table-filter`。

仓库无 locale。`identity::PRODUCT_NAME`、`default_routes().title_zh`、`HEALTH_ZH`、`AppErrorDto.message_zh`、通知 `title_zh`、HTML 导出 `lang="zh-CN"` 均为中文。C5 删除确认短语为 `删除全部本地数据`。

## Requirements

- 设置页提供中 / 英两项。默认中文。值写入本机设置（`put_setting`，与控制器设置同寿命）。非法或缺失值回落中文。
- 切换后立即刷新 WebView、窗口标题、托盘菜单。此后发出的系统通知和命令错误使用新语言。已发出的历史告警标题不回写。
- 英文显示名：`Residential Traffic Monitor`。英文口号：`Observed lower bound, not a bill.`
- 删除确认短语在两种语言下都是 `删除全部本地数据`。英文界面必须说明要输入这句中文。
- 后端按已保存语言填充对用户可见的句子。稳定 `code`、identifier、AUMID、凭据 target、`identity::PRODUCT_NAME` 不翻译。
- 不引入 UI 框架或远程语言包。文档与代码注释仍中文。
- 向后续子任务提供语言读取：bootstrap / 设置 DTO 带 `uiLocale`（`zh` | `en`）。实时表头由后续子任务落地。

## Out of Scope

- Clash 列集合、家宽筛选、自定义筛选行。
- 第三种语言；跟随 Windows UI 语言自动切换。
- 把 `PRODUCT.md` 正文、`docs/`、安装包 `productName` 改成英文。
- 回写历史告警或旧导出文件。

## Acceptance Criteria

- [ ] 设置可在中/英之间切换，值写入本机设置；缺省与非法值回落中文。
- [ ] 切换后侧栏、当前页、Recovery、窗口标题与语言一致。
- [ ] 托盘菜单项与新发出的系统通知标题/正文与语言一致。
- [ ] 后端错误对用户可见的句子与语言一致；错误 `code` 不变。
- [ ] 重启后语言与退出前一致。
- [ ] 英文下删除本地数据仍要求输入 `删除全部本地数据`。
- [ ] secret 扫描为零；`npm --prefix residential-monitor` typecheck / lint / test / build 与相关 Rust 测试通过。

## Key Decisions

- 覆盖范围 C。默认中文。显示名与口号按父任务 A。删除短语固定中文。

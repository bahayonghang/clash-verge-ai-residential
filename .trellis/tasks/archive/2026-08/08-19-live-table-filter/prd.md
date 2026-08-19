# 实时表 Clash 列与家宽筛选

## Goal

用户在「实时连接」看到与 Clash Verge Rev 相同的字段集合，默认只看家宽相关行，并能用字段+精确/包含条件缩小列表。

## Background

父任务：`08-19-live-clash-columns`。依赖 `08-19-ui-locale-zh-en` 已提供的 `uiLocale`：表头、筛选标签和空态必须跟全局语言走。

`renderLive()` 现为七列。`LiveConnectionView` 已有多数 Clash 字段，但速率/时间为 `None`，端口与入站类型未解析。`ConnectionFilter` 只有单字段包含。重点目标默认「家宽」与 `AI-家宽` 不能精确相等。

## Requirements

- 列与对照见父任务 R1。操作列保留单条关闭。
- 展示：`host:port`、`ip:port`、`rule(payload)`、`Tun(tcp)`、链路用 ` / ` 连接。缺字段「未知」/`Unknown`。累计 0 为 `0 B`。无前一帧时速率为未知。
- 解析 `sourcePort`、`destinationPort`、入站类型、`start`。有前后两帧字节差才填速率；差值为 0 时可显示 `0 B/s`。
- 「只看家宽」默认开：链路节点等于任一重点目标，或节点名包含「家宽」。针「家宽」不随语言翻译。
- 可添加筛选行：字段（主机/链路/规则/进程/来源/目标/类型）+ 精确/包含 + 文本。多条 AND，与「只看家宽」叠加。空值行忽略。只留当前会话。
- 查询走 `query_live_connections`。单条关闭与空态五类不变。

## Out of Scope

- 关闭全部；CLOSED 页；列宽拖动；表头排序；详情抽屉；虚拟化；翻页 UI。
- 语言基础设施（由 `08-19-ui-locale-zh-en` 交付）。
- 改报告、告警、备份、核算公式。

## Acceptance Criteria

- [ ] 表为 Clash 十二列 + 操作；表头随 `uiLocale` 切换。
- [ ] 默认只看家宽；`AI-家宽` / `家宽-SOCKS5` 在默认目标「家宽」下出现。
- [ ] 精确不匹配子串；包含匹配子串；多条 AND；与「只看家宽」同时生效。
- [ ] 无前一帧时速率为未知；缺时间/端口/入站显示未知。
- [ ] 筛选经 `query_live_connections`；单条关闭与空态回归通过。

## Key Decisions

- 筛选在 Rust 查询层执行。家宽针固定「家宽」。

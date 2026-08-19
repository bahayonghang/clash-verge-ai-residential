# 技术设计：实时表排序、数值条件与列布局

## 边界

```text
machine_setting["live_table_layout"]
        │ sanitize
        ▼
BootstrapDto.liveTableLayout
        │
        ▼
renderLive：colgroup + 显隐 + 滚动容器
        │
query_live_connections(liveQuery)
        │ filter: residentialOnly + clauses
        │ sort: sortField + descending
        ▼
query.rs：文本/数值条件 AND → 排序（未知在后）→ 第一页
        ▼
tbody
```

不改 Channel `schemaVersion`、核算、`LIST_PAGE_*`。C2 仍只经 `StorageCoordinator::put_setting` 写偏好。

## 列布局

键：`live_table_layout`。值 JSON，须小于 `SETTING_VALUE_MAX`。

```json
{ "widths": { "host": 180, "download": 88 }, "hidden": ["process"] }
```

列 id：`host` `download` `upload` `rateDownload` `rateUpload` `chain` `rule` `process` `duration` `source` `destination` `type`。`action` 不进 `widths` / `hidden`。

| 列 | 默认宽 | 最小宽 |
|---|---|---|
| host | 180 | 140 |
| download / upload | 88 | 72 |
| rateDownload / rateUpload | 104 | 80 |
| chain | 280 | 160 |
| rule | 220 | 160 |
| process | 180 | 100 |
| duration | 108 | 88 |
| source / destination | 160 | 120 |
| type | 120 | 80 |
| action | 76 | 76（不可改） |

上限 640。未知键丢弃。宽度夹到 [最小, 640]。`hidden` 去重；若会藏光全部数据列，丢掉将导致零数据列的那些项。非法 JSON、超长、Recovery 无库：回落默认全显示。

命令：`save_live_table_layout(layout) -> LiveTableLayout`。校验后 `put_setting`，更新内存，返回消毒后的对象。复制 `save_ui_theme`：不进控制器 JSON。

`BootstrapDto` 追加 `liveTableLayout`。前端缺字段当默认。

前端：

- `colgroup` + `table-layout: fixed`。去掉 `.live-table { display: block }`。`overflow` 只在 `.live-table-wrap`。
- `th`/`td`：`white-space: nowrap; overflow: hidden; text-overflow: ellipsis`。数字列右对齐。
- `th` 带 `data-col`。右缘 6px 拖动手柄。`pointerdown` 开始拖，`window` 上 `pointermove` 改当前宽，`pointerup` 调用保存。拖动中设标志，跳过 `paint()`。
- 工具条按钮打开列面板：十二个复选框 + 「恢复默认」。至少一列数据列勾选。
- `paint()` 保存/恢复 `.live-table-wrap` 的 `scrollTop` / `scrollLeft`。

## 排序

`sort_field` 取值：`identity`（默认）`host` `download` `upload` `rateDownload` `rateUpload` `chain` `rule` `process` `duration` `source` `destination` `type`。其它值当 `identity`。

比较分两段：有值在前，未知在后。有值段按 `descending` 升降。平局 `identity`。`download`/`upload` 的 0 是有值。`rate_*` / `duration_ms` 的 `None` 是未知。字符串 `None` 或空串是未知。

`chain` 键：`chains.join(" / ")`。`rule` 键：与展示相同的 `rule` 或 `rule(payload)`。`source` / `destination`：`ip:port`（无端口则 ip）。`type`：`inbound(network)`，缺一则用剩下那个。

keyset 游标必须用同一顺序。编码：有值 `0:{sort_key}`，未知 `1:`。降序只反转 `0:` 段；`1:` 仍在最后。改排序时 `cursor = null`。

前端：数据列 `th` 内可点区域设 `data-sort`。循环降序 → 升序 → `identity`。`aria-sort`：`descending` / `ascending` / `none`。拖动手柄 `stopPropagation`。`defaultLiveQuery()` 仍是 `identity`。delta 后用当前 `liveQuery` 查第一页。

`dto-and-decoding.md` 中「每次查 identity」改为「查当前 `liveQuery` 第一页，默认 identity」。

## 数值条件

`FilterClause` 仍是 `field` / `mode` / `value`。前端条件行另持 `unit`，invoke 时可带上；Rust 忽略未知字段。

数值 `field`：`download` `upload` `rateDownload` `rateUpload` `duration`。

数值 `mode`：`gt` `gte` `lt` `lte` `eq`。

`value`：十进制 `u64` 字符串，单位已是字节或毫秒。解析失败或空：忽略该行。

| 字段 | 行值 | 未知 |
|---|---|---|
| download / upload | `row.upload` / `download` | 无（0 可命中） |
| rateDownload / rateUpload | `rate_*` | `None` → 不命中 |
| duration | `duration_ms` | `None` → 不命中 |

前端换算：`Math.round(n * factor)`。`n` 空、非有限、小于 0：忽略该行。乘完 ≥ 2^64：忽略该行。

| 单位 | 因子 |
|---|---|
| B / B/s | 1 |
| KiB / KiB/s | 1024 |
| MiB / MiB/s | 1024² |
| GiB | 1024³ |
| 秒 | 1000 |
| 分钟 | 60_000 |
| 小时 | 3_600_000 |

新建数值条件默认：字段 `download`，`gte`，单位 KiB，值为空。字段从文本切到数值时：`gte` + 该字段默认单位，清空值。反向切回文本：`contains`，清空值。

文本字段只认 `exact`/`contains`。数值字段只认五个比较 `mode`。交叉组合忽略该行。现有「未知 mode 当 contains」对数值字段不再适用。

## 兼容

- 旧调用不传 `liveTableLayout`、不传数值 `field`：行为与现在相同。
- `residentialOnly` 缺省仍 false。前端默认仍 true。
- 不 bump Channel schema。

## 风险

- 1 Hz `paint()` 会拆掉正在拖的手柄：拖动期间禁止 `paint()`。
- 整页重绘会丢滚动：必须存容器滚动，不能只存 `document` 滚动。
- 未知排在两侧都最后：不能对 `sort_key` 整段 `reverse`。
- `SETTING_VALUE_MAX`：消毒后的 JSON 远小于 4096；超长当非法回落。
- 拖动手柄与排序按钮叠在同一 `th`：手柄独占右缘，点击列名才排序。

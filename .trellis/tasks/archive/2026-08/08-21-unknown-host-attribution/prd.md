# 未知主机归因与可检查

## Goal

主机排名把控制器已给出的域名、嗅探主机或目的 IP 显示为可检查的行。只在三者都缺失时保留 `__unknown__`。历史未知行提供规则 / 链路 / 网络组成，不伪造 IP。

## Background

父任务：`08-21-unknown-host-en-sidebar`。本机近 24 小时证据见父任务 `research/unknown-host-24h.md`。

Clash / mihomo 元数据含 `host`、`sniffHost`、`destinationIP`。`normalize_connection`（`controller.rs`）只取 `host`。`ensure_session_on`（`storage.rs`）首次插入后不更新 host。`RANK_RAW` 把空 host 收成 `__unknown__`。主机页对未知行 `onSelect` 直接返回。neko collector：`domain = metadata.host || metadata.sniffHost || ""`，流量另记 IP。

已确认：identity 优先级为 `host` → `sniffHost` → 目的 IP。

## Requirements

- 采集读取 `sniffHost`。解析后的主机 identity 写入现有 `connection_session.host` 与 `attr.host_id`，不新增列、不升 `SCHEMA_VERSION`。
- 后续快照出现更强 identity 时更新已写入的弱值：非空 `host` 覆盖一切；非 IP 的 `sniffHost` 覆盖已写入的 IP；空值不得覆盖已有值。
- 实时表主机列与排名使用同一回退。目的列仍显示 `destinationIP`。
- 可解析为 IP 的 identity 在排名表与条形图上可辨认为 IP，不加伪域名。
- `filters.host == "__unknown__"` 表示 `coalesce(s.host,'')=''`（raw）或 `h.dimension_id = 0`（精确层）。哨兵不得当作字典值绑定。
- 主机页在 `crossDimension` 下允许选中未知行，下钻到规则 / 链路 / 进程；检查不改父级 grouping。其它维度的未知行仍不可下钻。
- 不把历史 NULL host 改写成没存过的 IP。
- 条形图 Y 轴不得从左侧截断长 FQDN（当前 `RankBar` `YAxis width={120}`）。
- 精确层 `crossDimension: false` 时不画假下钻入口。

## Out of scope

- 独立 IP 页或 GeoIP。
- 回填历史未知行的目的 IP。
- 英文侧栏排版（`08-21-en-sidebar-layout`）。
- 修复 Clash 不报进程名。
- 重跑已物化的小时 / 日层未知桶。

## Acceptance Criteria

- [ ] AC1：空 `host` + 非空 `sniffHost` 或 `destinationIP` 的 fixture，raw 主机排名出现该 identity，而不是 `__unknown__`。
- [ ] AC2：三者皆空时 identity 仍为 `__unknown__`，label 为「未知」/ `Unknown`。非主机维未知行仍不可下钻。
- [ ] AC3：已存在的空 host 会话在后续快照拿到非空 `host` 后，该会话后续与已写入分钟都计入该 host。
- [ ] AC4：主机页选中 `__unknown__` 后，规则 / 链路 / 进程下钻结果只含空 host 会话；不出现假 IP 列表。
- [ ] AC5：条形图轴标签与悬停显示完整 identity；长域名不被左侧裁切。IP identity 有可辨认标记。
- [ ] AC6：`just monitor-check` 通过。Rust 覆盖 sniffHost 解析、identity 回退、host 升级、`__unknown__` 过滤。

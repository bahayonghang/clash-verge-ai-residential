# 设计：未知主机归因与可检查

## 1. Identity 单一函数

新模块 `residential-monitor/src-tauri/src/session_host.rs`（采集边界，不属于 C3 查询）：

```rust
pub fn resolve_host_identity(
    host: Option<&str>,
    sniff_host: Option<&str>,
    destination_ip: Option<&str>,
) -> Option<String>;

pub fn prefer_host_identity(stored: Option<&str>, incoming: Option<&str>) -> Option<String>;

pub fn looks_like_ip(value: &str) -> bool;
```

优先级：非空 `host` > 非空 `sniff_host` > 非空 `destination_ip`。`looks_like_ip` 只用于展示标记和升级（`sniff_host` 为域名时覆盖已写入的 IP）。`IpAddr::parse` 识别 v4/v6；带括号的 v6 先去括号。

`ConnectionMeta` 增加 `sniff_host: Option<String>`，`normalize_connection` 读 `sniffHost`。`project_live` 把 `LiveConnectionView.host` 写成 `resolve_host_identity` 的结果；`destination_ip` 字段不变。

## 2. 写入：不改 schema

不新增列、不升 `SCHEMA_VERSION`。解析后的 identity 写入现有 `connection_session.host`，并由 `intern_and_attr` intern 为 `host_id`。

`ensure_session_on`：会话已存在时，用 `prefer_host_identity(stored, incoming)`；incoming 更强则 `update connection_session set host = ?`。`intern_and_attr` 已有 `on conflict do update host_id`，保持。分钟事实路径仍可能以空 host 建会话；同一 snapshot 先写 live_rows，后写 facts，顺序不变。

历史 NULL 行保持未知。已物化的小时 / 日未知桶不重跑。

## 3. 查询：`__unknown__` 过滤

`filter_clause`：`filters.host == "__unknown__"` 时片段为 `and coalesce(s.host,'') = ''`，不绑定哨兵。其它 host 值仍 `and s.host = ?`。

`append_dim_identity` 对 host 哨兵：`and h.dimension_id = 0`，不查 `dimension_dict.value = '__unknown__'`。精确层 `crossDimension` 为 false，主机页不会走这条下钻；仍写出以免其它调用误绑哨兵。

`RANK_RAW` 的 `case when coalesce(s.host,'') = '' then '__unknown__'` 不变。新写入的 IP / sniff 直接出现在 `s.host`。

## 4. 前端

- `filtersForDrilldown('host', '__unknown__')` 设置 `host: '__unknown__'`。其它维的未知 identity 仍返回空过滤（不可下钻）。
- 主机页允许选中未知行。`RankTable`：`kind === 'host' && crossDimension` 时未知行显示下钻按钮；其它维保持「未知行不能下钻」。
- `dimension-page` 去掉对未知 identity 的 `onSelect` 早退；改为 `filtersForDrilldown` 决定能否查询。
- 实时 `displayLiveRow`：`joinHostPort(row.host ?? row.destinationIp, port)`，与后端回退双保险。
- IP 标记：`looksLikeIp` 放 `format/rank.ts`（或小函数），排名表名称列与条形图 tick 在 IP 旁加 muted `IP` 文本，不加伪域名。
- `RankBar` Y 轴：按标签估算宽度（下限 96、上限 220），`textAnchor="end"`；tick 超出时对中间省略并保留 `title`。禁止固定 `width={120}` 导致左侧裁切。

## 5. 兼容与回滚

- 旧库无需 migration。新会话从下一帧开始按优先级写入。
- 回滚：恢复采集与 `ensure_session_on` / `filter_clause` / 前端选中逻辑。已写入的 IP identity 会留在 `s.host`；这是观测值，不是伪默认。
- `dimension_dict.value` 仍不得写入 `__unknown__`（`intern_dim` 已拒绝）。

## 6. 取舍

把 IP 写入 `host` 列，主机图域名与 IP 混排。相对新列 + schema 5，避免 backup/restore 与 `user_version` 面。独立 IP 页不在范围。

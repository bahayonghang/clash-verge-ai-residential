# 未知主机归因与英文侧栏排版

## Goal

主机页把无域名会话显示为嗅探主机或目的 IP；只有三者都缺时才是未知，且未知行可检查组成。英文侧栏左上产品锁与左下设置项按双语宽度排版。

## User value

用户能读无 SNI 流量的目的，而不是 30% 份额的黑箱。切换英文后侧栏仍可扫读。

## Task map

| 子任务 | 交付 |
|---|---|
| `08-21-unknown-host-attribution` | 空 host 的采集、写入、排名、未知组成检查、条形图轴标签 |
| `08-21-en-sidebar-layout` | 英文侧栏品牌区与底栏排版 |

父任务只做需求集、跨子任务验收与收口。实现落在子任务。

## Confirmed facts

截图主机页「近 24 小时」第 1 行与本机库同一行：未知，上行 853.8 MiB，下行 3.55 GiB，会话 20692，份额 30.9%。组成见 `research/unknown-host-24h.md`。字节主导是 `IPCIDR` + `DIRECT`；连接数主导是无规则的机场 Proxy。`connection_session.host` 与 `attr.host_id` 均为 NULL。`destinationIP` 只在实时 DTO；采集不读 `sniffHost`。`RANK_RAW` 把空 host 映射为 `__unknown__`。前端对未知行禁止下钻。

英文截图侧栏宽 220px。`Residential Traffic Monitor`、`Live connections`、`Settings / data` 在 `text-xl` / 无 `nowrap` 下按单词被挤断。中文「家宽流量监控」在同宽度单行。

## Decision

空 host 的主机 identity：`metadata.host` → `sniffHost` → 目的 IP。三者都空才是 `__unknown__`。IP 行是主机维一等 identity，可下钻；展示按 IP，不加伪域名。历史 NULL host 保持未知，可检查规则 / 链路 / 网络组成，不回填没存过的 IP。

## Requirements

- 新写入使用上面的 identity 优先级。不得把未知画成 0。
- 历史未知行留在排名中，份额分母为 totals。
- 英文 220px 侧栏品牌区与底栏无单词中断换行。中文侧栏保持可扫读。官方显示名字符串不改。
- 子任务独立可验收。

## Out of scope

- 进程名全局缺失。
- GeoIP / ASN / Regions / 独立 IP 页。
- 开发态 `data_dir` 迁出临时目录。
- 改安装产品名、窗口标题绑定名、删除确认短语。

## Acceptance Criteria

- [ ] AC-P1：新采集的无域名、有 `sniffHost` 或目的 IP 的会话不再并进 `__unknown__`。
- [ ] AC-P2：历史 `__unknown__` 行仍在排名中；可检查组成；不把缺失 IP 填成假值。
- [ ] AC-P3：英文 220px 侧栏品牌区与底栏无单词中断换行；中文侧栏无回归。
- [ ] AC-P4：`just monitor-check` 与 secret 扫描通过。

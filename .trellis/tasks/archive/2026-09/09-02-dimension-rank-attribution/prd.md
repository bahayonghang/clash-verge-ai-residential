# 维度排名表归属列与可调列宽

## Goal

在主机 / 规则 / 进程排名表直接看出每行主出口（含 `DIRECT`），混出口标「混合」；四页排名表改到 DataTable 规格并允许拖列宽。用户不必先下钻才能判断直连还是代理。

## Background

2026-09-02 用户在主机页看到 `dl-pc-zb.drive.quark.cn` 占近 24 小时下载 25.9%，询问是否走了代理。本机库该 host 60 条连接全部 `chain_key=DIRECT`。用户要求给该表加「归属」（含 Direct）、优化样式、可改列宽。

## Requirements

- R1 主机 / 规则 / 进程的 `RankTable` 在份额与下钻之间增加「归属」列。链路页不渲染该列。取值由后端给出：该行当前查询窗内、非空 `chain_key` 中下载字节最多者的原文；存在至少两条不同非空 `chain_key` 时 `exitMixed=true`，前端在主出口后标「混合」。无非空 `chain_key`、或报告不在 Raw 层时主出口为未知，不得写成 `DIRECT` 或 0。排名行不按出口拆行。下钻仍可用。归属列不可排序。
- R2 四页 `RankTable` 走 `data-table.tsx`：表头/正文字号档差、数值右对齐 + `tabular-nums`、行 hover、不裸 `w-full`。`table-layout: fixed` + `colgroup` 像素宽，超出横向滚。保留 `aria-sort`、默认下行降序图标、`data-identity` / `data-unknown` / `data-kind` / `data-drill`。
- R3 数据列（名称、上行、下行、连接、份额、归属）可拖宽；排名序号列与下钻列不拖、不写入设置。pointercancel / 失焦回滚，松手成功才 `save_dimension_rank_table_layout`。四页共用一套宽度。非法或缺失回落默认。Recovery 无库只留内存。不复用 `live_table_layout`。
- R4 中英文 i18n。缺口/未知不得画成零。禁止远程 URL / CDN。旧 `report_archive` JSON 缺新字段时按未知解码，不崩回看。

## Acceptance Criteria

- [ ] AC1 仅 `DIRECT` 的主机行归属为 `DIRECT`，无「混合」。
- [ ] AC2 同一 identity 存在至少两条不同非空 `chain_key` 时，归属为主出口原文 +「混合」。
- [ ] AC3 无非空 `chain_key` 时归属为「未知」，不是 `DIRECT`。
- [ ] AC4 主机 / 规则 / 进程表有「归属」列（下钻左侧）；链路表无该列。
- [ ] AC5 `24h`（Raw / `minute10`）有出口数据；`7d`/`30d`（Hourly/Daily 物化、无 `chain_key`）归属为「未知」，表格仍显示。
- [ ] AC6 拖宽松手后重启应用，四页数据列宽仍是拖过的值；下钻列宽不变。
- [ ] AC7 数值列右对齐 + `tabular-nums`，行有 hover，表不 `w-full` 拉满。`just monitor-check` 通过。

## Out of scope

- 改核算口径、重点目标、公开模板域名、`*.local.toml`。
- 实时表列宽/显隐（已有 `live_table_layout`）。
- 分析报告 `RankingTable`、家宽聚合排名表、导出 HTML/CSV 增列。
- 列重排、列显隐、虚拟化、归属列排序。
- 把 `chain_key` 写入 hourly/daily 物化。
- 三类桶（直连/家宽/机场）或单元格列出全部链路。
- 打开 C3 自动 DELETE。

## Key decisions

- 归属 = 主出口 + 混合。主出口 = 非空 `chain_key` 中下载最多的原文。平局：`chain_key` 升序。空 `chain_key` 不计入出口、不单独标混合。
- 列出现在主机 / 规则 / 进程；链路页不显示。样式与拖宽仍作用于四页共用表。
- 列序：…份额、归属、下钻。下钻为操作列，最右，不拖宽。
- 列宽写入独立 `machine_setting` 键 `dimension_rank_table_layout`，四页共用，不进控制器 JSON。
- Raw 层才计算出口；HourlyDimension / DailyDimension / DailyCore 主出口为空、混合为 false。

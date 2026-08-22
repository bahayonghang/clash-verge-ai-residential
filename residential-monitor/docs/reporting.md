# 报告口径

- 应用内图表、数据表和 CSV / JSON / HTML 使用同一个 `ReportResult`。图表是 Recharts 封装；占比环图与趋势图旁保留同口径数据表。悬停或钉住高亮只读当前结果，不改 grouping、不自动重查。
- `report_snapshot_token` 返回前关闭 SQLite 读事务。
- 空区间总量可以为 0。缺口、未知和能力不支持不得写成 0。
- 30 天 raw 支持组合过滤和下钻。13 个月精确层只支持单维。更老的 core daily 只保留总量、历史主分类和 coverage。
- `granularity` 合法值为 `minute1` / `minute2` / `minute5` / `minute10` / `hour` / `day` / `month`。分钟档只在 raw 保留期内可用，不升粒度。
- 主机 identity 优先级为 `metadata.host` → `sniffHost` → 目的 IP，写入 `connection_session.host`。三者都空时排名 `identity` 为 `__unknown__`；前端按维度显示「未归因主机」，不会把它与连接中、覆盖 gap 或未报告进程混为一谈。
- `filters.host` 为 `__unknown__` 时匹配空 host，不把哨兵当域名绑定。主机页可对未知行下钻到规则 / 链路 / 进程。
- `filters.process` 为 `__unknown__` 时匹配空进程 identity。进程页可对未知行下钻到主机 / 链路。规则与链路维的未知行不参与下钻。
- `filters.category` 为 `__residential__` 时匹配核算口径（`primary_category_id` 非空）。进程页「仅核算口径」开关使用该哨兵，不是某个重点目标名称。
- 自动 DELETE 保持关闭，直到守恒门通过。

## Unknown 与维度归因

- 实时卡片的空值表示 `observationPhase` 尚未到 `current`，例如未配置、连接中、差分基线待建立、暂停、断连、重同步或解码失败；它不是历史排名中的缺失 identity。只有 `current` 阶段的 `0` 才是真实零。
- 报告的 coverage 只描述时间覆盖 / gap；`attributionQuality` 独立描述当前 grouping 的字段覆盖。后端精确返回 known/missing upload、download、connections，并保证 known + missing 等于 totals。Top N 不参与该计算。
- 排名 `identity="__unknown__"` 的字节仍完整保留。Host 显示「未归因主机」，Chain 显示「未报告链路」，Process 显示「控制器未报告进程」，Rule 显示「未保存或未报告规则」。
- 同一 controller generation 内，后续非空 metadata 可补全先前 session；空白帧不会擦除已知 Host / Process / Rule / Network / Chains。Process 缺失时只允许使用同帧或同 generation 已知 `processPath` 的 Windows / Unix basename；完整路径不进入历史字典、质量 DTO 或日志。
- Chain identity 与 Rule group 分开：Chain 的单跳 `DIRECT` 保留为 `DIRECT`，多跳取末个非空 hop；Rule 对单跳链仍回退 raw rule。旧 hourly/daily Chain 只在仍有完整 raw 旁证的既有派生窗口内事务性重建；若重建前派生总量与 raw 总量不等则回滚并保留旧层，raw 已删除区间与 frozen archive 不改写。
- 历史 Host / Process 缺失若没有当时 raw 旁证不可恢复，不用当前活动连接、DNS 或 Clash 页面猜测。Overview 将顶部标为「实时 · 当前控制器」，趋势和 Top 标为「历史 · 已存储数据 · 时间窗」，两者可同时处于 connecting 与 ready。

## 自动小时 / 日档案

- 采集节拍在 durable commit 之后最多生成 1 份默认报告。窗口是已闭合的本地小时或已闭合的本地自然日。
- 默认查询：`displayTimezone=local`，`grouping=host`，`targetPolicy=historical`，`topN=20`，`comparison.previousEqualWindow=true`。小时 `granularity=hour`，日 `granularity=day`。
- 成功结果写入 SQLite 表 `report_archive`，进程退出后仍可 `list_report_archives` / `get_report_archive`。首次成功即冻结；已有 `ok` 不覆盖。`failed` 可在后续节拍重试。
- 小时档案保留 30 天，日档案保留 13 个月（`DIMENSION_RETAIN_DAYS`，396 天）。过期删除只针对档案表，与 raw 自动 DELETE 无关。
- 近 30 天默认走 raw，不在每个整点跑全量 `RetentionService`。更早的日档案走日维；日维未就绪则记失败，不写假总量。
- 进入分析报告页加载最新成功日档案，否则最新成功小时档案。不自动选手动行。
- 分析报告「运行报告」、告警跳转与家宽「生成报告」在查询成功后写入 `report_archive`（`kind=manual`），按 `generated_utc` 保留 7 天。同一 `(kind, range_start_utc, query_fingerprint)` 再跑则覆盖。失败查询不写行。不覆盖自动小时 / 日档案。
- 概览、四聚合页与家宽聚合的 `useReport` 现查只进 10 分钟 spool token，不写 `report_archive`。spool 对未过期 fingerprint 复用 token；满 8 格或超 128 MiB 时按最近访问淘汰。单 token 超过 32 MiB 仍拒绝。
- 从档案导出时，`get_report_archive` 把冻结 JSON 水合进现有 snapshot token，再走 `export_report`。不为导出再查更新后的库。
- Recovery Shell 不调度自动档案。

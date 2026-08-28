# 家宽地址流量归因与家宽页聚合缺陷分析

日期：2026-08-25。窗口：近 24 小时（utc_minute 29,792,310–29,793,750）。

## 数据来源

| 来源 | 内容 |
|---|---|
| Mihomo controller `/connections` | 实时快照 150 条活跃连接（verge-mihomo v1.19.29，127.0.0.1:9097） |
| `netstat -ano` + PID 反查 | 家宽 IP 连接的发起进程 |
| 监控库 SQL | `%TEMP%\io.github.bahayonghang.residential-monitor\monitor.sqlite3`（531 MB） |
| 代码 | `residential-monitor/src-tauri`、`clash-verge-ai-residential.js` v5.11.0 |

## 发现 1：主机维 `89.42.81.110` 行是指纹浏览器直连家宽的隧道流量，不是 AI 流量

机制：主机 identity 回退链 `metadata.host → sniffHost → 目的 IP`（`session_host.rs:18-27`）。这些连接无域名，主机维显示裸 IP。

证据：

- 实时快照 38 条连接目的 = `89.42.81.110`，host / sniffHost 全空。
- 32 条 TCP:12324 由 **SunBrowser.exe** 发起（AdsPower 指纹浏览器内核，PID 65384，路径 `AppData\Roaming\adspower_global\cwd_global\chrome_149\SunBrowser.exe`，netstat 证实）。
- 24 小时库内该主机行共 25,529 条会话：25,040 条进程未报告（链路 `🇺🇸 US 09>Proxy`），489 条 `SunBrowser.exe`（链路 `🇺🇸 US 09>Proxy>Others`）。
- 体量：上行 5.26 GB / 下行 1.08 GB，合计 6.34 GB，占全流量（15.65 GB）40%。上行约为下行 5 倍。

结论：AdsPower profile 内配置了 `socks5://89.42.81.110:12324`（家宽机器上的代理端口）。真实目标域名加密在 SOCKS5 隧道内，Mihomo 层不可知，**域名级细分不可能**；进程级细分可行。该流量还经机场美国节点绕行回家宽（双重代理），同时消耗机场与家宽两侧带宽。

## 发现 2：AI-家宽 流量按服务细分（24 小时，合计 4.37 GB）

| 服务 | 上行 | 下行 | 合计 | 占比 | 主要主机（24h 会话数） |
|---|---|---|---|---|---|
| Cursor | 2.20 GB | 75 MB | 2.27 GB | 52% | `api2.cursor.sh`（2,832 条，上行 2.18 GB，代码上下文上传）、Cloud Agent VM `*.us8/us10.cursorvm.com`（5,630 条）、`api5/api3/api4.cursor.sh` |
| Grok | 1.03 GB | 187 MB | 1.22 GB | 28% | `cli-chat-proxy.grok.com`（1,784 条，上行 1.00 GB，推理 API + 代码库/会话上传）、`grok.com`（826 条） |
| Claude | 124 MB | 386 MB | 509 MB | 12% | `downloads.claude.ai` 371 MB 下行（安装/更新，`docs/routing-scope.md` 已知取舍）、`api.anthropic.com`（185 条，上行 120 MB）、`claude.ai` |
| ChatGPT | 10 MB | 291 MB | 301 MB | 7% | `chatgpt.com`（1,388 条）、`ab.chatgpt.com` A/B 65 MB 下行 |
| Antigravity/Vertex | 44 MB | 28 MB | 72 MB | 1.6% | `daily-cloudcode-pa.googleapis.com`（809 条） |
| AI DNS | — | — | 0.7 MB | <0.1% | `8.8.8.8` / `1.1.1.1` DoH 经家宽（脚本 `RESIDENTIAL_DOH` 设计） |

- 服务映射来自脚本 v5.11.0 默认激活域名清单；各服务合计与链路总量（3,404,483,764 + 967,945,472 B）闭合。
- UI 路径：链路页 → `AI-家宽` 行 → 下钻按主机（`format/rank.ts:112-123` 支持 chain→host；后端 `RANK_RAW_CHAIN` + chain 过滤为同一查询形态，已在本机同一数据库上以 SQL 实测）。SQL 级等价验证；未驱动原生 UI 点击。
- `openrouter.ai` 不在脚本清单，走机场 Others（实时 3 条，7.1 MiB），不占家宽。

## 发现 3：家宽页聚合「未知」由两个独立缺陷叠加

**缺陷 A（配置语义 + 产品引导缺失）**：`target_item` 配置为 `家宽`；核算口径要求链路节点**精确等于** target（`residential.rs:14-20`）。实际链路节点为 `家宽-SOCKS5` / `AI-家宽`，精确匹配零命中 → 全部 332,731 条会话 `primary_category_id` 为 NULL → 聚合统计、TARGET 排名、占比全部无数据。实时卡片走筛选口径（节点名含「家宽」即命中），所以实时段有数（3.1 MiB / 26 条）、聚合段全未知。

**缺陷 B（代码缺陷）**：断连期间每个输入帧写一条未闭合 gap 行（`accounting.rs:218-234`），现存 29,187 条 `coverage_interval` 行 `ended_utc=NULL`（时间跨度 1787071476–1787101208，约 8.3 小时，每秒一条，未合并未闭合）。覆盖计算逐条求和不去重（`c3/share.rs:116-129`、`c3/service.rs:614-628`）：29,187 条重叠 gap 使 `covered_sec=0` → `uncovered` → 家宽页「该区间无采集覆盖」+ 全部未知；概览「覆盖 partial，缺口 52,536,600 秒」= 29,187 × 1,800（30 分钟窗口内每条开放 gap 各计一个窗口长），数字精确吻合。

关系：**即使改对 target，缺陷 B 仍会让聚合区显示未知**（`share.rs:65` covered_sec==0 直接返回未知）。两个都必须处理，家宽页才能出数。这是「两个任务关联」的机制。

## 发现 4：修复路径

1. 用户侧立即可做：家宽页 target 从 `家宽` 改为 `家宽-SOCKS5`（精确节点名）。新会话开始命中；raw 保留期（30 天）内历史可用 `targetPolicy=current` 重算（家宽页「生成报告」current 模式）。
2. 代码侧（另立任务）：gap 行在断连时扩展既有开放行而非逐秒新增；`covered_sec` 按区间并集计算。
3. 数据目录缺陷：`lib.rs:307-309` 默认数据目录为 `%TEMP%\io.github.bahayonghang.residential-monitor`，531 MB 主库放在 Temp，系统清理会删除数据；`residential-monitor/docs/data-directory.md` 写的是 LocalAppData。代码与文档不一致，属发布缺陷，需迁移 + 改默认值。

## 发现 5：其他观察

- 概览 TOP 主机 `wetype.weixin.qq.com`（73.9 MiB，DIRECT，微信输入法）等与 AI 无关。
- 489 条 SunBrowser 会话有进程名、25,040 条没有，进程上报差异原因未查明；与 ADR-0003 controller-only 语义一致，不影响主结论。
- AdsPower 直连家宽的绕行问题可选解法：Clash 配置加 `IP-CIDR,89.42.81.110/32,DIRECT` 让其直连（省机场绕行）；是否采用属用户决策。

## 验证方式

实时：`curl /connections` + `netstat -ano` 反查 PID。历史：对监控库执行与后端同形 SQL（`RANK_RAW_CHAIN` / `SHARE_RESIDENTIAL_RAW` 同库同口径）。核算口径零命中：`connection_session_attr` 全表 `primary_category_id` 分布查询。缺口数字：29,187 × 1,800 = 52,536,600 与概览显示精确一致。

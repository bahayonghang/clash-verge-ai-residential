# 实时连接 Clash 列、家宽筛选与中英界面

## Goal

用户打开「实时连接」后，用与 Clash Verge Rev Connections 相同的字段集合读当前连接，默认先看家宽相关行，并可用精确/包含条件缩小列表。设置里可把整个产品切到中文或英文：WebView、托盘、系统通知和后端错误一起切换。缺字段仍显示「未知」，不把观测下界写成账单。

## Task Map

父任务只拥有需求源和跨子任务验收。实施从子任务开始，不在父任务上 `task.py start`。

| 子任务 | 交付 | 顺序 |
|---|---|---|
| `08-19-ui-locale-zh-en` | 全局语言、文案目录、托盘、通知、后端错误、导出 HTML `lang` | 先做 |
| `08-19-live-table-filter` | Clash 十二列 + 关闭列、补齐端口/入站/`start`/速率、只看家宽、字段+精确/包含 AND 筛选 | 后做，消费已保存语言 |

## Background

- Clash Verge Rev Connections 表头：Host、Downloaded、Uploaded、DL Speed、UL Speed、Chains、Rule、Process、Time、Source、Destination、Type。家宽行 Chains 示例：`AI-家宽 / 家宽-SOCKS5`。
- 当前 `renderLive()` 只渲染域名、进程、主分类、上行、下行、网络、操作。`LiveConnectionView` 已有速率、链路、规则、时间、源/目的，但 `project_live` 把速率和时间写成 `None`。
- `normalize_connection` 丢掉控制器样本里已有的 `sourcePort`、`destinationPort`、`start` 与入站类型。
- `query_live_connections` 支持单字段包含。前端始终传空筛选。重点目标默认「家宽」，分类要求与 `chains` 精确相等，故 `AI-家宽` / `家宽-SOCKS5` 主分类为「未知」。
- 无 locale。`PRODUCT_NAME`、路由 `title_zh`、`HEALTH_ZH`、托盘、通知、`message_zh`、删除确认短语均为中文。
- 稳定标识不随语言变：`io.github.bahayonghang.residential-monitor`、AUMID、凭据 target、`identity::PRODUCT_NAME`（中文产品名常量）、`DELETE_CONFIRM_PHRASE`。

## Requirements

### R1 Clash 字段列

由 `08-19-live-table-filter` 交付。列与语言对照：

| 中文 | 英文 | 数据 |
|---|---|---|
| 主机 | Host | `host`，有端口则 `host:port` |
| 下载 | Downloaded | `download` |
| 上传 | Uploaded | `upload` |
| 下载速度 | DL Speed | `rateDownload` |
| 上传速度 | UL Speed | `rateUpload` |
| 链路 | Chains | `chains`，用 ` / ` 连接 |
| 规则 | Rule | `rule`，有 payload 则 `rule(payload)` |
| 进程 | Process | `processName` |
| 时间 | Time | 由 `start` 或 `durationMs` 得到的相对时间 |
| 来源 | Source | `sourceIp`，有端口则 `ip:port` |
| 目标 | Destination | `destinationIp`，有端口则 `ip:port` |
| 类型 | Type | 入站类型 + 网络，如 `Tun(tcp)` |
| 操作 | Action | 单条关闭，行为不变 |

缺字段显示「未知」/ `Unknown`。累计字节为 0 显示 `0 B`。尚无前后两帧时速率为未知，不得写成 `0 B/s`。已有前一帧且差值为 0 时可显示 `0 B/s`。

### R2 补齐控制器已有元数据

由 `08-19-live-table-filter` 交付。解析 `/connections` 已有的 `sourcePort`、`destinationPort`、入站类型、`start`。不把 mihomo 原始 JSON 交给视图。

### R3 家宽筛选与自定义条件

由 `08-19-live-table-filter` 交付。

- 「只看家宽」默认打开。命中：`chains` 任一节点等于已保存重点目标，或节点名包含「家宽」。匹配针「家宽」不随界面语言翻译。
- 可添加条件行：字段（主机 / 链路 / 规则 / 进程 / 来源 / 目标 / 类型）+ 精确 / 包含 + 文本。多条 AND，与「只看家宽」叠加。空值行忽略。只留当前会话。
- 筛选走 `query_live_connections`。空结果沿用「已连接且查询为空」空态。

### R4 全局中英

由 `08-19-ui-locale-zh-en` 交付。

- 设置页中 / 英切换，默认中文，写入本机设置，重启后保持。
- 立即刷新 WebView 五页与 Recovery、窗口标题、托盘菜单；此后发出的系统通知和后端错误使用新语言。已发出的历史告警标题不回写。
- 英文显示名：`Residential Traffic Monitor`。英文口号：`Observed lower bound, not a bill.`
- 删除确认短语在中英文界面都是 `删除全部本地数据`。英文设置页须说明必须输入这句中文。
- 不引入 UI 框架或远程语言包。文档与代码注释仍中文。

### R5 既有实时合同

单条关闭、`204` 只标已发送、空态五类、secret 不进 Channel / 日志 / 导出。不改核算公式。

## Out of Scope

- 关闭全部连接；Clash CLOSED 页；列宽拖动；表头点击排序。
- 连接详情抽屉、虚拟化、keyset 翻页 UI。
- 第三种语言；跟随 Windows 显示语言自动切换。
- 改 Clash / mihomo 配置；读取流量内容。
- 重跑 C2 10k × 30 分钟峰值。
- 更换产品视觉世界。
- 把 `PRODUCT.md` 正文、`docs/`、代码注释改成英文。安装包 `productName` 与 `identity::PRODUCT_NAME` 保持中文。

## Acceptance Criteria

- [ ] **AC1 列集合**：实时表为 Clash 十二列 + 操作；表头随语言切换；不再用「域名 / 主分类 / 上行 / 下行 / 网络」当主列。
- [ ] **AC2 缺值**：无前一帧时速率为未知；时间、进程、端口、入站类型缺失时为「未知」/ `Unknown`，不把未知画成零。
- [ ] **AC3 家宽筛选**：默认只显示家宽相关行；`AI-家宽` / `家宽-SOCKS5` 在默认目标「家宽」下命中；关闭开关后见全部当前页。
- [ ] **AC4 查询合同**：筛选经 `query_live_connections`；表格不以 Channel upsert 排序。
- [ ] **AC5 自定义条件**：精确不匹配子串（`chatgpt.com` 精确不能命中 `ws.chatgpt.com`）；包含匹配子串；多条 AND；与「只看家宽」同时生效。
- [ ] **AC6 语言**：设置切换后，五页、Recovery、窗口标题、托盘、新发出的系统通知和后端错误与所选语言一致；重启后仍为所选语言。英文下删除确认仍要求 `删除全部本地数据`。
- [ ] **AC7 回归**：空态、单条关闭、secret 扫描保持；`npm --prefix residential-monitor` 的 typecheck / lint / test / build 通过；相关 Rust 测试通过。

## Key Decisions

- 家宽判定：重点目标精确匹配，或链路节点包含「家宽」。默认打开「只看家宽」。
- 自定义筛选：字段 + 精确/包含 + 值，多条 AND。不做全文单框，不写表达式。
- 语言覆盖：WebView、托盘、系统通知、后端错误（C）。默认中文。
- 英文产品显示名 `Residential Traffic Monitor`，口号 `Observed lower bound, not a bill.`
- 删除确认短语固定中文。稳定 identifier 与安装产品名不随语言改。

## Notes

- 父任务保持 `planning`。用户批准本规划摘要前，任何子任务都不得 `task.py start`，也不得改产品代码。

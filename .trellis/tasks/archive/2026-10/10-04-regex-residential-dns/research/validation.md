# 正则住宅 DNS 验证

## 授权

2026-10-04，用户批准按“空组回退防护 → 正则住宅 DNS”的顺序实施。本记录先保存第二项的技术前置验证；未获技术验证前不修改该项产品行为。

## 前置能力 gate：通过

- 内核：已安装 Mihomo Meta v1.19.32 windows amd64，Go 1.26.8，2026-09-30 构建。
- 命令：`node .trellis/tasks/10-04-regex-residential-dns/research/check-core.cjs <installed-mihomo.exe>`。
- 4 种 Vertex/Cursor 布尔组合分别启动独立内核；每种 9 个主机，共 36 次真实 TXT DNS 查询。
- 每次查询均验证本地 resolver 实际接收了请求，避免把 fake-IP 或缓存响应计为成功。
- 正向主机：us-central1-aiplatform.googleapis.com、europe-west4-aiplatform.googleapis.com、repo42.cursor.sh，按对应开关命中。
- 6 个负向主机：maps.googleapis.com、fonts.googleapis.com、storage.googleapis.com、repofoo.cursor.sh、repo42.cursor.sh.example.test、foo.us-central1-aiplatform.googleapis.com，全部保持机场解析。
- 使用本地 inline/classical provider + nameserver-policy `rule-set:AI-家宽-DNS-REGEX` + DOMAIN-REGEX；4 份配置的 `-t` 均为 0。
- 住宅查询观察到两级本地 SOCKS5 CONNECT：Airport → Home endpoint、Home → residential DNS；普通查询为 Airport → airport DNS。
- 全部断言通过，退出码 0。证明该实际内核支持规划中的组合及解析代理选择机制。

## 隔离与结论范围

所有 DNS、SOCKS5 和 controller 服务仅监听 127.0.0.1 随机端口。虚构 SOCKS5 只允许访问本次 fixture 端口，拒绝公网目标。独立内核使用临时目录，结束后进程、socket 和临时目录全部关闭/清理。不读取用户 Profile、local 配置或生产 controller，不修改 TUN 或系统设置。

本验证采用 TCP DNS 与回环模拟代理，证明内核的正则匹配、policy 选择和链式拨号；不证明用户真实住宅服务商、公网 DoH TLS、UDP 转发或固定公网 IP 状态。正式生成输出将在实施后用同一探测器再次验证；仅替换 fixture resolver 地址并移除地理库依赖，保留生成的规则集、DNS policy 和 SOCKS 链路。

## 产品验证

### 隔离实施环境

实施子代理所在旧 main worktree 无法执行 Git 对齐。指定测试命令报告 13 通过、14 失败：回归文件缺少 `routing-default-v5.11.json`，另有旧 renderer 对现有开关不兼容。子代理的 `npm run check` 通过，新 renderer DNS 矩阵也通过。未把这些环境失败视为产品通过。

主会话只比较并应用七个授权文件相对当前主仓库的内容差异，没有应用旧 worktree 的整体 Git diff，没有改动 fixture 或 renderer 源码。

### 主仓库验证

- `node --test tests/regression.test.js tests/sync-local-config.test.js`：118/118 通过，0 失败。
- 最终 `just ci`：退出码 0，包含 monitor 与 root gate。
- 最终 `just docs-build`：退出码 0。
- 两个子任务的规范已同步；自动格式化产生的原有表格、示例 JSON 和无关句子变更已撤销。
- 任务上下文 manifest 校验通过，`git diff --check` 通过。

### 最终生成配置的实际内核验证

命令：`node .trellis/tasks/10-04-regex-residential-dns/research/check-core.cjs <installed-mihomo.exe> clash-verge-ai-residential.js`。

从当前公共脚本的 `main` 输出生成四种开关配置，保留实际 provider、policy、业务规则和两级 SOCKS 链路。为保证离线隔离，只替换 resolver 端点为回环 fixture，移除 geosite 依赖，设置独立 DNS/controller 监听和关闭 TUN。

四份配置均通过内核 `-t`；36 次 TXT 查询全部命中预期 resolver，并观察到预期 SOCKS CONNECT。三类正向主机按开关生效，六类负向主机保持机场解析；全关闭不留下住宅 regex policy。退出码 0。

此结果通过 D10 的隔离实际内核部分。没有测试用户生产 Profile、公网 DoH TLS、住宅服务商 UDP 或固定公网 IP，不将这些状态写成通过。

### 独立审查

首次独立审查发现 1 项 P2：fake-IP/sniffer 对专用 provider 的引用未进入冲突检查。主会话的只读内存实验确认 `dns.fake-ip-filter`、`sniffer.skip-domain`、`sniffer.force-domain` 在全关闭时会保留悬空引用。

已按既有 R3/AC4 修复：三个域名列表与非托管 nameserver-policy key 使用同一保留名称检查。测试覆盖开/关、旧 canonical provider、单名称/逗号列表、大小写关键字、输入不变，并证明相邻普通 provider 的引用仍保留。文档和 spec 已补齐字段枚举，未新增业务规则或文件范围。

修复后的 118 项专项测试、最终完整 `just ci`、`just docs-build` 和生成配置的 36 次内核查询均已重新通过。独立定向复核确认原 P2 已关闭，本次范围内未发现其他确定的引用遗漏。实施与验证完成，未提交、未归档。

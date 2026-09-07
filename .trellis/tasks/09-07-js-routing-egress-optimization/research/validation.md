# 证据与验收矩阵

## 实施验证（2026-09-07）

以下记录对应用户批准实施后的工作区；下方规划记录保留为历史证据。

| 检查 | 结果 |
|---|---|
| 三个相关 Node 套件 | PASSED，97/97 |
| `npm run ci` | PASSED，语法检查、125/125 Node 测试及模板安全扫描 |
| 实际 `just ci` | PASSED，退出码 0；包括 monitor-check 和根目录 npm run ci |
| monitor 前端 | PASSED，图标、类型、lint、69 个文件的 284/284 测试及生产构建 |
| Rust | PASSED，fmt、clippy `-D warnings`、432 个单元测试及 3 个隔离进程测试 |
| Rust ignored | 1 项本机 Credential Manager 写入测试保持忽略，未获现场授权；见 credential.rs 的 ignore 原因 |
| 文档站 | PASSED；初次缺少本地 vitepress，按已有锁文件执行 `npm --prefix docs ci` 后构建通过，无依赖声明或锁文件变化；独立修复后的最终构建亦通过 |
| Trellis 上下文 | PASSED，implement.jsonl 5 条、check.jsonl 3 条 |
| `git diff --check` | PASSED |

默认 fixture 在修改源码前由基线提交 `063c5561b9e81e42f89d7966a5d1a9772bdc44b3` 和虚构 Profile 生成。包含 64 条完整规则及 49 个 DNS policy 键，仅投影家宽节点 type/udp/dialer-proxy，不含 server/username/password。默认比较保留私网、CIDR、原规则以及 resolver 数组顺序，只对连续同目标的域名区段排序。

V1–V13 的自动化部分已通过：新增三类 core 开关切换并清理旧基线规则/DNS；四类进程兜底服从 core，Cursor 另受专属开关控制；Anthropic IP 双条件；独立辅助与全局开关保留；原规则/未知规则、输入所有权与幂等保留；renderer 真/假、缺省、类型/重复键和 24/12/12 审计映射通过。正则 DNS 例外保持，不新增宽泛 DNS 后缀。

真实 Clash/Mihomo 加载、业务命中、UDP、实际 DNS 查询、固定出口 IP 和流量收益继续为 **UNVERIFIED**。本轮未操作真实 Profile、凭据、controller 或 render-local；Node/配置结构不能证明运行时 fail-closed，宿主可能在脚本异常后保留输入 Profile。

独立 check 最终无剩余代码阻断，确认 V1–V13 自动化合同与实现一致，并使用 `git show` + VM 独立重建旧提交投影，与 fixture 原始规则顺序及 DNS resolver 数组完全一致。审查修复了英文配置表误嵌入的重复正文，以及 README 对 DNS 范围和完整 CI 的旧表述；文档结构、renderer 21 项测试及最终文档构建复验通过。AC1–AC7 已按上述证据勾选。

规范同步涉及 frontend 的 index、component-guidelines、state-management 与 quality-guidelines；component 文件中同一输入所有权误述一并更正，已同步设计与元数据。代码尚未提交，任务保持 in_progress，归档与日志按提交步骤执行。

## 规划阶段已执行（历史）

- 2026-09-07，基线 `063c5561b9e81e42f89d7966a5d1a9772bdc44b3`。
- 主线程执行 `node --test --test-reporter=dot tests/regression.test.js tests/sync-local-config.test.js tests/install-agent-skills.test.js`，退出码 0，三个相关套件通过。
- 只读审计代理执行 `tests/regression.test.js`，54/54 通过。
- 规划产物：真实上下文清单已建立；8 个任务文件的本地链接、元数据路径、占位符和尾部空白检查通过。`git status --short` 仅新增此任务目录；`task.py current` 为空，未激活任务。最终上下文校验结果见本文件末尾。
- 使用 `node:vm` 将公开脚本读入内存，仅替换开关常量；未写产品源码/本地脚本，得到以下输出。

| 探针 | 观察 |
|---|---|
| gemini_web_core=false、vertex_ai_endpoints=false | 仍有 cloudcode-pa、daily-cloudcode-pa、cloudaicompanion、generativelanguage.googleapis.com 和 antigravity.google 共 5 条家宽规则 |
| openai_core=false、ai_process_fallback=true | 仍有 ChatGPT/Codex/OpenAI 的 PROCESS-NAME-REGEX 与 PROCESS-PATH-REGEX |
| 默认 Vertex | 有区域 DOMAIN-REGEX；DNS 仅有 aiplatform.googleapis.com、aiplatform.us.rep.googleapis.com、aiplatform.eu.rep.googleapis.com 三个 exact 键 |

这些证据证明配置生成行为；没有测量真实网络、流量、地区或固定 IP。

补充：只读代理用虚构 Profile 对 main() 做内存合成探针，输出家宽组分别为：

```json
{"case":"use","group":{"name":"AI-家宽","type":"select","proxies":["家宽-SOCKS5"],"use":["provider-a"],"disable-udp":false}}
{"case":"include-all","group":{"name":"AI-家宽","type":"select","proxies":["家宽-SOCKS5"],"include-all":true,"exclude-type":"direct","default-selected":"HK","exclude-filter":"^家宽-SOCKS5$","disable-udp":false}}
```

说明：额外来源与 default-selected 被接受，include-all 加固还会给该组追加家宽排除表达式。依据官方组成员规则，配置可包含替代出口，尚未对真实宿主发起请求。规划沿用“非托管同名组拒绝”，额外允许 icon/hidden 展示字段，不静默删除用户额外配置。

## 实施行为矩阵

| ID | 输入与观察点 | 必须达到的结果 |
|---|---|---|
| V1 | 干净 Profile + 默认配置 | 匹配集合/目标、优先级层次、DNS policy、上游选择保持等价；允许同目标非重叠条目内部排序变化；不新增域名和出口 |
| V2 | 显式单成员家宽组另带 use/include-all* | 在保留名检查报可理解错误，输入不变；不能生成包含机场/动态 provider 的家宽组 |
| V3 | 已有家宽组的排除条件/空组替代出口 | 按 design.md 的托管形状约束拒绝；不能把“显式一个节点”当作唯一出口证明 |
| V4 | 三个新开关分别 true→false→true | 只切换其域名规则及 exact/suffix DNS；旧托管项清干净；其他服务不变 |
| V5 | anthropic_core=false + anthropic_ip_fallback=true | 不生成 Anthropic 核心规则、核心 DNS 或 CIDR 回退；再次开启恢复；关闭 IP 开关仍独立有效 |
| V6 | ai_process_fallback=true + 任一服务 core=false | 不生成该服务进程规则；其他启用服务仍保留；Cursor 还须 cursor_process_fallback=true |
| V7 | core=false + 独立辅助/认证开关=true | 独立条目仍可生成，文档不可声称整产品所有流量禁用；全局实时/DNS开关同理 |
| V8 | 私网域与地址 + 进程兜底 | 私网 DIRECT 优先；不捕获本地 MCP/LAN |
| V9 | 关闭 core 后的原规则 | 已有 MATCH/DIRECT/服务组规则按原序保留；未知用户 AI-家宽 规则仍保留，不保证必然机场 |
| V10 | 二次 main 与开关切换 | 无重复项，旧托管规则、DNS和进程/IP条目不残留；返回副本且错误时输入不变 |
| V11 | renderer 的三个 boolean | 接受 true/false，拒绝错误类型/重复键；示例、常量映射、缺失默认补全一致且保留用户已有值 |
| V12 | build-inputs | 新域名正确归属三个 supported 开关；routing=24、supported=12、unsupported=12；不把 IP/进程统计冒充域名统计 |
| V13 | 正则与 DNS 例外 | 原正则边界保持；不生成宽泛 Google/Cursor DNS 后缀，不声称已有同出口证明 |

最小单元测试不模拟整个 Mihomo 引擎。对关闭后的规则位置可使用现有真实数组、已有测试辅助器和明确 fixture；不可只断言常量或复制实现的表。

## V1 / AC4 的固定基线比较

实施前从 task.json.meta.baseline_commit 读取公开脚本（git show 只读），在 Node 中对同一个虚构 Profile 生成参考输出。保存 tests/fixtures/routing-default-v5.11.json，记录来源提交；测试不能依赖当前分支的动态 HEAD，也不能使用真实 local.js。fixture 只包含规则、DNS、家宽组、家宽节点 type/udp/dialer-proxy，不包含 server/username/password。

比较算法固定如下：

1. 两侧首先验证规则优先级层次；仅抽取连续的、目标为 AI-家宽 的 DOMAIN/DOMAIN-SUFFIX/DOMAIN-REGEX 核心域名区段。
2. 只对该区段的完整规则字符串排序。私网、IP、端口/DNS/进程及原 Profile 规则保持原始顺序，不能全量排序 rules 掩盖先后变化。
3. DNS 对象仅规范化对象键顺序，resolver 数组保留原顺序；不删除现有 policy/分支后再比较。
4. 规范化后 deepEqual 规则、DNS、家宽组及 type/udp/dialer-proxy。日志与版本字符串不在比较投影内。
5. 以现有各开关 fixture 补 V4–V10，默认比较不代替开关组合测试。测试 fixture 属于历史行为证据，不是运行时兼容层。

## 未验证与后续现场验收

- 规划阶段未运行完整 `just ci`；批准实施后的完整门禁结果见本文顶部，不能以规划阶段的定向检查代替。
- 真实 Clash Verge Rev/Mihomo 加载、UDP 路由继续匹配行为、DNS 真查询路径、实际固定出口、业务可用性、节省流量均为 UNVERIFIED。
- 后续宿主验收至少记录版本、启用开关、脱敏命中规则/链与业务成功/失败；若需要暂时改变路由或停用家宽进行故障验证，应取得该实际运行目标的授权。规划不预先授权生产切换。
- 在离线 fixture 中设置不可达家宽，并检查无出口替代只能证明配置结构；真实 fail-closed 仍需隔离环境运行证据。禁止将其表述为“已验证不会泄漏”。

## 最终规划校验

- `task.py validate`：implement.jsonl 5 条真实上下文、check.jsonl 3 条，通过。
- 本地链接、元数据路径、尾部空白、TBD占位符检查通过；`git diff --check` 通过。此时仅新增本任务 8 个文件，没有产品代码改动。
- 独立规划审查提出的四项 should-fix 已落实：补齐四份配置表；明确固定来源提交的默认投影 fixture 与规范化算法；统一完整变更清单并标记计划新文件；明确 CHANGELOG Unreleased 与版本/发布延后策略。具体实施结果仍待后续授权和验证。
- 独立审查最终结论：GO（仅规划就绪），阻断 0、应修 0。已人工逐子句核对 AC1–AC7 → R1–R5 → 设计机制。审查工具的标识符提取器未识别粗体 R/AC 定义，不能把自动结构检查代替上述人工追溯；该提示不构成实施授权。

# 技术设计

## 1. 边界与最小改动

继续使用单文件、零依赖 CommonJS 的可粘贴脚本，以及独立 Node TOML 渲染器。所有改动限于既有配置生成层。`AI-家宽` 与 `家宽-SOCKS5` 名称、链路结构、上游选择算法、默认 DNS 策略保持。

不新增通用路由对象 schema、服务注册表、出口枚举、多档模式或规则下载器。沿用常量数组、boolean 开关和现有 builder。参考架构的价值落实为可控服务边界与唯一出口约束。

## 2. R1：家宽组所有权和唯一出口

在 `validateReservedNameCollisions()` 的既有校验中收紧“脚本托管组”定义：

- 名称为 AI-家宽、type 为 select、proxies 恰好包含家宽-SOCKS5。
- 仅允许现有结构字段 `name/type/proxies/disable-udp` 和中性展示字段 `icon/hidden`。`disable-udp` 可缺省，成功构建仍统一 false。
- 发现其余自有字段则视为需要用户处理的同名自定义组，沿用现有碰撞错误路径，报告字段名及重命名/移除额外字段的处理方法。绝不输出凭据/整个配置。
- 因而 use、include-all、include-all-proxies、include-all-providers、filter、exclude-filter、exclude-type、default-selected、empty-fallback 等不能悄悄生效；url/interval 等自定义附加项同样明确拒绝，不尝试逐字段推测其安全性。

这是对既有保留名对象的运行时所有权约束，不是另建校验框架。随后 `buildAiGroup()` 构造固定字段对象，只显式拷贝已存在的 icon/hidden，不再展开整组。异常发生在 `main()` clone 的校验阶段，源配置不变。

代价：以前可混入保留组的非标准字段现在会报错；这是明确的行为收紧，不提供迁移别名或静默删字段。合法旧组和脚本重复执行继续有效。不要扩大此封闭形状规则到其他用户代理组。

**固定出口语义**：成功生成的家宽组没有替代成员；连接家宽的机场线路可以变化，最终家宽 endpoint 仍是同一配置。配置生成器不能证明 SOCKS 服务商提供静态 IP、宿主未修改输出或真实 UDP 不绕行。

## 3. R2：三个最小核心分组

| TOML / 常量 | 域名数组 | 默认值 |
|---|---|---|
| anthropic_core / ROUTE_ANTHROPIC_CORE | ANTHROPIC_CORE_SUFFIX_DOMAINS + ANTHROPIC_CORE_EXACT_DOMAINS | true |
| gemini_api_core / ROUTE_GEMINI_API_CORE | GEMINI_API_CORE_EXACT_DOMAINS | true |
| antigravity_core / ROUTE_ANTIGRAVITY_CORE | ANTIGRAVITY_CORE_EXACT_DOMAINS | true |

从现有 CORE_* 拆分，域名逐项为：

- Anthropic suffix：claude.ai、claude.com、claudemcpcontent.com、claudeusercontent.com。
- Anthropic exact：api.anthropic.com、mcp-proxy.anthropic.com、assets-proxy.anthropic.com。
- Gemini API exact：generativelanguage.googleapis.com。
- Antigravity exact：cloudcode-pa.googleapis.com、daily-cloudcode-pa.googleapis.com、cloudaicompanion.googleapis.com、antigravity.google。

不保留无消费者的 CORE_* 聚合别名，不改 RETIRED_CORE_* 清理集合。activeSuffix/Exact 按 boolean 选择；allPossibleSuffix/Exact 无条件收录全部新数组及既有 retired 内容。DNS builder 继续复用 active 和 allPossible 清单，不复制第二套域名表。

默认匹配集合、目标组、DNS policy 等价；非重叠同目标条目内部次序可变化，但“私网 → 域名 → IP → 实时 → DNS → 进程 → 原规则”的层次不可变。

## 4. R3：专属兜底的控制关系

| 输出 | 生效条件 |
|---|---|
| Anthropic IP CIDR | anthropic_core AND anthropic_ip_fallback |
| Claude/Claude Code 进程 | ai_process_fallback AND anthropic_core |
| ChatGPT/Codex/OpenAI 进程 | ai_process_fallback AND openai_core |
| Antigravity 进程 | ai_process_fallback AND antigravity_core |
| Cursor 进程 | ai_process_fallback AND cursor_process_fallback AND cursor_core |

Gemini API 不新增进程匹配，Grok 不新增进程匹配。现有各进程正则原样使用，不能匹配任意 node/python 进程。

**清理不可漏**：`buildAllProcessRules()` 必须始终返回完整旧/新进程规则集合。可以将已有 core process helper 按产品拆成小函数，由 active builder 加条件、all builder 无条件组合；不要把开关判断放进被清理调用的全量 helper。Anthropic IP 清理同样不受 active 条件影响。

这是对非默认组合的有意收紧：以往 openai_core=false + ai_process_fallback=true 仍全量捕获 OpenAI 进程，以后不生成这些进程规则。与参考项目“明确的服务选择”一致。

core=false 并不否定独立认证/辅助/资源开关，也不覆写用户规则；全局实时端口/DoH捕获仍可能抓到流量。Google 全部核心退出需同时关闭 gemini_web_core、gemini_api_core、vertex_ai_endpoints、antigravity_core；相关可选 auth/project/telemetry 按用户配置另算。文档列明这种组合，不新增聚合总开关。

## 5. R4：渲染、审计输入、清理的一致性

1. 在 `scripts/sync-local-config.js` 的 SWITCH_CONFIG_FIELDS 中登记三个 boolean，沿用现有类型校验、原子渲染及缺失默认补全，不引入新解析器。
2. 更新 `clash-verge-ai-residential.local.toml.example`。测试中从公开模板和临时虚构配置渲染，不触碰用户 *.local.toml / *.local.js。
3. 通过 Node 导出暴露三个常量及四个数组，为已有 build-inputs 消费。Clash 宿主不引入 Node-only API。
4. `SUPPORTED_SWITCH_BUILDERS` 增加三个映射，规则仍来自公开 buildInjectedRules。域名映射不包含 IP 字节或进程字节。24 个 routing 键 = 12 supported + 12 unsupported；原 unsupported 项保留。
5. 修改源 skill 中 21 的陈旧计数，说明新 core 的专属兜底关系及公开模板观测边界。只改 `skills/residential-rule-tuning/` 源文件，不自动安装到各平台目录或做全局更新。
6. 同步 docs/configuration.md、docs/local-configuration.md 及其 docs/en/ 对应文件的四份配置表，另同步 routing/DNS 文档、必要 README 与本任务触及的 frontend spec。不扩展到监控端统计存储或 UI。

## 6. R5：DNS 和“关闭”的准确合同

exact/suffix 核心规则关闭时，相应托管 DNS key 同时删除；默认非 AI nameserver 仍绑定已解析的机场上游，私网/国内/节点 bootstrap 的既有例外不变。

对于区域 Vertex 与启用时的 Cursor 索引 regex，保持当前域名路由，不生成等价 DNS policy。文档明确：本地需要真实解析时可能走默认非 AI resolver；fake-IP 或 SOCKS 域名转发不自动证明所有查询同出口。禁止用宽域名后缀、臆造正则 key 或引入远程规则依赖填补缺口。

“关闭”准确含义是撤销本脚本该类别的托管捕获，交回原 Profile；原 Profile 也可能选择 DIRECT/其他组/自定义家宽规则。不注入强制机场例外，不按目标批量删除未知规则。

## 7. 验证与回退

研究 validation.md 的 V1–V13 映射 PRD AC1–AC7。先补能在基线上暴露 F1/F2/F3 的 fixture，再完成运行时、renderer、audit mapping 联动。以真实生成对象和规则集验证，避免复制实现的测试。

实施后先相关 Node 套件，再 npm run ci，最后实际 just ci（包含 monitor-check）；不以文档旧描述替换实际门禁。没有现场运行授权时，保持真实配置加载、业务/UDP/DNS/最终 IP 验收 UNVERIFIED，不能将自动化完成等同于现场完成。

回退范围是该任务的源码/测试/文档变更；公开默认值不改变，尚未部署时无需修改用户网络。若未来已部署，回退必须针对已授权的具体本地配置版本进行，不自动触碰其他 Profile 或全局设置。

## 8. 完整变更范围与版本处理

唯一文件清单为 task.json.relatedFiles，与本段对应。实施发现额外必要文件时先补齐范围与影响说明；不能扩展到本地配置或监控端。

| 类别 | 文件 |
|---|---|
| 运行和渲染 | clash-verge-ai-residential.js；scripts/sync-local-config.js；clash-verge-ai-residential.local.toml.example |
| 回归与基线 | tests/regression.test.js；tests/sync-local-config.test.js；tests/install-agent-skills.test.js；新建 tests/fixtures/routing-default-v5.11.json |
| 审计输入与源技能 | skills/residential-rule-tuning/scripts/build-inputs.js；skills/residential-rule-tuning/SKILL.md；skills/residential-rule-tuning/reference.md |
| 配置文档 | docs/configuration.md；docs/en/configuration.md；docs/local-configuration.md；docs/en/local-configuration.md |
| 路由与 DNS | docs/routing-scope.md；docs/en/routing-scope.md；docs/dns-and-leak-model.md；docs/en/dns-and-leak-model.md |
| 必要规范 | .trellis/spec/frontend/index.md；.trellis/spec/frontend/component-guidelines.md；.trellis/spec/frontend/state-management.md；.trellis/spec/frontend/quality-guidelines.md |
| 变更说明 | CHANGELOG.md；README.md 仅在范围说明需要同步时修改 |

本任务不是发布任务。新增开关、非默认 process/IP 行为收紧和家宽组所有权变化，必须在 CHANGELOG.md 的 Unreleased 中用英文记录。SCRIPT_VERSION、脚本历史版本标题、package.json version 与 README 当前版本暂保持一致的 5.11.0；版本号更新由另行授权的发布工作统一处理，不在这里猜下一版本或给其他未发布改动发版。发布前不能把这些新行为称为已发布 v5.11.0 功能；说明其位于 Unreleased。

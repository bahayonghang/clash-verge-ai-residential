# 技术设计：Cursor/Grok 家宽路由 + TOML 键值自动补全

对应 prd.md。本设计分两部分：A. 路由清单与开关；B. 同步补全算法。

## A. 路由清单与开关

### A1. 数据流（沿用现有单一清单驱动）

域名清单常量 → `activeSuffixDomains()/activeExactDomains()/activeDomainRegexes()`
（受开关过滤）→ `buildDomainRules()` 生成 DOMAIN 规则 + `buildNameserverPolicy()`
生成 DNS policy；`allPossible*()`（不过滤开关）→ `buildManagedRuleSet()`
用于升级时的托管规则精确清理。新增域名与开关必须同时接入这三条链路，
否则会出现“关不掉的规则”或“关掉后残留的规则”。

### A2. Cursor 清单变更（clash-verge-ai-residential.js）

```js
const CURSOR_SUFFIX_DOMAINS = [
  "api2.cursor.sh",
  "api5.cursor.sh",
  "gcpp.cursor.sh",
  "authenticate.cursor.sh",   // 新增：官方授权端点
  "authentication.cursor.sh",
  "cursorvm.com"              // 新增：Cloud Agent VM 服务（官方 wildcard *.cursorvm.com）
];

const CURSOR_DOMAIN_REGEXES = [
  "^repo[0-9]+\\.cursor\\.sh$",
  "^adminportal[0-9]+\\.cursor\\.sh$"   // 新增：SSO 配置/域验证，编号可滚动
];
```

- `authenticate.cursor.sh` 用 suffix：官方文档将其列为独立授权主机，
  与 `authentication.cursor.sh`（JWT issuer）是不同主机；suffix 同时
  覆盖未来子域。
- `adminportal42.cursor.sh` 用 bounded regex 而非 suffix/exact：编号
  （42）与 repo42 一致地滚动，regex 既不写过期 exact 也不放大到整个
  `cursor.sh`。
- `cursorvm.com` 用 suffix：官方 wildcard `*.cursorvm.com` 与
  `*.*.cursorvm.com`，属于 Cloud Agent（agent 执行）运行时依赖。
- 不纳入（排除类，更新 docs/routing-scope.md）：`marketplace.cursorapi.com`
  （扩展市场）、`cursor-cdn.com`（CDN/更新）、`downloads.cursor.com`
  （客户端更新）、`anysphere-binaries.s3.us-east-1.amazonaws.com`
  （二进制下载）。

### A3. Grok 清单（新增）

```js
// Grok Build CLI 推理 API 与产品域；默认走家宽，可在本地 TOML 关闭。
const ROUTE_GROK_CORE = true;

const GROK_SUFFIX_DOMAINS = [
  // cli-chat-proxy.grok.com：推理 API（/v1/responses）、代码库与会话轨迹
  // 上传（/v1/storage*）；同主机还承载 Grok 网页产品与遥测，域名层无法拆分。
  "grok.com"
];
```

- 不纳入（注释 + docs 记录）：`api.mixpanel.com`（第三方分析，与
  statsig/sentry 同类排除）、`storage.googleapis.com`（全域共享 GCS，
  已存在于 claude_code_auxiliary 清单，默认关）、`x.ai`（安装脚本与
  隐私端点，非推理）。
- 无 exact/regex 清单（Grok 目前只有一个已观测产品域）。

### A4. 开关接线

1. `ROUTE_CURSOR_CORE` 默认值 `false` → `true`（js:119 附近）。
2. `SWITCH_CONFIG_FIELDS`（scripts/sync-local-config.js:20-43）新增：
   `{ table: "routing", key: "grok_core", constant: "ROUTE_GROK_CORE", type: "boolean" }`，
   插在 `cursor_core` 之后。
3. `activeSuffixDomains()` 加入 `...(ROUTE_GROK_CORE ? GROK_SUFFIX_DOMAINS : [])`；
   `allPossibleSuffixDomains()` 无条件加入 `GROK_SUFFIX_DOMAINS`。
4. module.exports constants 增加 `ROUTE_GROK_CORE`、`GROK_SUFFIX_DOMAINS`。
5. example TOML `[routing]`：`cursor_core = true`、新增 `grok_core = true`
   （顺序与 SWITCH_CONFIG_FIELDS 一致）。
6. 版本：`SCRIPT_VERSION` 与 package.json `5.5.0 → 5.6.0`；js 头部注释
   与 v5.6 要点说明更新。

### A5. 兼容性

- 升级用户的本地 TOML 缺 `grok_core` → 由 B 部分补全为 example 默认
  `true`；补全前若直接渲染，旧行为是保持 JS 模板默认（也是 `true`），
  两者一致，无行为突变。
- 曾以 `cursor_core = true` 渲染过的本地 js，升级后重新渲染即可；
  `cleanExistingManagedRules` 经 allPossible* 已能清理新增域名的规则。
- `cursor_core` 默认值翻转只影响“本地 TOML 未显式写该键”的用户——
  补全机制会把 example 默认 `true` 写进本地 TOML，用户可见可关。

## B. 同步补全算法（scripts/sync-local-config.js）

### B0. 约束

- 零依赖（不引入 TOML 写入库）；本地 TOML 是用户文件：已有键值、
  注释、空行、行尾风格（CRLF/LF）必须逐字保留；只允许“追加缺失键”，
  不允许重排或重写已有内容。
- 幂等：无缺失时不改写文件（保持 mtime，方便用户 diff）。
- 原子写入：复用 `writeFileAtomically` 模式（tmp + rename）。

### B1. 默认值来源：example 文件（单一事实来源）

新增 `DEFAULT_EXAMPLE_PATH = clash-verge-ai-residential.local.toml.example`。

- 解析 example 用现有 `parseLocalToml`（example 由 CI/测试保证合法且
  键齐全），得到 `Map<table, Map<key, value>>`。
- 不在 sync 脚本里硬编码第二份默认值，避免 example 与脚本漂移；
  也不从 JS 模板正则反推默认值（渲染方向是 TOML→JS，反向解析脆弱）。
- example 缺某个 SWITCH_CONFIG_FIELDS 键时视为上游错误，抛错提示
  补齐 example，而不是静默用 JS 默认。

### B2. 补全算法（文本级插入）

```
completeLocalToml(localSource, exampleConfig):
  1. lines = localSource.split(/\r?\n/)（保留原始行串，含注释）
  2. linesWithMeta = 逐行标注：表头行（含表名）｜赋值行（含 key）｜其他
     —— 复用 stripComment + 与 parseLocalToml 相同的匹配正则，
        保证“能被解析器看到的行”与“能被补全器定位的行”一致。
  3. existing = 解析已出现的 (table,key) 集合（不需要值，只要键）
  4. missing = SWITCH_CONFIG_FIELDS 中 (table,key) ∉ existing 的项，
     按 SWITCH_CONFIG_FIELDS 声明顺序
  5. 若 missing 为空 → 返回 null（调用方不写文件）
  6. EOL = 源文本以 \r\n 为主的判定（\r\n 出现且多于纯 \n 时用 \r\n）
  7. 对每个缺失键按所属表分组：
     a. 表头已存在 → 插入点 = 该表区块内最后一个“非空行”之后
        （区块 = 表头行到下一个表头行或 EOF 之间；表尾注释行也属于
        该表，因此插入点取区块最后一个非空行后，而非表头后，
        避免把键插到表头注释与键之间）
     b. 表头不存在（整个表缺失）→ 插入点 = 文件末尾，先补一个空行
        和 `[table]` 表头再接键值行
  8. 生成缺失键行：`${key} = ${formatValue(default)}`（布尔输出
     true/false；此处 SWITCH 字段全为布尔，formatValue 只需支持布尔）
  9. 从后往前按插入点 splice（从后往前保证前面的插入点行号不失真），
     重组文本（用 EOL join，保持结尾换行与原文件一致）
```

关键点分析（为什么这样实现）：

- **为什么文本级插入而不是 parse→重新序列化**：本地 TOML 的注释、
  空行、键顺序都是用户手写内容，任何重新序列化都会破坏“逐字保留”
  约束；项目现有解析器只 parse 不 serialize，自写 serializer 还要
  处理字符串转义/多行值等 TOML 语法面，成本远高于插入。
- **为什么插入点选表尾而非表头后**：example 中表头下常有说明注释
  （如 `[routing]` 上方/下方的中文注释行属于文件级或表级注释），
  插到区块末尾对注释归属最安全。
- **为什么从后往前 splice**：一次补全可能涉及多个插入点，倒序插入
  避免行号偏移计算。
- **CRLF**：split 已经吃掉行尾，重组时统一用检测到的 EOL；文件末尾
  原本有/没有换行符都保持原样。
- **未知表/未知键仍报错**：补全只处理“SWITCH_CONFIG_FIELDS 里声明的
  键缺失”，解析器对未知键的严格报错（防拼写错误）不变。

### B3. 集成点（syncLocalConfig 流程）

```
syncLocalConfig():
  1. 读取 template、example、local toml
  2. exampleConfig = parseLocalToml(exampleSource)          // B1
  3. completed = completeLocalToml(localSource, exampleConfig) // B2
  4. 若 completed：
       writeFileAtomically(configPath, completed.source)
       重新 parse + validate（防止补全器 bug 产出非法 TOML）
       console.log 提示补充了哪些键（table.key 列表）
       后续渲染使用补全后的 source/config
  5. 原有渲染流程不变（injectHomeProxyTemplate + injectBooleanConstants）
```

- `home_proxy` 的 REQUIRED_KEYS 不参与补全（B2 的 missing 只从
  SWITCH_CONFIG_FIELDS 枚举，天然不含 home_proxy），缺键继续由
  `validateHomeProxyConfig` 报错——凭据必须手填，自动补 `xxx` 占位符
  只会把失败推迟到 Clash Verge 运行时。
- CLI 输出（main）补充一行：`已补充 N 个缺失开关到 <toml>：...`。
- justfile `render-local` 配方注释更新说明自动补全行为（逻辑不变，
  仍是一次 node 调用）。

### B4. 测试设计（tests/sync-local-config.test.js）

1. 缺失键 + 缺整表：本地 toml 去掉 `grok_core`、`public_encrypted_dns`
   和整个 `[runtime]` → sync 后：缺失项按 example 默认值出现、顺序符合
   SWITCH_CONFIG_FIELDS、用户已有键值与注释逐字保留、渲染产物中
   `ROUTE_GROK_CORE = true` 等被正确注入。
2. 幂等：对补全后的文件再跑一次 → configPath 内容不变（比对字符串）。
3. 完整文件：不缺键 → 不写文件（可断言 writeFileAtomically 未被触发，
   用内容比对即可）。
4. home_proxy 缺键 → 抛出与现状一致的错误信息。
5. CRLF 文件补全后仍为 CRLF。
6. example 与 SWITCH_CONFIG_FIELDS 一致性：example 必须包含全部
   SWITCH_CONFIG_FIELDS 键（防漂移守卫，双向：多键/少键都报错）。

### B5. 回滚

- 路由部分：恢复 `ROUTE_CURSOR_CORE=false`、删除 `ROUTE_GROK_CORE` 与
  GROK 清单、example 撤销两行；托管清理集合随 allPossible* 同步回退，
  重新渲染即清理已注入规则。
- 补全部分：补全只在本地 TOML 追加行，回滚 = 移除 completeLocalToml
  调用；已补全的本地 TOML 行本身无害（键合法、值与模板默认一致）。

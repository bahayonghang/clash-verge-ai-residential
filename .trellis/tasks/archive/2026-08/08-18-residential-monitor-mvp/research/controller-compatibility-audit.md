# Research: mihomo 控制器与 Clash Verge Rev 私有 IPC 兼容性核验

- **Query**: 使用 mihomo 与 Clash Verge Rev 一手源码/官方文档核验 `/connections`、`/traffic`、`DELETE` 连接、Windows named pipe、鉴权、ACL、`ERROR_PIPE_BUSY`、`chains` 顺序稳定性、Verge 私有 IPC 兼容风险，并给出完整 Windows 11 监控应用的设计约束。
- **Scope**: mixed（mihomo 官方文档与源码、Clash Verge Rev 官方源码与维护者说明、Microsoft Win32 官方文档、本机只读实测）
- **Date**: 2026-08-18

## 结论

1. `/connections`、`/traffic`、`DELETE /connections` 和 `DELETE /connections/{id}` 是 mihomo 的公开 API；Windows named pipe 也是 mihomo 的公开配置入口。
2. Clash Verge Rev 内部使用哪一个管道名、如何派生管道名、何时切换 sidecar/service 管道，不是 Clash Verge Rev 对第三方承诺的公开契约。维护者明确拒绝为内部 IPC 提供外部自动化支持，并要求此类场景启用 External Controller（TCP）。
3. mihomo named pipe **不校验 `secret`**。TCP 在 `secret` 非空时使用 Bearer 鉴权；管道的安全边界只有 Windows ACL。把 Bearer 加到管道请求中不会增加保护。
4. 当前稳定版 Clash Verge Rev `v2.5.2` 使用固定管道 `\\.\pipe\verge-mihomo`；2026-08-18 的 `dev` 源码已改为按运行模式、发布通道和 Windows 用户派生管道名，并收紧 ACL。固定管道名不能作为长期契约。
5. `chains` 是公开响应字段，但其顺序语义没有写入公开文档。当前 mihomo 实现和本机样本均表现为「具体出站在前、外层代理组在后」；这只能作为当前实现细节。分类必须按目标集合与用户优先级决定，不得依赖数组首尾。
6. `ERROR_PIPE_BUSY` 是正常的瞬态连接结果，不表示管道不存在。客户端必须设置总时限并重试。Clash Verge Rev `v2.5.2` 和当前开发版插件对该错误都存在无界重试路径，监控应用不得照搬。
7. 连接快照只能提供活跃连接及采样时刻的累计字节。连接退出后会立即从管理器删除，因此逐连接账本存在最后一个采样周期的尾差。该数据源不能生成严格计费账单，只能生成带覆盖率和缺口说明的监控统计。

## 证据分级

- **公开契约**：mihomo 官方文档或 Microsoft Win32 官方文档明确承诺的行为。
- **当前实现细节**：固定到具体 Git 提交的源码行为；升级后可能变化。
- **本机实测**：2026-08-18 在本机 Clash Verge Rev `2.5.2`、mihomo `v1.19.29` 上执行的只读验证。
- **危险假设**：现有证据否定，或没有公开契约支撑，不能进入稳定设计的前提。

## 核验基线

| 对象 | 固定版本或提交 | 用途 |
|---|---|---|
| 本机 Clash Verge Rev | `2.5.2`，发布提交 `28f2efc504059b1dc75c793618b775c8e1b2a5f1` | 对照本机行为 |
| 本机 mihomo | `v1.19.29` | 对照本机 API 响应 |
| mihomo 最新稳定版 | `v1.19.30` | 检查当前公开版本 |
| mihomo 审计源码快照 | `fe22fdd2ccd37915676af3be41434e832e541872`（Alpha，2026-08-17） | 核验控制器当前实现 |
| mihomo 官方文档快照 | `MetaCubeX/Meta-Docs@e848aefb77e0cddbf3f0dde1016ec4904924fcbd` | 核验公开契约 |
| Clash Verge Rev 开发树 | `4c1804baf48a34e2132a955f65899daf8d424022`（`dev`，包版本 `2.5.3`，尚非稳定发布） | 识别即将发生的 IPC 变化 |
| Clash Verge Service IPC | `v2.6.2@edf7cb57811d4e572f1e2d607472cb87bd421ef4` | 核验 service 模式管道派生和 ACL |

mihomo `v1.19.29` 到 `v1.19.30` 的相关控制器文件没有行为性改动；比较结果只有 `constant/adapters.go` 的适配器枚举发生变化。`v1.19.30` 到审计快照 `fe22fdd...` 的相关文件没有变化。

## API 与传输核验

### `/connections`

**公开契约**

- 支持 `GET` 和 WebSocket。
- WebSocket 参数 `interval` 的单位是毫秒，默认值是 `1000`。
- 响应包含全局 `downloadTotal`、`uploadTotal`、`memory` 和活跃连接数组。
- 连接包含 `id`、`metadata`、逐连接累计 `upload`/`download`、`start`、`chains`、`providerChains`、`rule`、`rulePayload`。
- 公开文档只把 `chains` 定义为代理链数组，没有规定首尾顺序。

**当前实现细节**

- 普通 `GET` 立即返回一次 `DefaultManager.Snapshot()`。
- WebSocket 连接成功后先立即发送一次快照，再按 `interval` 建立 ticker。
- 快照通过并发 map 的 `Range` 收集，连接数组顺序没有排序保证。UI 必须自行排序，并以 `id` 做状态合并。
- tracker 关闭时先从 manager 删除，再关闭底层连接。后续快照不会再包含该连接。
- `interval <= 0` 会传入 `time.NewTicker`。客户端只应使用正数，建议保持默认 `1000`。

**设计含义**

- 每个核心进程建立一个采集 epoch；持久主键使用 `(epoch, connection_id)`，不能假设 UUID 跨核心重启全局唯一。
- 对每个连接计算 `max(0, current - previous)`。计数器下降时结束旧 epoch 或重置该连接基线。
- 首次看到连接时，不能把已有累计值全部当成本应用观测期内的新流量；需要明确采用「从首次观测值开始计」或「把首次观测累计值记为启动前未知区间」中的一种。完整报告应保留采集覆盖边界。
- 连接消失时只能以最后一次快照为终值。采样间隔内的尾部字节不可恢复。
- 连接列表顺序变化不是连接增删信号。

### `/traffic`

**公开契约**

- 支持 `GET` 和 WebSocket。
- 每秒推送一次。
- `up`、`down` 的单位是 B/s；`upTotal`、`downTotal` 的单位是字节。

**当前实现细节**

- 普通 HTTP `GET` 也进入永久循环，每秒写一行 JSON 并 flush；它不是「请求一次、读到 EOF」的普通 REST 响应。
- WebSocket 走同一组计数器。
- 全局总量来自 manager 的进程内累计值，核心重启或显式重置统计后会归零。

**兼容约束**

- 解析模型应允许旧核心缺少 `upTotal`、`downTotal`，但本机 `v1.19.29` 和当前官方文档均包含四个字段。
- 若应用只需全局速率，可从相邻 `/connections` 快照的全局总量推导，减少一个长期管道连接。若使用 `/traffic`，必须按流式 HTTP 或 WebSocket 处理。
- `/traffic` 与 `/connections` 的总量来自同一 manager，不得把两者相加。
- Windows 休眠恢复后，不应把长时间跨度的总量差平均摊入一个正常采样周期。应记录休眠/缺口区间。

### 关闭连接

**公开契约**

- `DELETE /connections` 关闭全部连接并返回 HTTP `204`。
- `DELETE /connections/{id}` 关闭指定连接并返回 HTTP `204`。

**当前实现细节**

- 指定 `id` 不存在时，mihomo 仍返回 `204`。
- `204` 只说明请求处理完成，不能证明该 `id` 在请求时存在，也不能证明下一帧已经看不到该连接。

**设计含义**

- UI 文案应为「已发送关闭请求」，随后等待连接从快照消失；不要仅凭 `204` 显示「连接已关闭」。
- 只允许关闭当前快照中的 `id`。关闭全部连接应视为高风险操作；当前 PRD 只需要关闭单条连接，不应暴露关闭全部连接。
- 发出 `DELETE` 前保存该连接最后可见计数，但仍要声明存在并发增长和最后采样尾差。
- 管道没有只读角色。获得管道访问权的进程同时具备调用所有控制器写接口的能力。

## 鉴权与 ACL

### TCP 控制器

**公开契约与当前实现**

- 官方示例使用 `Authorization: Bearer ${secret}`。
- 当前实现仅在 `secret` 非空时安装鉴权中间件。
- WebSocket 可使用 `?token=<secret>`；非浏览器客户端也可发送 `Authorization` 请求头。
- `secret` 为空时，TCP 控制器不要求 Bearer。

### Windows named pipe

**公开契约与当前实现**

- 官方文档明确说明：通过 `external-controller-pipe` 访问 API 时不校验 `secret`。
- mihomo 启动 pipe server 时显式向 router 传入空 secret，因此不会安装鉴权中间件。
- 向管道请求添加 Bearer 不会改变鉴权结果，也不应把真实 TCP secret 发送给可能被抢占名称的未知管道。

**mihomo 默认 ACL**

- 当前默认 SDDL 是 `D:PAI(A;OICI;GWGR;;;BU)(A;OICI;GWGR;;;SY)`。
- 默认允许 `BU`（BUILTIN\Users）和 `SY`（LocalSystem）读写。
- 环境变量 `LISTEN_NAMEDPIPE_SDDL` 可覆盖默认 SDDL。

**Clash Verge Rev 的版本差异**

- 本机 `v2.5.2` 未覆盖 mihomo 默认 ACL。实测管道 owner 为 `BUILTIN\Administrators`，DACL 向 `NT AUTHORITY\SYSTEM` 和 `BUILTIN\Users` 授予读、写、同步。
- 当前 `dev` 源码在 sidecar 模式设置 `LISTEN_NAMEDPIPE_SDDL`，只允许当前 Windows 用户 SID、LocalSystem 和管理员完全访问。
- 当前 service IPC `v2.6.2` 也按 owner SID 生成相同的受保护 DACL，并在启动 mihomo 时传入该环境变量。

**Windows 11 应用约束**

- 采集进程应在当前登录用户会话中运行，不应为了访问管道而常驻为 LocalSystem 服务。
- `Access denied` 表示 ACL 不允许当前进程访问，不是 secret 错误。恢复方式是使用同一 Windows 用户运行，或引导用户启用受支持的 TCP External Controller；不要把它显示为「密钥错误」。
- TCP 的 HTTP `401` 才映射为「认证未通过」。管道模式不会因 secret 返回 `401`。
- 不要修改 Verge 或 mihomo 的 ACL，不要设置 `LISTEN_NAMEDPIPE_SDDL`，不要为了兼容旧版本扩大管道权限。

## `ERROR_PIPE_BUSY`

**Windows 公开契约**

- 当管道存在但所有实例都忙时，`CreateFile` 返回 `ERROR_PIPE_BUSY`（231）。
- Microsoft 的客户端流程是：先尝试打开；收到 `ERROR_PIPE_BUSY` 后调用 `WaitNamedPipe`；可用后再次打开。

**mihomo 当前实现**

- mihomo 使用的 `metacubex/wireguard-go` listener 将最大实例数传为 `0xffffffff`，不是单实例 server。
- listener 每次 `Accept` 创建并等待一个新实例。长连接不会永久独占整个管道，但启动、实例交接和并发抢占期间仍可能短暂出现 `ERROR_PIPE_BUSY`。

**Clash Verge Rev 插件现状**

- `v2.5.2` 固定的插件提交在 `ERROR_PIPE_BUSY` 分支不减少 `max_retry_count`，因此该错误可导致无界循环。
- 当前开发插件同样每 50 ms 重试 `ERROR_PIPE_BUSY`，没有总时限。
- 这两个行为属于 Verge 私有客户端实现，不能作为监控应用的重试规范。

**监控应用约束**

- 为一次 pipe open 设置可取消的总时限；在时限内对 `ERROR_PIPE_BUSY` 退避重试。
- 将 `ERROR_FILE_NOT_FOUND`、`ERROR_ACCESS_DENIED`、`ERROR_PIPE_BUSY` 超时、HTTP/协议错误拆成不同状态。
- 应用退出、暂停采集、切换控制器和系统休眠时，取消尚未完成的 open 与重连任务。
- 限制并发打开数量。建议只保留一个 `/connections` WebSocket；短请求串行或使用小并发上限。

## `chains` 顺序稳定性

### 可以确认的事实

- 公开 API 只承诺 `chains` 是数组，不承诺顺序语义。
- 当前实现创建底层连接时先追加具体出站名称；代理组包装连接时继续 append 组名。
- `Chain.Last()` 的当前实现返回 `c[0]`，日志格式在多元素时输出 `c[len(c)-1][c[0]]`。这印证当前内部语义是首项靠近实际出站，末项靠近外层规则组。
- tracker 在连接创建时复制 `conn.Chains()`，连接生命周期内通常保持不变。

### 本机样本

- 本机一次只读快照中，共有 24 条连接同时命中 `家宽-SOCKS5` 和 `AI-家宽`。
- 24 条连接均以 `家宽-SOCKS5` 为首项，以 `AI-家宽` 为末项；样本最大链长为 3。
- 该结果只能证明本机 `v1.19.29` 当前拓扑，不构成跨版本或跨拓扑保证。dialer-proxy、relay、provider chain 和未来实现都可能改变可见结构。

### 设计约束

- 分类算法对 `chains` 做精确名称集合匹配，再按用户配置的目标优先级选择唯一主分类。
- 不使用 `chains[0]`、`chains[-1]` 或反转后的显示顺序决定分类。
- 保留原始 `chains` 顺序用于诊断和展示，同时保存全部命中标签。
- 对缺失、空数组、重复名称和新增 `providerChains` 字段保持兼容。

## Clash Verge Rev 私有 IPC 风险

### 稳定版 `v2.5.2`

- Windows 路径固定为 `\\.\pipe\verge-mihomo`。
- 运行时 YAML 写入相同的 `external-controller-pipe`。
- 本机该版本可以通过固定管道直接访问 mihomo HTTP/WebSocket API。

### 当前 `dev` 实现

- sidecar 路径已变为 `\\.\pipe\verge-mihomo-sidecar-{release|dev}-{owner_key}`。
- service 模式的 mihomo 路径已变为 `\\.\pipe\verge-mihomo-{channel_id}-{owner_key}`。
- sidecar 与 service 使用不同路径。Clash Verge Rev 会在模式切换时更新自己的 mihomo 客户端。
- 当前生成配置仍调用 `sidecar_ipc_path()` 写入 `external-controller-pipe`；service 实际启动 mihomo 时再用命令行 `-ext-ctl-pipe <core_ipc_path>` 覆盖该值。因此在当前开发树的 service 模式下，运行时 YAML 中的管道值不一定是实际监听端点。
- `\\.\pipe\clash-verge-service` 是 Verge 服务控制协议，不是 mihomo HTTP API。监控应用不得连接或复刻该私有协议。

### 官方支持边界

Clash Verge Rev 维护者在 issue `#6886` 中明确说明：

- 内部调用从 HTTP 迁移到 IPC 是安全设计的一部分。
- 项目不会为外部使用者记录 named pipe/Unix socket 的调用方式。
- 需要外部控制时，应启用 External Controller；脚本化使用内部 IPC 不受支持。

因此，「Verge 自己在用，所以第三方可稳定复用」是危险假设。

### 完整 Windows 11 应用的兼容策略

1. 将 TCP External Controller 定义为受支持传输；将 Verge 内部 named pipe 定义为版本化、尽力兼容的传输。
2. 自动发现不能只读取 `clash-verge.yaml`：
   - 对 `v2.5.2`，配置值和固定管道可作为候选。
   - 对当前开发布局，必须区分 sidecar 与 service，并允许运行时端点与 YAML 不同。
   - 无法可靠解析私有布局时，停止猜测并引导用户启用 TCP External Controller。
3. 不复制 `owner_key` 私有算法作为永久协议。若枚举 `verge-mihomo*` 候选，连接后使用 `GetNamedPipeServerProcessId` 核对 server PID，并确认进程属于当前运行的 `verge-mihomo*.exe` 或用户选择的 mihomo 核心。Verge service 自身也执行 server PID 核验。
4. 每次核心重启、运行模式切换、配置文件替换或连续连接失败后，重新发现端点。不要永久缓存管道名。
5. 连接未知管道时不发送 secret。先用无敏感信息的 `GET /version` 完成能力探测。
6. 私有管道布局未知、ACL 拒绝或 server 身份核验失败时，UI 给出「内部控制通道不兼容」并提供启用 TCP 的操作说明。

## 完整 Windows 11 监控应用设计约束

### 端点与状态模型

- 端点模型至少包含 `Tcp { address, credential_target }` 和 `NamedPipe { path, compatibility_profile }`；secret 由 Rust CredentialStore 在连接时解析，不内联进可序列化端点配置。
- 状态至少区分：连接中、已连接、TCP 认证未通过、管道访问被拒绝、管道忙超时、端点不存在、协议不兼容、核心已重启、系统休眠/采集缺口、存储故障。
- 测试连接依次验证 server 身份（管道）、`GET /version`、必要鉴权和所需字段；不要只验证「能打开流」。
- 记录 `transport`、mihomo 版本、Verge 兼容配置和最后成功时间，便于诊断升级回归。

### HTTP 与 WebSocket

- HTTP/1.1 解析器必须支持 `Content-Length`、chunked、多段读取和 connection-close framing。本机 `/connections` 的 HTTP/1.1 响应实际使用 chunked。
- 不要用「读取到 EOF」实现 `/traffic`，因为普通 GET 是长期流。
- WebSocket 使用标准握手和帧解析；管道模式不需要 `token`。TCP 模式按 secret 决定 Bearer 或 token。
- 对所有响应启用宽松反序列化：忽略未知字段，将新增字段保留到原始 JSON 或扩展区，将版本相关字段设为可选。
- 限制响应体和帧大小，但阈值必须覆盖大量活跃连接和大型 `/proxies` 响应。

### 统计与持久化

- 保存全部原始连接明细以及观测 epoch，后续按新目标配置重新分析。
- 主分类由用户目标优先级决定；调整优先级不依赖 `chains` 顺序。
- 全局总量、目标分类总量和「其他连接」必须从同一批连接事实计算，避免重复累计。
- 核心重启、计数器下降、应用断线、进程退出和系统休眠都建立显式数据缺口，不写成零流量。
- 报告声明逐连接账本是采样下界。若需要更接近全局准确值，可同时保存 manager 全局总量差，但无法把未采到的尾差准确分配到域名、进程或分类。
- 应用自动启动后先建立新覆盖区间；不得把启动前的 mihomo 累计总量归入应用历史。

### 安全与隐私

- secret 只用于 TCP，不写日志、不放入管道 URL、不出现在导出文件。
- 连接元数据可能含域名、IP、端口、进程名和完整进程路径。数据库、诊断日志和导出分别设置字段白名单与脱敏选项。
- `DELETE` 必须由明确的用户操作触发；后台采集任务不得自动关闭连接。
- named pipe 没有读写权限分离。设置页应说明：启用管道兼容模式后，应用获得完整 mihomo 控制器能力。

### 必须覆盖的兼容性测试

1. Clash Verge Rev `v2.5.2` 固定管道、TCP 关闭。
2. 当前开发布局的 sidecar 与 service 两种模式，以及二者切换。
3. TCP secret 非空、secret 为空、错误 secret。
4. 管道不带 Authorization 仍成功；管道 ACL 拒绝时显示正确状态。
5. 注入 `ERROR_PIPE_BUSY`，验证重试可取消且在总时限后退出。
6. `/connections` 数组随机重排，统计结果不变。
7. 同一 `chains` 正序、逆序和多目标命中，主分类都只按用户优先级决定。
8. 核心重启导致全局和逐连接计数器归零。
9. 应用休眠、网络断开、Verge 退出后恢复，缺口不被记为零。
10. `DELETE` 不存在的 ID 返回 `204`，UI 不误报已关闭。
11. `/traffic` 缺少可选 totals 的旧版本响应。
12. 管道候选被其他进程抢占时，server PID 核验拒绝该端点。

## 本机实测

测试均为只读请求，没有读取或输出真实 secret，没有执行 `DELETE`。

| 项目 | 结果 | 分级 |
|---|---|---|
| 进程版本 | `clash-verge.exe 2.5.2`；`GET /version` 返回 mihomo `v1.19.29` | 本机实测 |
| 运行时配置 | `external-controller: ''`；`external-controller-pipe: \\.\pipe\verge-mihomo` | 本机实测 |
| 无 Bearer 的 `/version` | HTTP `200` | 本机实测；符合管道免 secret 契约 |
| 无 Bearer 的 `/connections` | HTTP `200`，包含 `connections`、`downloadTotal`、`uploadTotal`、`memory` | 本机实测 |
| 无 Bearer 的 `/traffic` | HTTP `200`，首帧包含 `up`、`down`、`upTotal`、`downTotal` | 本机实测 |
| HTTP framing | `/connections` 的 HTTP/1.1 响应使用 chunked；改用 HTTP/1.0 后按 connection-close 成功解析 | 本机实测 |
| 管道 ACL | owner `BUILTIN\Administrators`；SYSTEM 与 BUILTIN\Users 可读写 | 本机实测 |
| `chains` | 24 个目标样本全部是具体出站在首位、目标组在末位；最大链长 3 | 本机实测，不是公开契约 |
| `DELETE` | 未执行，避免中断用户现有连接 | 未实测；由官方文档与源码核验 |

## 危险假设清单

| 假设 | 结论 | 原因 |
|---|---|---|
| `\\.\pipe\verge-mihomo` 永久固定 | 错误 | 当前 `dev` 已改为按用户、模式和通道派生 |
| `clash-verge.yaml` 永远给出实际监听管道 | 错误 | 当前 `dev` service 模式通过 CLI 覆盖配置值 |
| pipe 请求携带 Bearer 就更安全 | 错误 | pipe router 不安装 secret 鉴权；还可能把 secret 泄露给伪造管道 |
| pipe 密钥错误会返回 `401` | 错误 | pipe 不校验 secret；ACL 错误表现为 Windows access denied |
| `chains[0]` 或末项是稳定公开语义 | 未获支持 | 只有当前实现和本机样本支持，官方文档未规定 |
| `204` 证明指定连接存在并已关闭 | 错误 | 不存在的 ID 也返回 `204` |
| `/traffic` 的普通 GET 是一次性 JSON | 错误 | 当前实现每秒持续写入 |
| named pipe 支持多实例，所以不会出现 busy | 错误 | Win32 明确定义 `ERROR_PIPE_BUSY`，实例交接仍可能触发 |
| Verge 内部 IPC 受官方外部兼容承诺 | 错误 | 维护者明确拒绝支持该用途 |
| 快照累计可作为严格计费账单 | 错误 | 活跃连接采样存在尾差、断线和休眠缺口 |

## 固定来源

### mihomo 公开文档

- [API 鉴权示例（固定提交）](https://github.com/MetaCubeX/Meta-Docs/blob/e848aefb77e0cddbf3f0dde1016ec4904924fcbd/docs/api/index.en.md#L7-L18)
- [`/traffic` 字段、单位与推送周期（固定提交）](https://github.com/MetaCubeX/Meta-Docs/blob/e848aefb77e0cddbf3f0dde1016ec4904924fcbd/docs/api/index.en.md#L45-L55)
- [`/connections`、`DELETE` 契约（固定提交）](https://github.com/MetaCubeX/Meta-Docs/blob/e848aefb77e0cddbf3f0dde1016ec4904924fcbd/docs/api/index.en.md#L372-L405)
- [Windows named pipe 不验证 secret（固定提交）](https://github.com/MetaCubeX/Meta-Docs/blob/e848aefb77e0cddbf3f0dde1016ec4904924fcbd/docs/config/general.en.md#L143-L188)

### mihomo 实现

- [`/connections` 路由、快照、WebSocket ticker 和 DELETE](https://github.com/MetaCubeX/mihomo/blob/fe22fdd2ccd37915676af3be41434e832e541872/hub/route/connections.go#L16-L87)
- [TCP 鉴权中间件与路由](https://github.com/MetaCubeX/mihomo/blob/fe22fdd2ccd37915676af3be41434e832e541872/hub/route/server.go#L105-L142)
- [pipe router 显式使用空 secret](https://github.com/MetaCubeX/mihomo/blob/fe22fdd2ccd37915676af3be41434e832e541872/hub/route/server.go#L299-L330)
- [Bearer、WebSocket token 鉴权实现](https://github.com/MetaCubeX/mihomo/blob/fe22fdd2ccd37915676af3be41434e832e541872/hub/route/server.go#L336-L369)
- [`/traffic` 的持续流实现](https://github.com/MetaCubeX/mihomo/blob/fe22fdd2ccd37915676af3be41434e832e541872/hub/route/server.go#L371-L411)
- [Windows pipe 默认 SDDL 与环境变量覆盖](https://github.com/MetaCubeX/mihomo/blob/fe22fdd2ccd37915676af3be41434e832e541872/adapter/inbound/listen_windows.go#L13-L31)
- [活跃连接 manager 与快照](https://github.com/MetaCubeX/mihomo/blob/fe22fdd2ccd37915676af3be41434e832e541872/tunnel/statistic/manager.go#L40-L98)
- [逐连接计数与关闭时删除](https://github.com/MetaCubeX/mihomo/blob/fe22fdd2ccd37915676af3be41434e832e541872/tunnel/statistic/tracker.go#L24-L35)
- [TCP tracker 保存链并累加计数](https://github.com/MetaCubeX/mihomo/blob/fe22fdd2ccd37915676af3be41434e832e541872/tunnel/statistic/tracker.go#L111-L145)
- [`Chain` 当前语义](https://github.com/MetaCubeX/mihomo/blob/fe22fdd2ccd37915676af3be41434e832e541872/constant/adapters.go#L76-L94)
- [连接创建和 `AppendToChains`](https://github.com/MetaCubeX/mihomo/blob/fe22fdd2ccd37915676af3be41434e832e541872/adapter/outbound/base.go#L246-L261)
- [named pipe 客户端对 `ERROR_PIPE_BUSY` 的上游处理](https://github.com/metacubex/wireguard-go/blob/a6cecdd7f57f01c6b90b9f113a692b767bca64cf/ipc/namedpipe/namedpipe.go#L119-L138)
- [named pipe 最大实例数与 listener](https://github.com/metacubex/wireguard-go/blob/a6cecdd7f57f01c6b90b9f113a692b767bca64cf/ipc/namedpipe/namedpipe.go#L255-L373)

### Clash Verge Rev

- [`v2.5.2` 固定 Windows 管道](https://github.com/clash-verge-rev/clash-verge-rev/blob/28f2efc504059b1dc75c793618b775c8e1b2a5f1/src-tauri/src/utils/dirs.rs#L225-L247)
- [`v2.5.2` 是否开启 TCP External Controller](https://github.com/clash-verge-rev/clash-verge-rev/blob/28f2efc504059b1dc75c793618b775c8e1b2a5f1/src-tauri/src/config/clash.rs#L288-L330)
- [`v2.5.2` 插件的 busy 无界重试分支](https://github.com/clash-verge-rev/tauri-plugin-mihomo/blob/cf97ff99e390a9b437d5cf94c6f454f024fc8f69/src/ipc.rs#L220-L247)
- [维护者说明内部 IPC 不提供外部支持](https://github.com/clash-verge-rev/clash-verge-rev/issues/6886#issuecomment-4302722904)
- [维护者拒绝脚本化使用内部 IPC](https://github.com/clash-verge-rev/clash-verge-rev/issues/6886#issuecomment-4302757305)
- [当前 `dev` 的动态 sidecar 管道](https://github.com/clash-verge-rev/clash-verge-rev/blob/4c1804baf48a34e2132a955f65899daf8d424022/src-tauri/src/utils/dirs.rs#L240-L318)
- [当前 `dev` 的当前用户 pipe SDDL](https://github.com/clash-verge-rev/clash-verge-rev/blob/4c1804baf48a34e2132a955f65899daf8d424022/src-tauri/src/core/owner_identity.rs#L69-L74)
- [当前 `dev` 区分 sidecar/service mihomo 客户端路径](https://github.com/clash-verge-rev/clash-verge-rev/blob/4c1804baf48a34e2132a955f65899daf8d424022/src-tauri/src/core/manager/state.rs#L105-L140)
- [当前 `dev` service 客户端路径](https://github.com/clash-verge-rev/clash-verge-rev/blob/4c1804baf48a34e2132a955f65899daf8d424022/src-tauri/src/core/manager/state.rs#L269-L279)
- [当前配置生成仍使用 sidecar path](https://github.com/clash-verge-rev/clash-verge-rev/blob/4c1804baf48a34e2132a955f65899daf8d424022/src-tauri/src/config/clash.rs#L320-L333)
- [service 模式动态 mihomo 管道名](https://github.com/clash-verge-rev/clash-verge-service-ipc/blob/edf7cb57811d4e572f1e2d607472cb87bd421ef4/src/core/paths.rs#L212-L253)
- [service 通过 CLI 覆盖 mihomo 控制管道](https://github.com/clash-verge-rev/clash-verge-service-ipc/blob/edf7cb57811d4e572f1e2d607472cb87bd421ef4/src/core/manager.rs#L174-L191)
- [service 为 mihomo 设置 owner-scoped ACL](https://github.com/clash-verge-rev/clash-verge-service-ipc/blob/edf7cb57811d4e572f1e2d607472cb87bd421ef4/src/core/manager.rs#L724-L736)
- [service 核验 mihomo pipe server PID 并收紧 ACL](https://github.com/clash-verge-rev/clash-verge-service-ipc/blob/edf7cb57811d4e572f1e2d607472cb87bd421ef4/src/core/manager.rs#L990-L1093)
- [当前开发插件的 busy 无界重试](https://github.com/clash-verge-rev/tauri-plugin-mihomo/blob/d9398bf8e862c0cd613b79ada45cc5d893820ed6/src/stream.rs#L150-L171)

### Windows 官方文档

- [`ERROR_PIPE_BUSY` 与 `WaitNamedPipe` 客户端流程（固定提交）](https://github.com/MicrosoftDocs/win32/blob/376f699767763377725b2702e2904040a39f97b9/desktop-src/ipc/named-pipe-client.md#L11-L72)
- [named pipe 安全描述符、ACL 与访问检查（固定提交）](https://github.com/MicrosoftDocs/win32/blob/376f699767763377725b2702e2904040a39f97b9/desktop-src/ipc/named-pipe-security-and-access-rights.md#L11-L41)

## Caveats / Not Found

- 没有执行真实 `DELETE`，因此本机未验证关闭后的具体时序；方法、状态码和不存在 ID 的行为已由官方文档与固定源码核验。
- Clash Verge Rev `dev` 提交 `4c1804...` 尚未作为稳定版发布。动态管道设计可能继续变化，但已经足以否定「固定管道名是长期契约」。
- mihomo 官方文档没有定义 `chains` 顺序、连接数组顺序、连接 ID 的跨重启稳定性，也没有提供逐连接结束事件。
- named pipe 没有应用层只读权限或单独的监控 token。ACL 只决定能否连接，不能限制调用哪些 API。

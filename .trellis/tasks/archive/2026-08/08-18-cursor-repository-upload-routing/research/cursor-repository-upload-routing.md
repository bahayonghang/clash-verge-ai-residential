# Research: Cursor 仓库上传、索引与域名路由边界

- Query: Cursor 是否为代码库索引上传仓库内容；Privacy Mode 改变哪些传输、保留和训练行为；能否仅从 Clash 排除索引上传，同时保留 Chat、Tab、Agent、认证和 Cloud Agent。
- Scope: mixed。以 Cursor 官方文档、官方博客、官方 API 文档和本机 Cursor 官方发行包为主；未使用社区报告作为结论依据。
- Date: 2026-08-18

## Findings

### 结论摘要

1. **索引不是「只上传哈希」。** Cursor 会上传新代码库的文件或小代码块，由服务器计算 embeddings。客户端还会上传 Merkle tree、文件 SHA-256 哈希、simhash、文件名或路径元数据。官方材料明确写明，新代码库的索引流程会上传每个文件。[S1][S2]
2. **索引是客户端与服务器共同完成，不是纯本地索引。** 客户端计算 Merkle tree、差异和 simhash；服务器接收发生变化的文件、生成或复用 embeddings，并保存可查询索引。团队内的相似索引复用也在服务器侧完成。[S1][S2]
3. **Privacy Mode 不会关闭仓库上传或服务器索引。** Privacy Mode 的明确保证是客户数据不用于 Cursor 或模型提供商的训练，并对模型请求应用 Zero Data Retention（ZDR）合同；它不是「索引仅在本地运行」或「服务器不保留 embeddings/元数据」开关。[S1][S3][S4]
4. **官方唯一明确分配给代码库索引的主机是 `repo42.cursor.sh`，协议为 HTTP/2。** 官方网络文档把 `api2`、`api5`、`api3`、`api4`、`gcpp` 分别分配给通用 API、Agent/NAL 和 Tab，不应因排除索引而一并改路由。[S5]
5. **仅排除 `repo42.cursor.sh` 可以隔离默认索引主机，但不是跨版本绝对保证。** 本机 Cursor 3.16.17 的官方发行包显示：启用 `cursor.general.disableHttp2` 或服务器强制禁用 HTTP/2 时，RepositoryService 会把 `repo42.cursor.sh` 替换为 `api2.cursor.sh`，改用 HTTP/1.1。此时域名层无法同时「阻止索引」和「保留 `api2` 上的大多数 API」。
6. **排除索引不等于阻止所有代码离开本机。** Chat、Tab 和 Agent 仍可能通过各自 API 发送提示、当前代码上下文或编辑上下文；Cloud Agent 还会在 Cursor 基础设施中临时保存加密仓库副本。[S4][S6]

### 1. 索引实际上传和保存什么

#### 已确认的上传内容

| 数据 | 是否上传 | 依据 |
|---|---:|---|
| 文件内容或代码块 | 是 | Cursor Data Use 写明「upload your codebase in small chunks」；官方索引博客写明新代码库会「upload every file」。[S1][S2] |
| 文件 SHA-256 哈希、目录哈希、Merkle root/tree | 是 | 官方索引博客描述客户端与服务器比较 Merkle tree，并在相似索引复用时上传完整 Merkle tree。[S2] |
| simhash | 是 | 客户端从 Merkle tree 计算 simhash，上传给服务器以搜索同一用户或团队的相似索引。[S2] |
| 文件名或路径元数据 | 是 | Data Use 明确列出 metadata `(hashes, file names)`；当前 Search 文档说明路径在发送前加密、文件名被混淆。[S1][S7] |
| embeddings | 服务器生成并保存，不是客户端只上传既成 embeddings | Data Use 明确写明上传代码块到服务器「to compute embeddings」；官方网络文档说明服务器保存加密 vector database。[S1][S5] |
| 仅哈希 | 否 | 哈希用于增量比较和内容证明，但首次索引仍上传文件；发生变化的文件或块也会同步。[S2] |

#### 已确认的保留与删除

- **明文代码：** Data Use 说明，用于计算 embeddings 的所有明文代码在请求生命周期结束后不再存在。Search 文档进一步说明，明文代码只在索引期间保存在内存中，随后丢弃。[S1][S7]
- **embeddings 与元数据：** Data Use 说明，embeddings 以及哈希、文件名等元数据可能保存在 Cursor 数据库中。Network Configuration 将其描述为 encrypted vector database storage。[S1][S5]
- **加密代码块：** Search 文档说明代码块被加密；Agent 搜索时，Cursor 取回 embeddings，并在客户端解密代码块。该流程意味着服务器端索引至少能取回加密块，但公开文档没有给出加密块的精确保留期、删除触发器或备份清除期限。[S7]
- **相似索引内容证明：** 官方索引博客说明，复制团队索引时，服务器临时保存完整 Merkle tree 作为 content proofs；客户端与服务器 Merkle root 一致后，服务器删除这些 content proofs。[S2]
- **Cloud Agent 仓库：** 这是独立于本地索引的流程。Cursor 会在每个隔离虚拟机中临时保存加密仓库副本，并在 Agent 完成后删除。[S4]

因此，不能把 Cursor 索引描述为「上传哈希、代码始终在本地」。更准确的描述是：**客户端用哈希确定需要同步的文件，服务器接收代码内容以建立 embeddings；明文请求内容随后删除，索引派生物、元数据和可检索的加密块按当前服务合同保留。**

### 2. Privacy Mode 与非 Privacy Mode

本文把「普通模式」解释为 **Privacy Mode 关闭**。Cursor 官方没有把它命名为 Normal Mode。

| 行为 | Privacy Mode 开启 | Privacy Mode 关闭 |
|---|---|---|
| 为索引上传代码块 | 仍会上传；官方没有说明 Privacy Mode 会停用索引 | 仍会上传 |
| 明文索引内容 | 请求生命周期或索引内存阶段结束后丢弃 | 索引文档给出的明文生命周期同样适用；但其他代码数据可能被额外保存和用于改进产品 |
| embeddings、哈希、文件名元数据 | 仍可能保存在 Cursor 数据库；Privacy Mode 不是索引删除开关 | 仍可能保存 |
| Cursor/模型训练 | 不用于训练；模型提供商受 ZDR 合同约束 | Cursor 可能使用并保存 codebase data、prompts、editor actions、code snippets 等来改进功能和训练模型 |
| 模型提供商保留 | 通常为 ZDR；风险分类器触发滥用调查、显式启用非 ZDR 模型等有例外 | 部分推理提供商可能临时访问和保存输入/输出，使用后删除 |
| 路径密钥 | 当前公开网页只承诺路径加密/文件名混淆 | 本机 3.16.17 发行包显示，握手在 Privacy Mode 关闭时发送 `pathKey`，开启时发送空字符串；这是版本限定的客户端实现证据，不应替代线上合同 |

Privacy Mode 的官方保证集中在**训练、模型提供商保留和合同控制**。[S1][S3][S4] 若目标是禁止服务器建立代码库索引，应关闭 Cursor 的代码库索引功能或在网络层阻断索引服务；只启用 Privacy Mode 不满足该目标。

### 3. 官方主机职责与协议边界

| 主机或模式 | 官方明确职责 | 是否属于本地代码库索引 | 路由结论 |
|---|---|---:|---|
| `repo42.cursor.sh` | codebase indexing，HTTP/2 only | 是 | 默认索引隔离的唯一官方精确主机。[S5] |
| `api2.cursor.sh` | most API requests；官方提供 Connect health 测试 | 默认不是专属索引主机 | 保留 Chat/通用 API 需要此域；本机 3.16.17 可能把索引回退到这里。[S5] |
| `api5.cursor.sh`、`agent*.api5.cursor.sh` | Agent requests、Network Access Layer（NAL） | 否 | 排除索引时保留。[S5] |
| `api3.cursor.sh` | Cursor Tab，HTTP/2 only | 否 | 排除索引时保留。[S5] |
| `api4.cursor.sh`、`us-asia.gcpp.cursor.sh`、`us-eu.gcpp.cursor.sh`、`us-only.gcpp.cursor.sh` | 按地区提供 Cursor Tab，HTTP/2 only | 否 | `gcpp.cursor.sh` 不是官方索引域；排除索引时保留。[S5] |
| `authenticate.cursor.sh` | Authorization endpoint | 否 | 保留。[S5] |
| `authenticator.cursor.sh` | Auth UI/login webview | 否 | 保留。[S5] |
| `prod.authentication.cursor.sh`、`authentication.cursor.sh` | token/JWT issuer | 否 | 保留。[S5] |
| `api.cursor.com` | Cloud Agent REST API，例如 `/v1/agents` | 否 | 保留 Cloud Agent 时必须保留。[S6] |
| `*.cursorvm.com`、`*.*.cursorvm.com` | 官方要求为其禁用 SSL inspection；与 Cursor 托管虚拟机流量相关，但该网络页没有逐接口职责表 | 否 | 保留 Cloud Agent 时不要排除。[S5] |
| `marketplace.cursorapi.com`、`cursor-cdn.com`、`downloads.cursor.com`、`anysphere-binaries.s3.us-east-1.amazonaws.com` | 客户端更新和扩展市场下载 | 否 | 与索引无关，不应为了排除索引而改路由。[S5] |

#### Connect/gRPC 端点

- 官方网络文档公开了 `api2.cursor.sh` 的 Connect health 路径：
  - `POST https://api2.cursor.sh/aiserver.v1.HealthService/StreamSSE`
  - `POST https://api2.cursor.sh/aiserver.v1.HealthService/StreamBidi`
  - 示例 Content-Type 为 `application/connect+json`。[S5]
- 本机 Cursor 3.16.17 的 `cursor-retrieval` 官方发行包定义 `aiserver.v1.RepositoryService`，包括 `FastRepoInitHandshakeV2`、`SyncMerkleSubtreeV2`、`FastUpdateFileV2`、`FastRepoSyncComplete`、`SemSearchFast`、`RemoveRepositoryV2` 等方法。Connect URL 构造器使用 `/{service.typeName}/{method.name}`，因此当前版本的索引请求路径形如：
  - `https://repo42.cursor.sh/aiserver.v1.RepositoryService/FastRepoInitHandshakeV2`
  - `https://repo42.cursor.sh/aiserver.v1.RepositoryService/SyncMerkleSubtreeV2`
  - `https://repo42.cursor.sh/aiserver.v1.RepositoryService/FastUpdateFileV2`
  - `https://repo42.cursor.sh/aiserver.v1.RepositoryService/FastRepoSyncComplete`
- `FastUpdateFileV2Request.LocalFile` 包含 `file`、`hash` 和 `unencrypted_relative_workspace_path`；共享的 `aiserver.v1.File` 消息包含 `relative_workspace_path` 和 `contents`。这进一步证明当前实现会在 RepositoryService 请求体中直接发送文件内容，不是只发送哈希。
- 上述 RepositoryService 方法和路径来自本机 3.16.17 的压缩发行包，不是 Cursor 的公开稳定 API。版本升级可改变方法名、消息字段或后端主机。

#### Signed object-storage URL

- 官方网络文档只把 `anysphere-binaries.s3.us-east-1.amazonaws.com` 分配给更新/扩展下载，不是代码库索引。[S5]
- Cloud Agent API 的 artifact download 会返回 `cloud-agent-artifacts.s3.us-east-1.amazonaws.com` 的临时 URL；这是 Cloud Agent 产物下载，不是本地代码库索引上传。[S6]
- 当前 3.16.17 的 RepositoryService schema 直接在 `FastUpdateFile(V2)` 消息中携带文件内容；未发现索引服务的 presigned upload 字段。
- 因此，**没有第一方证据支持为本地代码库索引添加宽泛 `amazonaws.com`、`storage.googleapis.com`、Azure Blob 或其他对象存储规则。** 这只能写成「当前未发现」，不能写成「未来绝不会使用」。

### 4. Clash 能否只排除索引上传

#### 官方证据支持的最窄边界

在 HTTP/2 默认路径、当前官方职责表不变的条件下，Clash 可以只把精确主机 `repo42.cursor.sh` 送到非家宽策略，同时继续让以下流量沿现有 Cursor 核心策略运行：

- Chat/通用 API：`api2.cursor.sh`。
- Agent/NAL：`api5.cursor.sh` 及其 `agent*` 子域。
- Tab：`api3.cursor.sh`、`api4.cursor.sh`、三个 `gcpp.cursor.sh` 地区主机。
- 认证：四个官方认证主机。
- Cloud Agent：`api.cursor.com`、Cursor 网页入口和 `cursorvm.com` 相关主机。

这是**已确认的默认主机隔离**。官方只列出精确的 `repo42.cursor.sh`；没有官方材料证明索引主机一定按 `repo[0-9]+.cursor.sh` 滚动。仓库现有 `^repo[0-9]+\.cursor\.sh$` 正则属于前向兼容推断，不应被描述为官方合同。

#### 无法仅靠域名规则保证的情况

1. **HTTP/1.1 回退：** 本机 3.16.17 在 `cursor.general.disableHttp2=true` 或服务器配置强制禁用 HTTP/2 时，把 RepositoryService 从 `repo42` 改到 `api2`。阻断 `api2` 会同时破坏「most API requests」，不满足保留 Chat 的要求。
2. **服务端配置漂移：** 客户端从认证/配置响应读取 `repoBackendUrl`。后续版本可返回不同索引主机；只匹配 `repo42` 不会自动覆盖它。
3. **其他代码上传：** 即使 `repo42` 被排除，Chat/Agent 的 LLM 请求仍会发送提示和代码上下文。该规则只能排除 repository indexing，不是 DLP 或「任何仓库内容不得上传」策略。[S4]
4. **Cloud Agent：** Cloud Agent 从托管代码平台访问并临时保存仓库，不经过本地 `repo42` 索引通道。保留 Cloud Agent 与禁止任何远端仓库副本是互相冲突的需求。[S4]
5. **URL 路径不可用于常规 Clash 域名规则：** HTTPS 下 Clash 通常只能依据 DNS/SNI/目标 IP 和进程信息路由，不能依据加密后的 Connect 方法路径区分 `api2` 上的索引与 Chat。

因此，可实施结论是：

- **可以：** 排除默认的、官方专属的 `repo42.cursor.sh` 索引通道，并保留其他明确职责主机。
- **不能保证：** 在索引回退到 `api2`、后端主机由服务器配置改变或希望阻止 Agent 代码上下文时，仍只靠域名规则实现完整隔离。

### 5. 版本和日期歧义

- Cursor Data Use 最后更新于 **2026-07-15**；Cursor Security 最后更新于 **2026-04-24**。[S1][S3]
- Network Configuration、Search、Privacy and Data Governance 页面未显示稳定的页面版本号或最后更新时间。本报告访问日期为 **2026-08-18**。[S4][S5][S7]
- 官方索引博客发布并最后修改于 **2026-01-27**。它描述的新团队索引复用架构可能晚于旧版 Cursor 客户端。[S2]
- 本机官方发行包为 Cursor **3.16.17 stable**，commit `6b2afae0257df2bb5e1835f15165dc2f0de056b0`，构建时间 `2026-08-14T01:41:12Z`。发行包证据只能说明这一版本，不是未来兼容保证。
- Cursor 的公开 GitHub 仓库没有公开桌面客户端索引器源码。本报告引用的 RepositoryService 是本机官方发行包中的压缩 JavaScript，不应称为公开源码或稳定 API。

### 6. 关闭证据缺口所需的本机观测

使用不含真实秘密的临时仓库和唯一 canary 文件执行以下矩阵。不要在生产仓库上做 TLS 中间人解密。

1. 记录 Cursor 版本、Privacy Mode、代码库索引开关、`cursor.general.disableHttp2` 和团队策略。
2. 清空该临时仓库的既有索引状态，分别测试 Privacy Mode 开/关以及 HTTP/2 开/关。修改 canary 文件，触发首次索引和增量索引。
3. 保存 Cursor 的 `Cursor Indexing & Retrieval` 输出。重点核对 `Creating Indexing Repo client`、最终 backend URL、握手/同步错误和回退行为。
4. 保存 Clash Connections 和 DNS 日志，至少记录时间、进程、SNI/域名、目标 IP、策略组和传输协议。验证默认索引是否只访问 `repo42.cursor.sh`，以及禁用 HTTP/2 后是否访问 `api2.cursor.sh`。
5. 同时触发 Chat、Tab、Agent、登录和 Cloud Agent，证明排除规则没有让这些主机改走或失败。不要仅凭单次「功能可用」推断所有子功能均未受影响。
6. 监视是否出现新的 Cursor 主机或对象存储主机。若出现 signed URL，保存已脱敏的主机名、证书 SAN、DNS CNAME 和触发动作；不要保存查询参数、令牌或仓库内容。
7. 若必须确认 Connect 方法路径，只能在隔离测试机、虚构仓库和明确授权下进行 TLS 观测。Cursor 官方建议对 Cursor 域禁用 SSL inspection，并说明部分关键服务使用 certificate pinning；普通 Clash 日志无法证明请求体中具体发送了哪些字段。[S5]

满足以下条件后，才能把 `repo42` 排除写成当前环境的「已验证」结论：

- 默认 HTTP/2 索引只命中 `repo42.cursor.sh`。
- Privacy Mode 两种状态均未改变索引主机到其他域。
- `cursor.general.disableHttp2=false`，并且服务端没有强制 HTTP/1.1。
- 阻断或改路由 `repo42` 后，Chat、Tab、Agent、认证和 Cloud Agent 的独立探针均成功。
- Clash/DNS 日志未观察到索引触发的对象存储或新 `repo*` 主机。

## Files Found

- `clash-verge-ai-residential.js:252`：仓库当前 Cursor 后缀域列表；`repo` 主机由后续正则处理。
- `clash-verge-ai-residential.js:275`：当前使用 `^repo[0-9]+\.cursor\.sh$` 路由索引候选主机。
- `tests/regression.test.js:591`：Cursor 当前正向路由测试，包含 `repo42.cursor.sh` 和 `repo99.cursor.sh`。
- `C:\Users\lyh\AppData\Local\Programs\cursor\resources\app\package.json:3`：本机官方 Cursor 发行包版本 `3.16.17`。
- `C:\Users\lyh\AppData\Local\Programs\cursor\resources\app\product.json`：本机发行包 commit、构建时间和固定服务 URL。
- `C:\Users\lyh\AppData\Local\Programs\cursor\resources\app\extensions\cursor-retrieval\dist\main.js:1`：压缩的官方索引客户端；包含 RepositoryService、Connect URL 构造、文件消息、Privacy Mode 路径密钥逻辑和 `repo42 -> api2` HTTP/1.1 回退。

## Code Patterns

- `clash-verge-ai-residential.js:276`：现有注释假设索引端点可能滚动编号；本次研究只确认官方精确主机 `repo42.cursor.sh`，未确认编号滚动合同。
- `tests/regression.test.js:610`：测试把 `repo99.cursor.sh` 视为预期路由；这是项目策略，不是 Cursor 第一方网络职责证据。
- `cursor-retrieval/dist/main.js:1`：`FastUpdateFileV2Request.LocalFile -> aiserver.v1.File -> contents`，证明当前发行包的增量索引 RPC 可携带文件内容。
- `cursor-retrieval/dist/main.js:1`：`cursor.general.disableHttp2` 为真时，`repo42.cursor` 被替换成 `api2.cursor`，HTTP 版本从 `2` 改为 `1.1`。
- `cursor-retrieval/dist/main.js:1`：握手字段 `pathKey` 在 Privacy Mode 关闭时发送路径加密密钥，开启时发送空字符串；该实现可能随版本变化。

## External References

- [S1] [Cursor, Data Use & Privacy Overview](https://cursor.com/data-use)，最后更新 2026-07-15，访问 2026-08-18。直接证据：Privacy Mode/ZDR、非 Privacy Mode 的训练与保存、索引上传小代码块、明文请求生命周期、embeddings 和 `(hashes, file names)` 元数据保存。
- [S2] [Cursor, Securely indexing large codebases](https://cursor.com/blog/secure-codebase-indexing)，发布/修改 2026-01-27，访问 2026-08-18。直接证据：Merkle tree、SHA-256、增量文件同步、首次上传全部文件、simhash、服务器相似索引和 content proofs 删除条件。
- [S3] [Cursor, Security](https://cursor.com/security)，最后更新 2026-04-24，访问 2026-08-18。直接证据：Privacy Mode 不用于训练、模型提供商技术和合同控制、Cursor 后端承担 API/indexing/update/marketplace。
- [S4] [Cursor Docs, Privacy and Data Governance](https://cursor.com/docs/enterprise/privacy-and-data-governance)，访问 2026-08-18。直接证据：LLM 请求发送 prompts/code context；Cloud Agent 是需要存储代码的独立功能；临时加密仓库副本及完成后删除；CMEK 可加密 embeddings。
- [S5] [Cursor Docs, Network Configuration](https://cursor.com/docs/enterprise/network-configuration)，访问 2026-08-18。直接证据：逐主机职责、`repo42` 索引、HTTP/2/HTTP/1.1、Connect health 路径、SSL inspection、加密 vector database、更新/市场 S3 域。
- [S6] [Cursor Docs, Cloud Agent API Endpoints](https://cursor.com/docs/cloud-agent/api/endpoints)，访问 2026-08-18。直接证据：Cloud Agent API 基址 `https://api.cursor.com/v1/agents`，artifact download 返回独立 S3 临时 URL。
- [S7] [Cursor Docs, Agent Search](https://cursor.com/docs/agent/tools/search)，访问 2026-08-18。直接证据：路径发送前加密；明文代码仅在索引内存中存在；filenames obfuscated、code chunks encrypted；服务器取回 embeddings、客户端解密代码块。

## Related Specs

- `.trellis/spec/frontend/index.md`：修改路由域前读取根脚本和对应正向、负向、托管规则及幂等测试。
- `.trellis/spec/frontend/quality-guidelines.md`：新路由域必须有官方或脱敏 Connections 证据，并为相邻共享流量添加负向覆盖；不能用 Node 测试替代真实 Clash 配置观测。
- `CLAUDE.md`：AI-only 路由边界；宽泛 provider suffix、市场/CDN、更新和遥测默认不纳入家宽路由。

## Caveats / Not Found

- 未找到 Cursor 第一方公开文档把 `repo[0-9]+.cursor.sh` 定义为稳定通配模式；只确认 `repo42.cursor.sh`。
- 未找到 Cursor 第一方公开文档为代码库索引分配 `gcpp.cursor.sh`、`api5.cursor.sh`、`api3.cursor.sh`、`api4.cursor.sh`、`api.cursor.com` 或 signed object-storage URL。官方职责表把这些主机分配给其他功能。
- 未找到公开的 embeddings、加密代码块、哈希和文件名元数据的统一删除期限。明文代码删除时点与相似索引 content proofs 删除条件有明确说明，但不能外推为全部索引派生数据的删除期限。
- 未进行本机 Cursor 索引触发、Clash Connections 捕获或 TLS 解密。本报告的主机隔离结论是文档与静态发行包证据，不是当前网络会话的实测证明。
- Privacy Mode 文档集中说明训练和 ZDR；不要把它扩展解释成「Cursor 不接收代码」「不保存 embeddings」或「Cloud Agent 不保存仓库」。

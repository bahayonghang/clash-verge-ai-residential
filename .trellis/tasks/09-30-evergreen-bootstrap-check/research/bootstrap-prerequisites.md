# Research: T04 bootstrap prerequisites

- Query: 是否已有与项目 Trellis 版本匹配的本机 executable 或可信源码入口；固定版本包是否存在；后续隔离 bootstrap 需要哪些前提。
- Scope: mixed；只读本机已知 wrapper、安装包、项目引用及 npm registry 固定版本元数据。
- Date: 2026-09-30
- Active task: .trellis/tasks/09-30-evergreen-bootstrap-check。
- Write scope: 本研究文件。未安装包、运行 init/update、修改 PATH/trust、编译或运行客户端推理。

## Findings

### 1. 当前入口与项目版本不匹配

项目要求 Trellis 0.7.0-beta.3，来源为 .trellis/.version:1。当前 PATH 的包装器指向全局 npm 包 @mindfoldhq/trellis@0.6.17。在本次允许检查的 wrapper、安装包和项目历史引用范围内，未找到可直接执行的 0.7.0-beta.3 本机入口。

当前 bootstrap 前提保持 **BLOCKED**：还缺少已获准使用且实际核验为匹配版本的 executable。registry 上存在匹配包，提供了后续隔离获取的候选方案；本研究没有获取或安装该包，不能据此宣称 AC3/AC4 通过。

Get-Command trellis -All 返回：

| 类型 | 路径 |
|---|---|
| ExternalScript | C:/home/lyh/.npm-global/trellis.ps1 |
| Application | C:/home/lyh/.npm-global/trellis.cmd |
| Application | C:/home/lyh/.npm-global/trellis |

PowerShell 包装器采用以下路径：

- C:/home/lyh/.npm-global/trellis.ps1:11-16：优先使用包装器同目录的 node.exe，执行同目录 node_modules/@mindfoldhq/trellis/bin/trellis.js。
- C:/home/lyh/.npm-global/trellis.ps1:19-24：该 Node 文件不存在时，使用 PATH 上的 node.exe，脚本目标不变。
- C:/home/lyh/.npm-global/node_modules/@mindfoldhq/trellis/bin/trellis.js:3：加载 ../dist/cli/index.js。
- C:/home/lyh/.npm-global/node_modules/@mindfoldhq/trellis/package.json:2-3：包名为 @mindfoldhq/trellis，版本为 0.6.17。
- 同一 package.json:8-10：trellis 与 tl 都指向 ./bin/trellis.js。
- 同一 package.json:37：依赖 @mindfoldhq/trellis-core，固定版本 0.6.17。
- 同一 package.json:53,61-63：Node 要求 >=18.17.0，仓库为 https://github.com/mindfold-ai/trellis.git。

安装目录的 Get-Item 结果未显示 LinkType/Target，没有发现该路径作为源码 checkout 的链接证据。全局兄弟路径 C:/home/lyh/.npm-global/node_modules/@mindfoldhq/trellis-core/package.json 不存在；依赖也可能安装在包内，本研究没有由此推断 CLI 损坏。

父任务原审计已记录 trellis --version 退出码 0、版本 0.6.17，见 ../09-30-evergreen-five-harness-audit/research/audit.md 的版本调查。本研究回读了安装包与包装器，没有重复启动 CLI。当前证据支持“版本不匹配”；未查明为何先前安装的 0.7.0-beta.3 变为当前 0.6.17。

### 2. 历史 bootstrap 记录未保留 executable 路径

.trellis/tasks/archive/2026-09/09-07-evergreen-harness-adapters/research/harness-smoke.md:23 记录 2026-09-07 的 CLI 与项目版本均为 0.7.0-beta.3。该文件 :27-31 保存候选目录、日志路径及命令；:33-47 保存四个 override 的跳过信息及 SHA-256；:51-61 保存五平台标记文件与上下文加载结果。

该历史记录使用命令名 trellis，没有保存当时的可执行程序绝对路径、npm 安装目录或源码入口。本研究未读取历史临时目录，也未检查其他项目或全盘搜索。2026-09-07 的通过记录不能覆盖 2026-09-30 的版本不匹配。

历史记录 :63-76 还说明候选目录的 .template-hashes.json 曾按平台集合重写，跳过的本地 override 从候选 manifest 消失；当前仓库的 manifest 当时未改。后续不得将候选 manifest 拷回当前工作树。

### 3. Registry 提供精确匹配的 CLI/core 包

本轮通过 PowerShell Invoke-RestMethod 查询以下固定版本元数据，两次读取均成功。仅读取 JSON，未下载 tarball、执行 npm、安装依赖或写入 npm cache。

| 查询 URL | 返回包名 | 返回版本 | Node 要求 |
|---|---|---|---|
| https://registry.npmjs.org/@mindfoldhq%2ftrellis/0.7.0-beta.3 | @mindfoldhq/trellis | 0.7.0-beta.3 | >=18.17.0 |
| https://registry.npmjs.org/@mindfoldhq%2ftrellis-core/0.7.0-beta.3 | @mindfoldhq/trellis-core | 0.7.0-beta.3 | >=18.17.0 |

CLI 的 bin 为 trellis / tl → bin/trellis.js；CLI 对 core 的依赖精确固定为 0.7.0-beta.3。两份元数据的 repository 均指向 git+https://github.com/mindfold-ai/trellis.git，未返回 gitHead 值。

CLI 包元数据：

~~~text
name: @mindfoldhq/trellis
version: 0.7.0-beta.3
tarball: https://registry.npmjs.org/@mindfoldhq/trellis/-/trellis-0.7.0-beta.3.tgz
shasum: f7ca77abc22b378aa6ef6c203d32fe43e658edef
integrity: sha512-jwdY2ArPxNE6wYRw0Gp5kiB7eGZY3ZiqXZU9KkxV89IaWnQd3ZU1MzgFZCLurwYdio9joksTuSL+hK3zzlbLhw==
fileCount: 807
unpackedSize: 3282068 bytes
~~~

Core 包元数据：

~~~text
name: @mindfoldhq/trellis-core
version: 0.7.0-beta.3
tarball: https://registry.npmjs.org/@mindfoldhq/trellis-core/-/trellis-core-0.7.0-beta.3.tgz
shasum: d349482bd51a4544ae22cd94f4617b0a790739f3
integrity: sha512-Xm2XgAcDVQ/GhNO407cgLJZMHaqjNXu9PSzGHzbVJmmp1BpXe+wJfCJ9McIkpT/V7ybty5eOjorIFksFgTi4vA==
fileCount: 218
unpackedSize: 658580 bytes
~~~

CLI 的其他依赖为 zod ^4.4.2、chalk ^5.3.0、giget ^3.1.1、figlet ^1.9.4、undici ^6.21.0、inquirer ^9.3.7、commander ^12.1.0。顶层 CLI/core 固定版本没有锁定全部传递依赖。实际获取后需要保留解析版本、依赖记录与实际 executable 路径。

Registry 返回了签名/provenance 元数据；本研究未验证签名或下载内容的 integrity，也没有检查该 tarball 内的实现。元数据查询只确认固定版本发布项存在。

### 4. 后续候选命令与授权前提

以下命令为后续执行建议，**本轮未运行**。T04 prd.md:31 排除默认安装授权，design.md:5 允许显式已安装入口或遵守授权边界的隔离固定版本执行器。主会话应先解决获取/安装的授权与入口选择，再运行命令。

工作目录必须是 T04 准备的新隔离目录，其中已放入规定的项目合同和四个 tracked override，已记录初始 SHA-256。不要将当前仓库作为 init 的工作目录。

~~~powershell
npm exec --yes --package=@mindfoldhq/trellis@0.7.0-beta.3 -- trellis --version
npm exec --yes --package=@mindfoldhq/trellis@0.7.0-beta.3 -- trellis init --claude --codex --grok --kimi --omp --skip-existing -y
~~~

第一条必须成功并返回与 .trellis/.version 相同的版本，才能进入第二条。版本不符、入口损坏或进程失败均应停止，不可用旧全局入口补跑。

npm exec 在缺少目标包时会获取依赖并安装到 npm cache。--yes 会自动同意 npm 的安装提示。该方式因此包含工具获取行为，不能称为纯只读检查。本研究没有执行该方式。

若需要精确控制 executable，可在后续获准获取时将包及依赖放入单独目录，记录该目录与 lock/依赖解析结果，再使用 `node <isolated-package>/bin/trellis.js --version`。只有核对版本、入口与目录后才从候选工作目录执行同一路径的 init。本研究未创建该包目录，尖括号部分是待确定路径。

后续命令不使用无版本的 npx trellis，不使用 latest，不升级全局包，不修改 PATH，不改变 trust。trellis 命令名与 npm 包名 @mindfoldhq/trellis 必须分别记录。

### 5. 后续 bootstrap 的证据清单

适用工具：Claude Code、Codex、Grok Build、Kimi Code、OMP。下表为 T04 后续验收所需证据，本研究未执行验收。

| 检查项 | 必须保存的证据 |
|---|---|
| executable 与版本 | 选择的绝对路径、包装器目标、--version 完整输出、退出码、与 .trellis/.version 的比较 |
| 获取记录 | 包名、精确版本、实际依赖版本/lock、获取目录；按使用方式记录 cache；不得声称全依赖已锁定而无记录 |
| 四个 override | .codex/config.toml、.kimi-code/skills/trellis-implement/SKILL.md、.kimi-code/skills/trellis-check/SKILL.md、.kimi-code/skills/trellis-research/SKILL.md 在候选 init 前后的 SHA-256 |
| 隔离 init | 新候选目录、完整命令、完整日志、退出码；版本门通过后才运行 |
| 五平台必要资产 | Claude .claude/agents/trellis-implement.md；Codex .codex/hooks.json；Grok .grok/agents/trellis-implement.md；Kimi .kimi-code/skills/trellis-start/SKILL.md；OMP .omp/agents/trellis-implement.md；继续按当前合同检查适用资产 |
| 当前仓库不变范围 | .trellis/.template-hashes.json、全局配置、项目 trust 与私有文件均未被本次执行改变；检查不得输出秘密 |
| 候选 manifest | 明确记录候选与当前工作树的范围；不得手工修改 hash 以隐藏本地 override，也不得拷回候选 manifest |
| 忽略范围与业务 skill | 由主会话执行已批准 T04 检查；bootstrap 不能替代 T02 业务 skill 安装/校验 |
| 动态能力 | 静态资产与隔离 init 的通过不代表五客户端 fresh-session、hook、权限或委派通过 |

Get-Command 路径发现、包元数据读取、--help、registry 元数据和历史通过记录，分别只能证明其直接检查的范围。缺少当前匹配入口和实际隔离 init 时，AC3/AC4 仍未验证。

## Files Found

| 文件 | 用途与证据位置 |
|---|---|
| .trellis/.version:1 | 项目版本来源；0.7.0-beta.3 |
| .trellis/tasks/09-30-evergreen-bootstrap-check/prd.md:19-31 | AC1–AC5、依赖、授权边界 |
| .trellis/tasks/09-30-evergreen-bootstrap-check/design.md:5-25 | 只读环境探针、入口方案、文件范围与回退边界 |
| .trellis/tasks/09-30-evergreen-bootstrap-check/implement.md:7-24 | 路径/版本检查先行、隔离 init 与正式检查 |
| .trellis/workflow.md:97-116 | 五平台入口导航与只读审查授权边界 |
| .trellis/spec/guides/index.md:68-80 | 审查结论须核验数据来源与代码证据 |
| .agents/skills/trellis-meta/SKILL.md | Trellis 本地架构研究入口；本轮已加载 |
| .agents/skills/trellis-meta/references/local-architecture/overview.md:3,23-34 | 全局 npm CLI 与项目生成资产的区分；manifest 用途 |
| docs/agents/harnesses.md:38-55 | 既有 bootstrap 命令、四 override 与 manifest 边界；文件由主会话按 T03/T04 顺序更新，行号可能变化 |
| .trellis/tasks/archive/2026-09/09-07-evergreen-harness-adapters/research/harness-smoke.md:23-76 | 历史成功记录与候选路径；未记录 executable 绝对路径 |
| C:/home/lyh/.npm-global/trellis.ps1:11-24 | 当前 PowerShell 包装器目标与 Node 选择 |
| C:/home/lyh/.npm-global/node_modules/@mindfoldhq/trellis/bin/trellis.js:3 | CLI 模块入口 |
| C:/home/lyh/.npm-global/node_modules/@mindfoldhq/trellis/package.json:2-10,37,53,61-63 | 当前包名/版本、bin、core、Node 与仓库元数据 |

## Related Specs

- .trellis/workflow.md:103-116：只读研究仅写明确获准的研究产物；工具能力与任务状态不扩大授权。
- .trellis/spec/guides/index.md:68-80：审查发现需要核验；不将缺少一个兄弟依赖路径推断为损坏。
- T04 prd.md:19-23：版本一致性门、四文件 SHA-256、五平台资产与不可替代的隔离 init 证据。
- docs/agents/harnesses.md：适用五工具的 bootstrap 与 fallback 合同；同步修改属于主会话的 T03/T04 范围。

## External References

- npm Registry @mindfoldhq/trellis@0.7.0-beta.3 固定版本 JSON：https://registry.npmjs.org/@mindfoldhq%2ftrellis/0.7.0-beta.3，读取日 2026-09-30。
- npm Registry @mindfoldhq/trellis-core@0.7.0-beta.3 固定版本 JSON：https://registry.npmjs.org/@mindfoldhq%2ftrellis-core/0.7.0-beta.3，读取日 2026-09-30。
- 包元数据声明的源码仓库：https://github.com/mindfold-ai/trellis.git；本研究未 clone 或检查该仓库，未将仓库 URL 视作已核验的本机源码入口。

## Caveats / Not Found

- 没有在允许检查的已知 wrapper、包与项目引用范围内找到匹配的本机 executable 或源码 checkout。不据此判断整台机器不存在匹配入口。
- 未检查其他项目、历史临时目录、全局秘密、真实数据库或私有配置内容。
- 本研究没有执行 npm 安装、cache 安装、init/update、全局 upgrade、PATH/trust 改动、编译、压测或客户端推理。
- 未读取本任务 implement.jsonl / check.jsonl，未执行 Git 操作；只写本研究文件。
- 未跑 T04 新探针及 fixture 测试，未验证当前版本的五平台生成资产或四 override 保留行为。当前已有通过证据来自历史记录，不能代替此次验收。
- 未验证 registry 签名、tarball integrity 或完整传递依赖。版本发布项存在不等于供应链验证通过。
- 项目实施已经获得用户批准；本研究派发另行限制工具获取与 init。后续获取行为由主会话按 T04 的明确授权边界处理，研究文件没有扩大权限。

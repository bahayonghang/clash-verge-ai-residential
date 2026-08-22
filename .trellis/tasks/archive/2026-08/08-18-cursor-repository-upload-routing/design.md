# Design: Cursor 仓库上传家宽排除优化

## Problem Boundary

Cursor 代码库索引会把文件或代码块发送到服务器建立 embeddings。官方网络配置和本机 3.16.17
日志均确认默认 HTTP/2 索引后端为 `repo42.cursor.sh`；当前脚本却把 repo 正则与 Chat、Tab、Agent、
认证和 Cloud Agent 一起受 `routing.cursor_core=true` 控制。目标是只改变可明确识别的仓库索引主机
是否经过 `AI-家宽`，不是阻止 Cursor 上传，也不是数据防泄漏方案。

## Routing Contract

新增两个单一职责常量：

```js
const ROUTE_CURSOR_REPOSITORY_INDEXING = false;
const CURSOR_REPOSITORY_INDEXING_DOMAIN_REGEXES = [
  "^repo[0-9]+\\.cursor\\.sh$"
];
```

现有 Cursor 核心正则目录只保留 `^adminportal[0-9]+\\.cursor\\.sh$`。实现时可将其重命名为
`CURSOR_CORE_DOMAIN_REGEXES`，避免继续用一个含义模糊的集合名。

活动规则矩阵：

| `cursor_core` | `cursor_repository_indexing` | 结果 |
| ---: | ---: | --- |
| `true` | `false` | Chat/Tab/Agent/认证/Cloud Agent 走家宽，repo 索引回落原 Profile |
| `true` | `true` | 恢复 v5.8.1 的 Cursor 路由行为 |
| `false` | `false` | Cursor 核心和 repo 索引均回落原 Profile |
| `false` | `true` | 仅 repo 索引走家宽，其他 Cursor 核心回落原 Profile |

`activeDomainRegexes()` 分别根据两个开关注入核心和索引正则。`allPossibleDomainRegexes()` 无条件包含
两组正则，使关闭新开关后的下一次运行可以清理旧的脚本托管 repo 规则。规则继续位于私有网段
DIRECT 规则之后、进程兜底之前；默认关闭的 Cursor 进程兜底保持不变。

正则域目前不会生成 `nameserver-policy` 键，因此拆分只影响规则注入与托管规则清理，不引入新的
DNS policy 机制。不得添加 `cursor.sh`、`cursor.com`、对象存储或共享 `api2.cursor.sh` 的宽泛排除。

## Local Configuration Contract

`scripts/sync-local-config.js` 的 `SWITCH_CONFIG_FIELDS` 新增：

```text
routing.cursor_repository_indexing -> ROUTE_CURSOR_REPOSITORY_INDEXING
```

公开示例默认 `false`。已有 `.local.toml` 缺字段时，现有原子补全机制写入 `false`，保留已有值、
注释、BOM、行尾和尾换行；随后生成的 `.local.js` 才应用新常量。非法类型、重复字段、常量锚点
缺失或重复继续 fail-closed，不写半成品。

这是一项有意的默认行为变化：既有用户下一次 render 后 repo 索引不再走家宽。用户显式设置
`cursor_repository_indexing=true` 可恢复旧行为，回滚不需要删除字段。

## Evidence And Compatibility Limits

- 官方精确合同只列出 `repo42.cursor.sh`；保留现有数字正则是项目的前向兼容策略，不应写成官方保证。
- 本机 2026-08-17 实际使用 `repo42` HTTP/2，且未启用 `cursor.general.disableHttp2`，因此默认路径可拆分。
- Cursor 3.16.17 在禁用 HTTP/2 或服务端强制回退时可把 RepositoryService 放到共享
  `api2.cursor.sh`。Clash 域名规则无法在该主机上区分索引与多数 API；设计选择保留 `api2`，并把
  该模式标记为无法完整隔离。
- Privacy Mode 影响训练和数据保留合同，不会关闭索引上传。Chat、Agent 和 Cloud Agent 仍可能
  发送或保存代码上下文；本开关只覆盖 repository indexing 域名。
- 当前未发现索引使用 signed object-storage URL；不能由此保证未来版本绝不使用新主机。

## Managed State And Migration

脚本只清理当前版本能够生成的托管规则。repo 正则从核心目录拆出后仍必须进入
`buildManagedRuleSet()`，否则关闭默认值无法移除 v5.8.1 已注入的规则。未知 `AI-家宽` 规则和文档中
已退役、现视为用户所有的旧 Cursor exact/regex 规则继续保留。

版本属于新增用户配置并改变默认路由行为，实施时按仓库现有版本同步约定升级次版本号，并同步
脚本、`package.json`、README 与 CHANGELOG。不得编辑或提交 `.local.toml`、`.local.js` 或 Clash
运行时生成 Profile。

## Validation Strategy

1. 单元层验证四种开关组合、正向与负向域名、规则顺序、清理幂等和未知规则保留。
2. 渲染层验证默认补全为 `false`、显式 `true/false`、非法输入与锚点异常的原子失败。
3. 文档层搜索所有旧的“Cursor 索引默认走家宽”表述并同步边界。
4. 运行 `just ci`；Node 测试不能证明真实 Mihomo/Clash host 行为。
5. 实施后如进行本机验证，只使用脱敏 Profile/Connections，确认 `repo42` 回落原 Profile，
   `api2/api3/api4/api5/gcpp` 仍走 `AI-家宽`。未做真实连接观测时标记为 `UNVERIFIED`。

## Rollback

配置级回滚：设置 `routing.cursor_repository_indexing = true` 并重新生成本地脚本，即恢复旧 repo 路由。
代码级回滚应同时还原常量、目录拆分、渲染字段、测试、文档和版本元数据；不能只删开关而留下
托管规则清理缺口。

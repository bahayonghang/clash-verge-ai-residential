# FlClash null 输入兼容：实施与回归证据

日期：2026-10-19（America/New_York）。用户已批准实施和验证，明确要求不提交。本文件只记录实现角色负责的公开模板和自动化验证；完整 `just ci`、spec 同步及本机 FlClash 操作由主会话独立记录。

## 修改范围与机制

- `clash-verge-ai-residential.js`：`cloneConfigForEdit` 构造副本后，以严格 `=== null` 判断并删除副本的顶层 `rule-providers`；新增中文原因注释。产品源码共增加 3 行。
- `tests/regression.test.js`：新增四组 Vertex/Cursor null 正例；测试辅助函数在未提供 profileName 时真正只传一个参数。每组覆盖单/双参数入口、与缺省输入完整深等、原节点/组/规则保留、家宽链、payload/policy、成功输入不变和幂等。类型负例移除的仅为顶层 null，新增空串、0 和两种布尔值；同名 provider 内部 null、非规范类型/字段/payload 负例全部保留。13 种冲突引用分别在顶层 null/规范映射、开关全开/全关时拒绝，且检查失败输入不变及错误不泄露虚构凭据。
- `tests/sync-local-config.test.js`：保留原有五组开关/default 场景，子进程改为 null 输入与单参数 main；检查输入不变、家宽节点/组生成、生成规则及 DNS 结果。运行生成脚本前后比较临时 TOML 内容，公开模板仍逐字不变。
- `CHANGELOG.md`：未发布区增加中文兼容修复说明，不升版本、不声明完整 FlClash 支持。

`isPlainObject`、`validateRegexDnsProviderOwnership`、builder 和主流程均未修改。没有依赖/锁文件/路由域名变化，没有读取或手改真实 local TOML/JS，没有操作 FlClash 或提交/推送。

## 先红后绿的独立模板

产品修复前将公开模板复制到系统临时目录。该目录不含 local 脚本或真实配置，测试仅使用虚构 `.example.test` 节点。

| 模板 | SHA-256 |
| --- | --- |
| 修复前公开模板快照 | `cc6d7f85376c21bab02924b6d0ff34451f505751263d158df81969fdfdf116ba` |
| 修复后公开模板 | `467e8f80fbfbd09b5341b5616011c993316f58d4970532ac393c2e93bf8031c6` |

旧快照位于系统临时目录 `flclash-null-before-fdc2a1649eef4ce6b8e66bea72f6da75/script.js`。在独立 PowerShell 测试进程内设置 `CLASH_SCRIPT_PATH` 指向该文件，结束时用 `finally` 恢复原环境值，没有 reset/stash/revert 工作区。

```powershell
node --test --test-name-pattern='FlClash null 规则集' tests/regression.test.js
```

| 阶段 | 结果 |
| --- | --- |
| 修复前，旧快照 + 新正例 | RED：exit 1，4 tests / 0 pass / 4 fail，144.4787 ms |
| 修复后，工作区模板 + 同一正例（包含于定向 16 项） | GREEN：exit 0，4 个 null 正例全部通过 |
| 修复后，再以旧快照运行同一正例 | RED：exit 1，4 tests / 0 pass / 4 fail，149.1711 ms |

两个 RED 的四项失败均为 `[AI-家宽] rule-providers 必须是对象，拒绝覆盖已有配置`，栈位于旧 `validateRegexDnsProviderOwnership`。未以路径、占位凭据或上游失败充当红测；旧快照 hash 在复测后未变。

## 已运行检查

环境为 Windows PowerShell、Node.js `v26.7.0`。没有声称运行 Node 18/20/22 矩阵或 FlClash 的 QuickJS 引擎。

| 命令 | 结果 |
| --- | --- |
| `node --test --test-name-pattern='FlClash null 规则集\|正则住宅 DNS' tests/regression.test.js` | PASS，exit 0，16 tests / 16 pass，264.7794 ms |
| `node --test tests/sync-local-config.test.js` | PASS，exit 0，26 tests / 26 pass，1965.4918 ms（随后又补家宽生成断言，最终根 CI 覆盖最新版） |
| `npm run ci` 首轮 | PASS，exit 0，259 tests / 259 pass；语法、agent contract 与 secret scan 全部成功 |
| `npm run ci` 最终复核 | PASS，exit 0，259 tests / 259 pass，测试阶段 6366.1446 ms；包含新增家宽生成断言，语法、agent contract 与 secret scan 成功 |
| `git diff --check` | PASS，exit 0 |
| `just ci` | 主会话负责，不用上述 root gate 替代 |

渲染器既有合同会在 sync 时补全临时 TOML 的缺失开关。因此 TOML 不变的快照取在该既有补全之后，证明执行生成脚本不会再次改写它；没有为这次兼容修改渲染器行为。五种输入仍包含缺省开关场景。

## 限制与交接

- F-AC1–F-AC5 的代码行为已由上述正/负用例覆盖；F-AC6 的 root gate 与红绿验证已执行，完整产品门等待主会话结果。
- F-AC7 本角色 `NOT_RUN`：未操作本机 FlClash，没有把 Node 测试视为实际配置应用、住宅连通、公网 DoH 或出口验证。
- 历史 `research/repro.cjs` 保留旧拒绝合同，修复后不是绿测入口，未改写为修复证据。
- 本次没有提交或推送。主会话继续 spec、独立 check、完整门禁和客户端授权范围内操作。

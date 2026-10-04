# T02 独立验收审查

审查日期：2026-09-30。范围：`.trellis/tasks/09-30-evergreen-skill-sync`。用户已批准项目内同步与备份。审查代理为已派发的 `trellis-check`，未再次委派。

## 结论

T02 AC1–AC4 通过本地验收。未发现需要修正的 T02 产品问题。审查只新增本文件和 `review-snapshot.json`；没有修改产品文件、安装器或已投递副本。产品文件所有权已释放给主会话。

## 验收矩阵

| 条款 | 结论 | 独立核对与收据 |
| --- | --- | --- |
| AC1：审查旧副本差异、备份和额外文件 | PASS | 读取 `before-diffs.diff`，对照 `before-inventory.json` 和当前文件重新计算 SHA256。21 个替换前文件分别对应 21 个新 `.bak-20260930T130351Z`；哈希全部相同。30 个原有额外文件全部保留且哈希未变。 |
| AC2：实际七目标一致、检查成功、幂等 | PASS | 七目标为 `.agents`、`.claude`、`.codex`、`.cursor`、`.omp`、`.grok`、`.kimi-code`。当前 21 个业务文件逐字节匹配单一源。实际目录共 72 个文件，正好为 21 个业务文件、21 个新备份、30 个旧额外备份。`delivery-commands.json` 记录限定七目标的检查退出 0 和二次安装写入 0。 |
| AC3：真实 payload 与映射 | PASS | `tests/install-agent-skills.test.js` 的真实七目标测试逐一 `require` 临时安装目录的生成器，并读取当前公开模板。各目标均断言 routing 26、supported 13、unsupported 13；`extra_anyrouter` 仅映射 `anyrouter.top`，且不进入 unsupported；`extra` 保持 unsupported。`actual-generators.log` 对实际七目录给出同样结果。 |
| AC4：交付与运行证据分层 | PASS | 单一源 `SKILL.md` 和 `docs/agents/residential-rule-tuning.md` 写明适用五工具及 `.agents`/`.cursor`。说明区分临时 fixture、实际副本一致、客户端发现与执行，并明确零目标检查成功不能证明交付。 |

## 合同与范围

- T03 的授权规则仍在单一源及全部七份副本中：默认输出本地 TOML 调整建议；自动编辑需要用户明确例外授权和范围；生成的本地 JavaScript 不得手改；只读审查不能扩大 self-fix 权限。
- `scripts/install-agent-skills.js` 没有 diff。继续使用原安装器的冲突拒绝、逐文件备份、额外文件保留与幂等逻辑。没有新增依赖或替代安装路径。
- `before-inventory.json` 的源 SKILL 哈希先于 T02 交付说明修改；交付时的最终源哈希单独保存在 `delivery-source.json`。当前 SKILL 哈希为 `b644d3db0a4947b4e01a7bcb341ec02d104c89cfcd43454229ca9485471dfcaa`，与交付和正式检查后的收据一致。
- 对七个业务 skill 目录运行 `git ls-files` 返回空，副本及备份未进入跟踪。当前 diff 未改安装器、私有配置或生产路由代码。同步命令限制在七个项目目标；没有执行 `install-all`、全局安装、提交、归档或远端操作。
- 当前合同与交付流程已写回业务 skill 源及适用工具文档。根规范已有 Node 标准库、正式门和授权边界；本项没有引入需要新增规范的算法或安装器机制。

## 正式验证

复用实施代理在最终 T02 文件版本上运行的正式门，未与正式门并行运行竞争性测试或构建，也未重复完整门。审查重新计算 `validated-state.json` 中 6 个文件的哈希，全部匹配。审查还验证聚焦测试、完整门、docs 和同步日志的收据哈希；`.log.gz` 的 `raw_log_sha256` 对应解压后的原始内容。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| 聚焦安装器测试 | PASS，14/14 | `checks.json`、`focused-tests.log` |
| 前端 lint / typecheck | PASS | `just-ci.log` 中 `eslint src`、`tsc --noEmit`，完整命令退出 0 |
| Rust fmt / clippy | PASS | `just-ci.log`，clippy 保持 `-D warnings` |
| `just ci` | PASS | Node 182；Vitest 73 文件、300 测试；Rust 单元 542、集成 3；6 ignored；doc tests 0 |
| `just docs-build` | PASS | `checks.json`、`docs-build.log` |
| 实际副本 `--check` | PASS，退出 0 | `delivery-commands.json`、`after-check.log` |
| 二次默认安装 | PASS，写入 0 | `delivery-commands.json`、`second-install.log` |
| 独立磁盘与收据哈希核对 | PASS | `review-snapshot.json`；21 个业务文件、21 个新备份、30 个旧额外文件 |
| `git diff --check` | PASS | 实施收据及审查再次执行均退出 0 |

修复前 `--check` 退出 1 的 21 项差异保留在 `before-check.json` 和 `before-check.log`。T02 正式门未复现此前 T03 的 `OperationInterrupted`；T03 原始失败证据仍由 T03 保留。

## 未验证边界

五客户端的 fresh-session、真实 skill 调用、子代理行为与 hosted CI 不属于本项通过结论，继续由 T06 记录。6 个 Rust ignored 检查保持未运行。副本文件一致和 fixture 通过均不证明客户端已经发现或执行 skill。本审查没有读取真实私有 TOML、生成 JavaScript 或数据库。

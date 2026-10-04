# T02 实施与验收证据

日期：2026-09-30。基线 HEAD：`d3a25b4164343f5cbeab8a51efe1a1d3974cdf4a`。用户已批准项目内七目标备份同步。T03 授权文本保持完整；本次没有修改安装器、业务生成器或公开路由逻辑。

## 改动

- `tests/install-agent-skills.test.js`：在现有真实 payload 七目标 fixture 中逐份加载已复制生成器，验证 26/13/13、`extra_anyrouter → anyrouter.top`、`extra` 为 unsupported。生成器通过 `buildInputs(repoRoot)` 显式读取当前公开模板。
- `skills/residential-rule-tuning/SKILL.md`：增加源更新后的交付流程和证据边界。
- `docs/agents/residential-rule-tuning.md`：记录差异审查、限定目标、备份核验、目标数量、幂等和三类证据；注明五工具及 `.agents` / `.cursor`。
- 实际同步 `.agents`、`.claude`、`.codex`、`.cursor`、`.omp`、`.grok`、`.kimi-code` 的 `skills/residential-rule-tuning/`，每个目标包含 `SKILL.md`、`reference.md`、`scripts/build-inputs.js`。这些副本和备份仍受 gitignore 管理。

## AC 对照

| AC | 结果 | 证据 |
| --- | --- | --- |
| AC1 | 21 项差异均为已审查旧副本；21 个新备份与覆盖前 SHA256 相同；30 个原有额外文件全部保留 | `before-inventory.json`、`before-diffs.diff`、`after-verification.json` |
| AC2 | 7 个实际目标、21 个文件与单一源逐字节一致；同步后 `--check=0`；二次安装写入 0 | `delivery-commands.json`、`after-check.log`、`second-install.log`、`after-verification.json` |
| AC3 | 临时七目标 fixture 与实际七目标生成器均得到 26 routing / 13 supported / 13 unsupported；两个 extra 断言通过 | `focused-tests.log`、`actual-generators.log`、`after-verification.json` |
| AC4 | 文档区分临时 fixture、实际副本一致、客户端发现与执行；明确零目标成功退出不证明交付 | 上述两个说明文件 |

现存额外文件 30 个均为历史备份；临时 fixture 另验证 `user-notes.md` 的保留。没有打开或写入私有 TOML/JS、真实数据库或凭据。同步命令限定七个既存目标，未使用 `--create` 或 `install-all`。

## 检查

| 命令 | 退出码 / 结果 |
| --- | --- |
| 同步前 `node scripts/install-agent-skills.js --check` | 1，保留原始失败收据 |
| `node --test tests/install-agent-skills.test.js` | 0；14 通过、0 失败 |
| 限定七目标的 `--force` / `--check` / 二次默认安装 | 各 0；写入 21 / 检查一致 / 写入 0 |
| `just ci` | 0；Node 182；Vitest 73 文件、300 测试；Rust 542 单元和 3 个 kill_gate 集成测试通过、6 ignored、doc tests 0 |
| `just docs-build` | 0；使用已有 docs 依赖 |
| `git diff --check` | 0 |

`checks.json` 记录正式门的时刻和退出码。`validated-state.json` 记录门后文件哈希；所有 T02 产品编辑均在聚焦测试之前完成，之后未再修改产品文件。`delivery-source.json` 与门后源文件哈希一致。正式门未复现 T03 曾记录的 `SQLITE_INTERRUPT`；该历史失败记录不受本次结果影响。

`just-ci.log.gz`、`docs-build.log.gz` 和 `before-diffs.diff.gz` 保留原始内容。可读文件只去掉行尾空白；哈希分别记录在 `checks.json` 和 `evidence-normalization.json`。

## 剩余边界

本任务证明本机七目录投递和显式根路径的生成器结果。五客户端发现、fresh-session 加载与实际调用继续由 T06 负责；本次没有运行客户端推理会话。Node 26 的本机结果不能替代 hosted Node 18/20/22 矩阵。六项 Rust ignored、原生安装态和性能容量门未在 T02 验证。未提交、归档或执行远端操作。

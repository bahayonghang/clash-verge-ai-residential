# Implementation

1. 批准后重读七目录差异，只有已审查旧内容时同步；额外用户修改保留并报告。
2. 扩展现有tests，验证真实payload/新目标/幂等，不复制生产业务映射自证。
3. 更新安装说明和适用工具。
4. `node --test tests/install-agent-skills.test.js`、`npm run ci`、docs build。
5. 限定本仓库运行 `node scripts/install-agent-skills.js --force --platforms .agents,.claude,.codex,.cursor,.omp,.grok,.kimi-code`，保留.bak。
6. `node scripts/install-agent-skills.js --check` exit0，第二次正常安装written0；确认全局目录与私有配置未变。
7. 强模型复核统计语义及安装证据，向C3和父任务回写五工具适用结论。

低价模型可执行fixture/文案/确定性同步，强模型负责统计与覆盖边界。

## C4 实施记录（2026-09-07）

- 测试调用已发布安装器，在临时仓库用真实三文件 payload 覆盖七个平台根；二次安装 `written=0`；保留冲突拒绝与 `--force` 备份。
- 生成器合同仍走源 `build-inputs.js`：24/12/12、三 core、`grok_web_assets` → `GROK_STRICT_EXACT_DOMAINS`。
- 本机 `node scripts/install-agent-skills.js --force --platforms .agents,.claude,.codex,.cursor,.omp,.grok,.kimi-code` 后 `--check` 退出 0。ignored 副本不提交。
- `node --test tests/install-agent-skills.test.js` 两次 14/14；`npm run ci` 退出 0。
- `docs/en/agents/residential-rule-tuning.md` 不在本任务文件清单，未纳入提交。

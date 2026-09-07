# Implementation

1. 批准后读取 C1 workflow、真实 justfile 与三个 manifests。
2. 修改 AGENTS/CLAUDE/README/index 与 quality spec，保留 managed block 和产品边界。
3. `python .trellis/scripts/get_context.py --mode packages`，逐一读索引与链接目标。
4. `npm run check:secrets`、`npm --prefix docs run build`、`git diff --check`；人工核对新链接。
5. 强模型核对五工具 loader 证据，保证单向引用、路径范围和审批语义正确。
6. 回写项目合同并注明五工具适用，C3 再检查客户端加载。

分工：Claude Code/Codex 强模型决定合同归属；低价模型按明确内容整理文案、替换路径。不得扩展为 runtime 或包体系改造。

## C2 实施记录（2026-09-07）

- `AGENTS.md` 在 Trellis managed block 外写入自包含合同；不再 `@CLAUDE.md`。
- `CLAUDE.md` 单向 `@AGENTS.md`，无循环。
- `README.md` 仅校正本地验证段。`.trellis/spec/residential-monitor/index.md` 新建并指向 backend/frontend/storage。
- `quality-guidelines.md` 只更新 Validation Gate；Main Branch Protection 未改。
- 验证：`get_context.py --mode packages` 退出 0；`npm run check:secrets` 退出 0；`npm --prefix docs run build` 退出 0；`git diff --check` 退出 0。五工具动态加载仍 UNVERIFIED，归 C3。

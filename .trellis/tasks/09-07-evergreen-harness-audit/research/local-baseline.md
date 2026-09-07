# 本地审查基线

日期：2026-09-07。工作目录：仓库根。基线：dev / `5578576787dba73ba1b96985a2fc408fc246bfed`；初始工作树干净。

环境：Windows / PowerShell；Node v26.7.0、npm 12.0.2、rustc 1.98.0、cargo 1.98.0、just 1.58.0。沿用现有依赖安装；本轮没有执行 npm ci、更新依赖或锁文件。命令通过 RTK 执行，下面写实际底层命令。

## 现有检查结果

| 检查 | 结果 | 解释 |
|---|---|---|
| `npm run ci` | PASS，exit 0；125/125 tests | 包含语法检查与模板安全扫描；不是整个仓库的 `just ci` |
| `npm --prefix residential-monitor run check` | PASS，exit 0；69 文件，284/284 tests | 图标、tsc、ESLint、Vitest、Vite build 均通过 |
| `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace` | PASS，exit 0；435 passed / 1 ignored | RTK 汇总 6 suites；忽略的是 credential_windows_generic_crud |
| `cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check` | PASS，exit 0 | 首次与其他检查顺序执行；为排除末命令掩盖状态，单独取退出码再次确认 |
| `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings` | PASS，exit 0 | No issues found |
| `node scripts/sync-monitor-version.js --check` | PASS，exit 0 | 单独运行确认版本已对齐 0.3.0；不写文件 |
| `npm --prefix docs run build` | PASS，exit 0 | VitePress 2.0.0-alpha.19 / Vite 8.2.2 |
| `node scripts/install-agent-skills.js --check` | FAIL，exit 1 | 七个平台各 3 文件与源不同，共 21 差异 |
| `cargo run --quiet --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- c5-fault` | PASS，exit 0，7 行 passed=true | fixture / temp-sqlite / 未接通知 sink；不是现场故障证明 |
| `cargo run --quiet --manifest-path residential-monitor/src-tauri/Cargo.toml --bin monitor-bench -- c5-supply` | PASS，exit 0 | cargoPackages=472，npmPackages=511，secretHits=[]，signed=false，installerSha256=null；只是 lockfile 清单，不是漏洞审计或签名验证 |

`just ci` 未作为整体重复执行：它还含 `npm --prefix residential-monitor ci`；本轮分别完成现有安装环境中的各检查，不声称完成锁文件重装或 Node 18/20/22 的本地矩阵。`monitor-c5-auto` 的 Rust c5 单元测试包含在全 workspace 中，两个 bench 子命令另外通过，未重复运行其 wrapper。

## 复现：skill 源与安装副本漂移

只读运行两个模块的 `buildInputs(process.cwd())`，不写 JSON、不读私有配置。结果：

| 值 | 源 skills/residential-rule-tuning | 安装副本 .agents/skills/residential-rule-tuning |
|---|---|---|
| routing 总数 | 24 | 24 |
| supported / unsupported | 12 / 12 | 9 / 15 |
| 三个新 core 开关 | supported | 全落入 unsupported |
| grok_web_assets | grok.com, cli-chat-proxy.grok.com, code.grok.com | auth.x.ai |

七个平台为 `.agents`、`.claude`、`.codex`、`.cursor`、`.omp`、`.grok`、`.kimi-code`。各目录的 `SKILL.md`、`reference.md`、`scripts/build-inputs.js` 均有字节差异；对 `.agents` 的换行归一化比较仍不同，排除 CRLF 假阳性。源文件已经包含正确映射，根测试只覆盖源和临时安装 fixture；`package.json` / `justfile` 的 CI 没有核对本地副本。

已安装 SKILL 实际要求从仓库根运行源脚本，因此不能宣称所有正常调用均产生错误数字。已验证的是副本说明过时，以及显式加载副本模块时的错误映射；源脚本路径下输出正确。

## 规则导航

`get_context.py --mode packages` 输出 single-repo、layers=frontend,residential-monitor。实际 `.trellis/spec/residential-monitor/index.md` 不存在，规范位于它的 backend/frontend/storage 三个子目录。启动指引按 `<layer>/index.md` 读取会遇到缺失路径；应加最小导航索引，避免为此改造包体系或 Trellis 引擎。

## 未验证

- Credential Manager 的真实写入测试被明确 ignore（credential.rs:315）；本轮不越过这个边界。
- 真实 Clash/Mihomo、NSIS、WebView、Windows 自动启动、30 天库与 24 小时 soak。
- 五个客户端实际冷启动、子代理读到的规则和权限；静态文件或当前 Codex 会话不能替代其它客户端证据。
- 当前 SHA 的 hosted CI 由独立 CI 审查记录；不引用此前归档的成功记录替代本轮执行。

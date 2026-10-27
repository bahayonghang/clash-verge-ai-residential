# 验证记录

## 修复前复现

实施代理先写回归测试，再在未修改产品代码时运行四个开关组合：4 tests，2 pass / 2 fail。两个 core=true 组合均缺少 DOMAIN,auth.grok.com,AI-家宽，复现成立。修复后四组合全部通过。

## 实施与独立复核

- npm run ci：264/264 通过，语法/代理合同/安全扫描通过。日志：C:/Users/lyh/AppData/Local/Temp/grok-signin-implement-root-ci.log。
- npm --prefix docs run build：通过。日志：C:/Users/lyh/AppData/Local/Temp/grok-signin-implement-docs-build.log。
- 独立 trellis-check：完整 diff 范围复核未发现遗留问题，无追加修改；独立 npm run check 与 git diff --check 通过。
- 新测试通过 main 处理开启输出再重建四种开关组合，覆盖旧规则/DNS 清理、输入不变、自定义规则保留、完整输出重复执行幂等，以及关闭网页资源时排除 assets、connectors 与嵌套 auth 主机。
- 默认规则数量 60；历史 fixture 未改；下游 build-inputs 归属 core。

## TLS 辅助检查

通过 node:tls，host/servername=auth.grok.com，port=443，rejectUnauthorized=true，10 秒超时。结果 authorized=true，protocol=TLSv1.3。

视角是系统解析器及当前网络路径，未强制住宅 DoH，不能作为住宅 DNS 视角、住宅代理出口或完整登录流程可用性的证明。业务准入来源仍是官方最小登录主机名单，证书通配范围不决定路由范围。

## 完整检查环境

上一轮审查的 just ci 被包装器及 cargo 符号链接启动异常阻塞。本轮只读定位已安装的真实默认工具链：rustup show active-toolchain = 1.99.0-x86_64-pc-windows-msvc (default)。该工具链 bin/cargo.exe 与 rustc.exe 均可直接执行。仅在检查进程的 PATH 前置该目录，并使用已安装 just 1.59.0 的实际 exe 路径运行原始 ci recipe；不修改工具、系统环境、检查命令或检查门。

第一次重试通过 monitor 前端 300 项测试与构建及 Rust fmt，clippy 受原有 residential-monitor/src-tauri/target 符号链接的 Access is denied 阻断。保留该链接，第二次重试仅将 CARGO_TARGET_DIR 指向系统临时目录 grok-signin-fix-rust-target，继续运行同一个 canonical just ci。日志：C:/Users/lyh/AppData/Local/Temp/grok-signin-fix-just-ci-temp-target.log。

最终 canonical just ci 退出码 0：版本对齐、前端安装、typecheck、lint、73 个前端测试文件/300 项测试、前端构建、Rust fmt、Clippy -D warnings、Rust lib 550 passed/6 ignored、集成测试 3 passed、根目录 264 项测试、语法/代理合同/安全扫描全部通过。ignored 为原有测试，未更改忽略设置。文档构建作为独立检查另行通过。

## 交付状态

两项修复与全部约定检查已完成；真实住宅 DoH 和完整登录流程未实测。用户后续已明确授权提交改动并归档任务，当前按修复提交、任务归档、会话记录顺序收尾。不推送，不改变本地配置或运行中代理。

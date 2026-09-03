# Clash Verge AI 家宽路由

Clash Verge Rev 全局扩展脚本：只把 Claude、ChatGPT、Gemini、Antigravity、Cursor、Grok Build 的核心 AI 流量送进住宅 SOCKS5 链路。插件市场、下载、YouTube 和其他非 AI 流量仍走原 Profile。

```text
本机 -> 当前 Profile 的机场代理组/节点 -> 家宽 SOCKS5 -> AI 服务
```

完整边界见 [路由范围](routing-scope.md)。不要在本页查找开关表或域名清单。

## 本地打开文档站

文档站只在本机预览，不发布 GitHub Pages。需要 Node.js 22+。在仓库根目录：

```bash
npm --prefix docs install
just docs-dev
```

没有 `just` 时用 `npm run docs:dev`。构建用 `just docs-build`。扩展脚本的 `just ci` 仍是 Node 18+，不安装文档依赖。

英文版在 [English](/en/)，源文件在仓库 `docs/en/`。

## 使用与配置

- [本地配置](local-configuration.md)：本地 TOML、`just render-local`、凭据保护和完整开关表
- [配置开关](configuration.md)：两种使用模式、上游候选、Clash Verge 设置
- [路由范围](routing-scope.md)：纳入、排除、未验证项、新域名准入
- [多 Profile](multi-profile.md)：`dialer-proxy` 解析顺序与递归保护
- [DNS 与泄漏模型](dns-and-leak-model.md)：DNS 路径和脚本单独保证不了的泄漏面
- [故障排查](troubleshooting.md)：常见失败与残留规则

根 [README.md](https://github.com/bahayonghang/clash-verge-ai-residential/blob/dev/README.md) 仍是 GitHub 上的总览。GitHub 可以直接打开上述中文 Markdown。

## Agent

给仓库里的 Agent 技能用，不是终端用户手册。

- [Domain 文档](agents/domain.md)
- [Issue tracker](agents/issue-tracker.md)
- [分诊标签](agents/triage-labels.md)
- [家宽规则优化](agents/residential-rule-tuning.md)

已记录的决策在仓库文件 `docs/adr/`，不进入本站点。

# Implementation Plan: Cursor 仓库上传家宽排除优化

## Preconditions

- 用户已批准 `routing.cursor_repository_indexing` 默认 `false`。
- 实施前运行 `task.py start`，并读取 `prd.md`、`design.md`、本任务 research 与前端 spec。
- 保留未跟踪的 `.trellis/tasks/08-18-residential-monitor-mvp/`，不得改写或纳入本任务提交。
- 不编辑用户 `.local.toml`、生成 `.local.js`、Clash Verge 运行时 Profile 或未脱敏日志。

## Ordered Checklist

- [x] 1. 在根脚本新增 `ROUTE_CURSOR_REPOSITORY_INDEXING=false`，将 repo 正则从 Cursor 核心正则
  拆为独立目录；更新活动规则、全部可能规则和测试导出，确保托管旧 repo 规则仍可清理。
- [x] 2. 在 `scripts/sync-local-config.js` 注册 `routing.cursor_repository_indexing`，在公开 TOML 示例中
  添加 `false` 默认值；沿用现有补全、布尔校验和原子写入机制，不增加解析分支。
- [x] 3. 更新 `tests/regression.test.js`：覆盖四种开关组合、`repo42/repo99`、Cursor 其他核心主机、
  Marketplace/CDN/下载负向范围、规则顺序、关闭后的旧规则清理、未知/退役规则保留和二次运行幂等。
- [x] 4. 更新 `tests/sync-local-config.test.js`：覆盖缺字段补全为 `false`、显式 `true/false` 生成行为、
  新常量唯一锚点、非法类型/重复字段失败、公开模板不变和失败时无半成品。
- [x] 5. 同步 README、`docs/local-configuration.md`、`docs/configuration.md`、
  `docs/routing-scope.md`、`docs/troubleshooting.md` 与英文 CHANGELOG。明确默认回落机场、显式开启回滚、
  `repo42` 证据、数字正则的项目策略、Privacy Mode 与 `api2` HTTP/1.1 回退限制。
- [x] 6. 按仓库版本约定同步根脚本版本、`package.json`、README 当前版本和 CHANGELOG；不要发布 tag、
  GitHub Release 或执行任何远程写入。
- [x] 7. 检查 diff，确认没有凭据、完整本机路径、codebase ID、生成配置或现有监控任务内容进入产品改动。

## Validation Commands

按失败面从小到大执行：

```powershell
node --test tests/regression.test.js
node --test tests/sync-local-config.test.js
npm test
just ci
python -X utf8 .\.trellis\scripts\task.py validate 08-18-cursor-repository-upload-routing
```

补充静态审计：

```powershell
rg -n "ROUTE_CURSOR_REPOSITORY_INDEXING|cursor_repository_indexing|CURSOR_.*DOMAIN_REGEXES" .
rg -n "Cursor.*索引|repository indexing|repo42|repo\[0-9\]" README.md docs CHANGELOG.md
git diff --check
git status --short
```

真实 Clash/Mihomo 验证不是 Node 测试的一部分。若实施会话未在脱敏 Profile 上观察 Connections，报告
`repo42` 实际回落和其他 Cursor 功能保持家宽为 `UNVERIFIED`，不得用生成规则测试冒充运行时证据。

## Review Gates And Rollback Points

- 完成步骤 1 后先运行路由回归；若未知规则被误删或 repo 规则无法在关闭后清理，停在根脚本层修正。
- 完成步骤 2-4 后运行渲染测试；若补全改写已有 TOML 格式或失败路径产生输出，回滚新字段接线并复查
  现有通用机制，不能为新字段建立第二套解析器。
- 文档必须与最终常量默认值和实际测试一致；`repo[0-9]+` 不得标为 Cursor 官方通配合同。
- `api2.cursor.sh` 必须继续受 `cursor_core` 控制；任何试图用共享域名弥补 HTTP/1.1 回退的改动均越界。
- 全量 `just ci` 通过且 diff 无关项为空后，才进入 Trellis 检查、spec 判断和提交审批。

## Deferred Verification

- Cursor 后续版本是否改变 `repoBackendUrl`、RepositoryService schema 或对象存储传输：`UNVERIFIED`。
- 服务端是否对当前账号强制 HTTP/1.1 回退：现有 2026-08-17 日志显示 HTTP/2，未来状态 `UNVERIFIED`。
- 规则部署到 Clash Verge Rev 后的实际连接字节变化：需用户批准实施并更新运行时脚本后另行验证。

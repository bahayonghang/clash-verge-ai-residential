# 设计：可达上游 empty-fallback 防护

## 最小改动

在 `hardenReachableUpstreamGraph` 的 visit 中，完成组入栈后、继续改写该组前，检查 `group["empty-fallback"]` 是否为 `injectedNames()` 中的保留名称。命中则调用现有 `fail`，说明组名、empty-fallback 和当前 stack 路径。

两个保留名称均禁止。真实有效的循环示例是 fallback 指向 SOCKS5 节点 `家宽-SOCKS5`；官方文档不允许 empty-fallback 指向其他代理组，因此不依赖 `AI-家宽` 组来证明运行时行为。

不新建通用图框架，不遍历所有代理节点 dialer，不更改普通字段。沿用当前显式 proxies 的 DFS；对顶层和嵌套可达组使用同一检查。不可达组不进入新检查。

## 错误与兼容

选择报错，避免删除用户字段后落入内核缺省 COMPATIBLE，或替用户选择新的传输路径。输出错误不包含 endpoint、凭据或完整配置。

普通节点回退保持，不存在该字段时不补值。原来的 include-all 排除、显式住宅引用清理、UDP 告警及循环检查不改语义。

`cloneConfigForEdit` 已克隆组对象。新检查抛错时原始对象保持不变。文档提醒先修正配置并确认加载成功：Verge 可能放弃失败脚本的输出而回到原 Profile，不能宣称抛错完成网络阻断。

## 文件范围

- `clash-verge-ai-residential.js`
- `tests/regression.test.js`
- `docs/configuration.md`
- `docs/troubleshooting.md`
- `.trellis/spec/frontend/state-management.md`
- `CHANGELOG.md` 的 Unreleased
- 本任务规划与验证记录

不修改旧固定投影 fixture、域名列表、DNS、渲染器、版本号或依赖。

## 回滚

可独立回滚新检查、对应测试与文档，不更改用户 Profile。回滚恢复旧校验覆盖，不代表不安全 fallback 已变得有效；实际用户配置应由用户修正。

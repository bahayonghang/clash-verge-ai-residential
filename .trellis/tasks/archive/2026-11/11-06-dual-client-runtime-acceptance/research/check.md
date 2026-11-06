# 运行验收文档独立审查

日期：2026-11-06。角色：已派发的 trellis-check；未再派发代理。范围限双语运行验收页、DNS 模型、排错与多 Profile 的运行说明及导航。只读核对任务 PRD/design/implement、四份平台报告、宿主合同/验证记录与 hook-guidelines 新合同；未操作真实客户端或网络。

## Findings (fixed)

- 文件：`docs/client-validation.md`、`docs/en/client-validation.md`、`docs/dns-and-leak-model.md`、`docs/en/dns-and-leak-model.md`。
- 问题：AI DNS 路径的概括遗漏额外站点的 `residentialDns: false` 例外，可能把有意只配置家宽业务路由的站点误判成 DNS 故障。
- 修复：四页同步说明 AnyRouter 等额外站点的业务/DNS 分离；应依据实际生成的 policy 验收，不为通过检查扩大 DNS 范围。依据根脚本 `EXTRA_SITES`、`residentialDnsSuffixSet` / `residentialDnsExactSet` 与 `buildNameserverPolicy` 的实际调用路径。没有改代码、名单或默认值。

## Findings (not fixed)

- 运行证据仍不足以宣称四组合整体验收通过。报告正确保留 Verge GUI 未运行、FlClash 核心版本/加载身份、真实选路、DNS 五类、TUN/IPv6 及持久化缺口；这些需要主会话后续验收，文档不能补成 PASS。
- 已向主会话提出：四份平台记录补充模板要求但当前未知的桌面会话、安装方式、Profile 来源/绑定及候选身份等元数据，未知项明确 UNVERIFIED。研究报告归主会话所有，本审查不代填未观测事实。

## 审查结论

- 四态定义、中英文含义与缺设备用 UNVERIFIED 一致；未发现用单元测试、配置预览或核心接受冒充运行通过的结论。
- 两语正则 DNS 说明均为现有 inline/classical provider 与 `rule-set:` policy，保留开关、清理、所有权和回退原始 Profile 的边界。
- Verge 行为限定在已核对版本；FlClash 单参数、后置覆写、整表 policy 替换与 macOS DNS 协调器边界同宿主证据一致。FlClashCore 明确禁止套用独立 Mihomo 的 `-v/-h/-t`。
- 私网 system/stub、IPv6 字段、TCP/UDP DNS、ICE 零候选及实际通话分项描述正确；恢复先保留原始输入，未要求关闭现用代理做对照。
- 报告的 HTTP 404/200 只判 TLS/HTTP 传输，未冒充认证、推理、住宅选路或 TUN/DNS 通过。Windows ICE 样本保留 701 与路径未知，不宣称无泄漏。
- `.trellis/spec/frontend/hook-guidelines.md` 新合同只读审查未发现需修复项；宿主测试与原生运行的区别明确。

## Verification

- 格式检查：`git diff --check` PASS。
- 导航 JavaScript 语法：`node --check docs/.vitepress/config.mjs` PASS。
- Lint / TypeCheck / Tests：本轮无产品或测试代码改动，不重复完整 gate；主会话提供的 canonical `just ci` 成功证据见宿主任务 `research/validation.md`，本次未独立重跑。
- 最终 `just docs-build` 与敏感信息扫描：由主会话在集成本审查改动后执行，本文不预判结果。

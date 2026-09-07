# 修复业务 Skill 分发漂移

## Goal
让业务 skill 源库与实际平台副本一致，并给出干净 checkout 的项目级安装路径。

## Evidence
父任务 local-baseline.md：--check exit1，七平台21文件不同；installed supported9/unsupported15，source12/12；installed grok_web_assets 错映射 auth.x.ai。正常 SKILL 文档仍指向根源脚本，不夸大成所有调用失败。

## Requirements
- R1 批准后审查并同步本项目七目录业务 skill，保留 force 备份，保护额外用户内容。
- R2 在临时 fixture 验证所有已声明平台的完整 payload 和幂等，不依赖用户全局安装。
- R3 明确源库、本机 ignored 副本、干净 clone 和 --check 的证据差别。
- R4 保持24/12/12与正确Grok映射，不修改路由算法。

## Acceptance Criteria
- [x] AC1 / R1：七目录三个文件均一致，--check exit0；覆盖前确认仅有已审查旧版本差异。
- [x] AC2 / R1–R2：原幂等/冲突拒绝/备份测试通过，真实三文件payload覆盖全部七目录，二次安装written0。
- [x] AC3 / R2–R4：从根按文档调用源生成器得到24/12/12，三core和Grok回归通过；副本payload完整。
- [x] AC4 / R3：说明 fresh checkout 的显式创建目标命令、--check作用域、批准同步流程；无目录时 --check成功不代表五平台安装。
- [x] AC5 / R1–R4：npm run ci通过，副本同步有证据，产品与私有配置无diff。

## Out of scope
全局skill/CLI安装、install-all/tinstall、真实私库、新平台或新分发框架。

## Dependencies
可独立执行；C3 客户端 smoke 使用本项完成后的业务 skill。

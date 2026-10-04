# 五 harness 继续实施最终本地审查（2026-10-01）

结论：本轮 raw_fold 最小缓存回退、实际合同同步与本地产品/文档检查通过。T03 修订 AC5 已记录本轮本地验收，T04 修订 AC3 保持未勾选。T05 性能/容量与 T06 真实客户端门仍未完成。没有提交、归档、push、全局修改或私有库读取。

## 回退源码与合同

独立逐字比较 HEAD 生产区与当前 raw_fold.rs：唯一差异为 `if value == UNKNOWN_IDENTITY` 增加 `&& kind == "process"`。Dict 恢复四个 HashMap<i64,String>、load_sessions 恢复逐 session 字典/chain派生、identity_from_dict/rule_name/load_dict 恢复原合同实现；没有留下 DictValue/ChainIdentities/lazy缓存。当前测试区与 owner 保存的 retained raw_fold-before.rs 逐字相同，独立SQL oracle、重复chain/规则fallback/空值/悬空ID/负分钟/exit tie及未引用字典值 fixture保留。当前全部文件 hash 为4D6E8FEB88B142E09F8E62D60BAF20BAC970F11CB8D8E9108B08CFFBF717E6AA。

相对HEAD，本次批准的SQL、service及其它四份bench仪表文件没有新增差异，facade仅保留原始writer PRAGMA实际采样与相关fixture的16增/2删。reader投影边界3120、整报告deadline、取消、schema、WAL/FULL与AUTO_DELETE_ENABLED未改变。facade初末库存读取原writer connection，初始库存位于CPU/VFS计量前，最终库存位于ending samples之后；顶层synchronous来自实际最终数值。process采样仍GetProcessTimes当前进程差值，未为历史CPU0异常新增推断。

storage sqlite-contract.md新增七段描述实际身份与chain派生合同，并仅对未来cache提出保持语义条件；没有要求当前实现必须有cache。Network literal __unknown__使用字典等值，特殊缺失分支仅Host/Process；空dictionary存在标志与missing/dangling分离，规则fallback和原始exit key保留。probe必须区分测试执行与production JSON成功，空测试、DeadlineExceeded、已读缓存和null写入不能构成更广验收。

108项source manifest检查前后相同，SHA256=8EE21DEACDFE15166BC97A92D78E24445EF26AC90D4813FB30EB368C180996CA。本审查重新逐项hash当前源码，108/108匹配。对应独立证据为final-checks-20261001/rollback-independent-validation.json。该manifest与原retained860165F3...不同，旧性能PASS不迁移。local debug test exe不构成release性能identity。

## 实际检查证据

owner rollback-cache-20261001五份finished receipt均exit0并绑定上述8EE21DEA...源码manifest；10份gzip原stdout/stderr解压后hash独立核对匹配。原始完整just ci日志逐项记录monitor版本对齐、npm ci、icons/typecheck/lint/前端tests/build、Rust fmt/clippy/workspace tests、安全扫描及根npm run ci。独立核对实际计数：前端300、Rust单元546及集成3、根Node203，0failed；6个Rust ignored未执行。targeted raw为7passed/1ignored，service53passed/3ignored。没有重复运行这些native检查。

本审查新执行just docs-build、node scripts/check-agent-contract.js、node scripts/install-agent-skills.js --check、git diff --check及父+T01–T06共七条task.py validate，11项均exit0。随后仅因T03验收meta与本报告写入，复验T03 task validate/空白检查，均exit0。每条argv、PID、时间、原exit、stdout/stderr gzip及hash保存在final-checks-20261001/；初批summary.json与post-edit receipt分开，不覆盖原收据。此前两项Node可选覆盖fixture为55/55PASS，原receipt/stdout仍保留。

Lint PASS（owner原完整gate的eslint/clippy）；TypeCheck PASS（owner原gate的tsc --noEmit）；Tests PASS（上述实际日志计数及55项定向fixture）；Docs PASS（本次实际build）。根无独立linter，syntax/contract/secret由原根gate与本次合同检查证明。dependency-audit不属于just ci或docs-build；本次未重新运行独立安全审计，也不将本地安装成功当auditPASS。

七task validate均为context路径/格式验证通过，不代表七任务已验收。均保留既有workflow.md为44244B超过32768B注入上限Warning；需要完整pull上下文，不能以截断片段替代，本审查复用了已完整加载上下文。git仅有已有CRLF提示，无空白错误；不在本轮重写workflow或行尾策略。

195份旧matrix、中断primary与完整恢复primary证据重新hash全部保持；见独立validation。旧FAIL/UNVERIFIED与首轮candidate CPU前后9.765625、300样本恒定、delta0的原因未查明继续保留。driver3与实际outer1仍分别保留，不转为执行PASS。

## T03/T04 条款状态

T03 AC5的新可选覆盖合同符合当前checker/env实际路径：必需合同存在；可选覆盖ENOENT跳过；非ENOENT读错失败；已有Codex/Kimi check按定义markers校验；Kimi implement/research仅按定义读取与引用。project.overrides报告缺失/未选用，不增加checker输出门。55项fixture、当前合同检查、完整产品门和docs均通过，因此本次只勾选T03新AC5、更新本轮verification meta；旧review与implementation_status保留历史范围，task仍in_progress。没有修改四个ignored覆盖或Git index，不声明角色已加载。

T04历史bootstrap-result/review-bootstrap-verification中同一0.7.0-beta.3入口真实init=0、四份已选copy覆盖前后相同仍为有效历史证据。当前可选缺失fixture已通过；本轮没有在缺失覆盖的新候选上实际init，因此新AC3仅部分证明，继续未勾选。没有伪填fresh absence init、全局config/trust全盘字节保护或真实客户端PASS。

## 后续边界与停写

capacity为资产前置BLOCKED、0/106，固定旧db/bench exe及A50/A25030d语料缺失、无备份；原因未查明。公开source重建方案仍DRAFT，未构建/生成新asset或降低106/21/10000ms门。A1000、同窗口matrix、全部应用文件写入、安装态/WebView/worker/24h/peak、物理冷页与hosted检查保留各自未完成状态；普通本地CI不能关闭这些门。

T06准备元数据和synthetic审查通过，仅PREPARATION_CHECKS_PASS_RUNTIME_NOT_RUN。新entry版本0.159.2不转移旧V1/V2；model/backend仍UNKNOWN。主会话将冻结当前dirty树并分别给予parent/childGO。完成本报告及验收meta后，审查代理停止全部写入和负载，供T06全保护snapshot冻结。后续仅由主会话按正式边界安排运行。

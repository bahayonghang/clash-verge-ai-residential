# 重构设置与数据管理界面

## Goal

在不改变产品能力与安全边界的前提下，把 `settings-data` 从纵向堆叠的大面板改为参考 SkillPort 的“二级设置导航 + 单一内容面”桌面工作台，让连接、外观、数据、关于与危险操作可快速定位。

## Confirmed Facts

- 当前设置页由 `renderSettings` 一次渲染四个 `.panel`，控制器配置与外观同处首块，数据动作平铺，关于和删除沿长页向下排列。
- 当前已有中文 / 英文、Latte / Frappé / Macchiato / Mocha、TCP secret 回填与显隐、重点目标、日志目录、备份 / 恢复、retention、VACUUM、关于与删除能力。
- 固定顶级 route 为 `overview`、`live`、`reports`、`alerts`、`settings-data`；本任务不增加顶级 route。
- 参考截图中的 SkillPort 品牌、技能来源、Accent Color 和字体管理不是本产品能力。

## Requirements

- 在 `settings-data` 内加入可键盘操作的二级导航：外观与语言、连接与监控、数据与备份、关于、危险区域；默认进入连接与监控。
- 主内容面每次聚焦一个分组，使用清晰标题、简短说明、分组行和就地状态；删除现有占首屏的长编号设置向导。
- 外观与语言把四个既有主题呈现为带色彩识别与选中状态的主题选项，把中 / 英呈现为分段选择；选择后继续即时保存和换肤。
- 连接与监控展示启动自动行为、权威连接状态、控制器地址、secret、重点目标和保存 / 测试 / 重连 / 断开动作；单帧测试不得显示成持续监控。
- 数据与备份按日志、备份恢复、保留 / 汇总和 VACUUM 分组；关于独立展示版本、identifier、签名候选状态和固定 Releases URL。
- 危险区域独立置底或独立导航项，使用明确危险视觉、预览、固定确认短语与分项结果；不得把部分失败显示为全部成功。
- 保留四套现有主题变量、中文产品名与本地隐私文案；不复制 SkillPort 字体、红色品牌或专属功能。
- 默认 1200×800 下二级导航与连接核心设置同屏；窄窗口无水平溢出，二级导航转换为可滚动标签或紧凑选择器。
- 全部控件具备可见 focus、40×40 px 最小桌面命中区、明确 hover / active / disabled / loading / error / success 状态；active press 使用克制的 `scale(0.96)`，reduced motion 下停用非必要运动。
- 不使用 `transition: all`、UI 框架、远程资源或新增依赖；图标沿用同一 `currentColor` 描边体系或纯 CSS 状态标记。
- 设置导航选择与未保存表单草稿只属于当前会话视图状态；任何动态重绘不得意外清空正在编辑的地址、secret 或重点目标。

## Acceptance Criteria

- [ ] 五个设置分组可通过鼠标与键盘切换，当前项有 `aria-current` 或等价语义，顶级五页导航不变。
- [ ] 外观 / 语言、连接、数据、关于与删除的现有功能全部可达，四套主题即时切换且中英文本完整。
- [ ] 1200×800 首屏可见连接状态、地址、secret、重点目标与主要动作；窄窗口没有截断、重叠或水平溢出。
- [ ] 切换设置分组、主题、语言或连接状态重绘时，未提交表单草稿与 secret 显隐状态不会意外丢失或泄漏进 HTML。
- [ ] 危险删除与普通数据维护视觉和结构分离，确认短语、预览与部分失败语义保持不变；Recovery Shell 不新增删除入口。
- [ ] 全交互状态、40 px 命中区、可见 focus、reduced motion、精确 transition 属性、`tabular-nums` 通过代码与实拍检查。
- [ ] TypeScript typecheck、lint、tests、build 通过；1200×800 和窄窗口截图经一次完整界面工艺复核，无未解决 HIGH / MEDIUM 问题。

## Out of Scope

- 重构全局侧栏或其他四个业务页面。
- Accent Color、自定义字体、更多主题、云同步、SkillPort 平台管理 / 集成能力。
- 改变备份、retention、VACUUM、关于、删除或 Recovery 的后端语义。

## Key Decisions

- 默认设置分组为“连接与监控”，因为启动连接是本任务的首要用户结果。
- 未保存地址时显示“未配置”与 `127.0.0.1:9097` 建议，不自动发起连接；已保存地址时展示自动连接 / 采集状态。
- 本次是既有视觉世界内的设置页重构，不重写全局 `DESIGN.md`；若形成持久设置页策略，只更新对应 surface brief。

## Planning Status

- `implemented`；实现、独立检查、前端自动门和 Impeccable detector 已通过；真实 Tauri WebView 截图与键盘 / 四主题实拍仍为 `UNVERIFIED`。

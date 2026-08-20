# 设计：筛选工作区

## 状态

- `appliedQuery`：提交给 `query_live_connections` 的权威筛选。
- `filterDraft`：表单原值，不触发查询；字段、模式、单位变化只改 draft。
- `requestSeq`：提交时递增，响应带 seq；旧响应丢弃。

## 交互

快速开关和结果状态位于工具栏首层；已应用条件以 chip/紧凑行显示；编辑器可内联展开。应用时调用既有 `toQueryClause`，重置 cursor；清空全部回到无 clauses。列管理保持单独的 popover。

## 兼容

不扩展 Rust filter contract，不将 draft 写入设置或控制器 JSON。动态 paint 时保留 applied/draft 与焦点 id；条件值统一 HTML escape。实现后由父任务接入热点摘要消费同一 applied query。

# 实施：固定列宽交互

1. [ ] 先补布局/resize 状态纯函数测试，冻结 clamp、最后一列可见和 table width 规则。
2. [ ] 改造 handle markup/CSS 与 pointer capture 状态机，统一取消/结束和单次持久化。
3. [ ] 确认 wrapper/table/colgroup CSS 在实时 paint、主题/语言、隐藏列时保持像素宽度和横向滚动。
4. [ ] 补键盘/焦点/保存失败手动检查；运行 `npm run typecheck && npm run lint && npm test`。

回滚点：`live-table-layout.ts` 的布局模型保持兼容；只回滚 resize 事件和 CSS 即可，不触及筛选/摘要契约。

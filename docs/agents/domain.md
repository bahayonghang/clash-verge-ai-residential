# Domain 文档

工程技能在探索代码库时，应按本页消费本仓库的领域文档。

## 探索前先读这些

- 仓库根的 **`CONTEXT.md`**，或
- 仓库根的 **`CONTEXT-MAP.md`**（若存在）：它指向每个上下文一份 `CONTEXT.md`。只读与当前主题相关的那些。
- 仓库目录 **`docs/adr/`**：读与即将动手的区域有关的 ADR。多上下文仓库还要看 `src/<context>/docs/adr/`。这些是 Git 里的决策原文，不是本站点的页面。不要链到 `/adr/` 或 `/en/adr/`。

上述文件若缺失，**静默继续**。不要把缺失当问题提出，也不要建议先去创建它们。`/domain-modeling` 技能（经 `/grill-with-docs` 和 `/improve-codebase-architecture` 进入）只在术语或决策真正落地时才惰性创建。

## 文件结构

单上下文仓库（本仓库属于这一类）：

```
/
├── CONTEXT.md
├── docs/adr/
│   ├── 0001-process-lookup-vs-process-routing.md
│   ├── 0002-unknown-process-drilldown.md
│   └── 0003-controller-only-process-identity.md
└── src/
```

多上下文仓库（根目录存在 `CONTEXT-MAP.md`）：

```
/
├── CONTEXT-MAP.md
├── docs/adr/                          ← 系统级决策
└── src/
    ├── ordering/
    │   ├── CONTEXT.md
    │   └── docs/adr/                  ← 上下文决策
    └── billing/
        ├── CONTEXT.md
        └── docs/adr/
```

## 使用词汇表里的词

输出里一旦命名领域概念（issue 标题、重构提案、假设、测试名），必须用 `CONTEXT.md` 定义的词。不要滑到词汇表明确避免的同义词。

若需要的概念还不在词汇表里，这是信号：要么在发明项目不用的语言（重新考虑），要么真有缺口（记下来给 `/domain-modeling`）。

## 标出与 ADR 的冲突

若输出与已有 ADR 矛盾，明确写出，而不是默默覆盖：

> _与仓库文件 `docs/adr/0001-process-lookup-vs-process-routing.md` 矛盾，但值得重开，因为…_

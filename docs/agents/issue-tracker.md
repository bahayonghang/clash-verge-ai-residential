# Issue tracker：GitHub

本仓库的 issue 与规格以 GitHub Issues 为准。所有操作使用 `gh` CLI。

## 约定

- **创建 issue**：`gh issue create --title "..." --body "..."`。多行正文用 heredoc。
- **读 issue**：`gh issue view <number> --comments`，用 `jq` 过滤评论并同时取标签。
- **列 issue**：`gh issue list --state open --json number,title,body,labels,comments --jq '[.[] | {number, title, body, labels: [.labels[].name], comments: [.comments[].body]}]'`，加上合适的 `--label` 与 `--state`。
- **评论**：`gh issue comment <number> --body "..."`
- **加/删标签**：`gh issue edit <number> --add-label "..."` / `--remove-label "..."`
- **关闭**：`gh issue close <number> --comment "..."`

仓库从 `git remote -v` 推断；在 clone 内运行时 `gh` 会自动处理。

## 是否把 PR 当分流入口

**把 PR 当请求入口：否。**（若本仓库把外部 PR 当功能请求，把这里改成 `yes`；`/triage` 读这个开关。）

设为 `yes` 时，PR 走与 issue 相同的标签和状态，使用对应的 `gh pr` 命令：

- **读 PR**：`gh pr view <number> --comments`，diff 用 `gh pr diff <number>`。
- **列出待分流的外部 PR**：`gh pr list --state open --json number,title,body,labels,author,authorAssociation,comments`，只保留 `authorAssociation` 为 `CONTRIBUTOR`、`FIRST_TIME_CONTRIBUTOR` 或 `NONE`（丢掉 `OWNER` / `MEMBER` / `COLLABORATOR`）。
- **评论 / 标签 / 关闭**：`gh pr comment`、`gh pr edit --add-label` / `--remove-label`、`gh pr close`。

GitHub 的 issue 与 PR 共用一套数字，光看 `#42` 可能是其中任一种：先 `gh pr view 42`，失败再 `gh issue view 42`。

## 技能说「发布到 issue tracker」时

创建一个 GitHub issue。

## 技能说「拉取相关 ticket」时

运行 `gh issue view <number> --comments`。

## 寻路操作

供 `/wayfinder` 使用。**map** 是单个 issue，**child** issue 是 ticket。

- **Map**：带 `wayfinder:map` 标签的单个 issue，正文含 Notes / Decisions-so-far / Fog。`gh issue create --label wayfinder:map`。
- **Child ticket**：用 GitHub sub-issue（`gh api` 调 sub-issues 端点）链到 map。若未启用 sub-issue，把 child 加进 map 正文的任务列表，并在 child 正文顶部写 `Part of #<map>`。标签：`wayfinder:<type>`（`research` / `prototype` / `grilling` / `task`）。认领后把 ticket 指派给驱动开发的人。
- **Blocking**：GitHub **原生 issue 依赖**，这是 UI 可见的权威表示。用 `gh api --method POST repos/<owner>/<repo>/issues/<child>/dependencies/blocked_by -F issue_id=<blocker-db-id>` 加边，其中 `<blocker-db-id>` 是 blocker 的数字 **database id**（`gh api repos/<owner>/<repo>/issues/<n> --jq .id`，不是 `#number` 也不是 `node_id`）。GitHub 报告 `issue_dependencies_summary.blocked_by`（仅未关闭的 blocker，即现场闸门）。依赖不可用时，退回到 child 正文顶部的 `Blocked by: #<n>, #<n>`。每个 blocker 都关闭后，ticket 才算解除阻塞。
- **Frontier 查询**：列出 map 的未关闭 child（`gh issue list --state open`，范围限定为 map 的 sub-issue / 任务列表），丢掉仍有未关闭 blocker（`issue_dependencies_summary.blocked_by > 0`，或 `Blocked by` 行里的未关闭 issue）或已有 assignee 的项；map 顺序里第一个胜出。
- **认领**：`gh issue edit <n> --add-assignee @me`，这是该会话的第一次写入。
- **结束**：`gh issue comment <n> --body "<answer>"`，然后 `gh issue close <n>`，再把上下文指针（gist + 链接）追加到 map 的 Decisions-so-far。

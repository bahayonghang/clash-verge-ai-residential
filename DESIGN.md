---
name: 家宽流量监控
description: 深色侧栏工作台，浅灰主区读数
colors:
  sidebar: "#222733"
  sidebar-mid: "#2c3545"
  sidebar-text: "#e8eef6"
  sidebar-muted: "#9aa6b6"
  accent: "#3b82f6"
  accent-pressed: "#2563eb"
  main: "#c8c9d1"
  card: "#eef0f5"
  ink: "#1a1f28"
  muted: "#4b5565"
  table-head: "#2a3344"
  table-row: "#343e4f"
  table-text: "#e8eef6"
  danger: "#b42318"
  ok: "#1f7a4d"
  focus: "#93c5fd"
typography:
  title:
    fontFamily: "Segoe UI, Microsoft YaHei, sans-serif"
    fontSize: "1.05rem"
    fontWeight: 650
    letterSpacing: "-0.02em"
  body:
    fontFamily: "Segoe UI, Microsoft YaHei, sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.55
  label:
    fontFamily: "Segoe UI, Microsoft YaHei, sans-serif"
    fontSize: "0.82rem"
    fontWeight: 400
  mono:
    fontFamily: "Cascadia Mono, Sarasa Mono SC, ui-monospace, monospace"
    fontSize: "1.2rem"
    fontWeight: 650
rounded:
  sm: "8px"
  md: "12px"
  mark: "0.85rem"
spacing:
  sm: "0.55rem"
  md: "0.85rem"
  lg: "1.25rem"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "#ffffff"
    rounded: "{rounded.sm}"
    padding: "0.4rem 0.8rem"
  button-primary-hover:
    backgroundColor: "{colors.accent-pressed}"
    textColor: "#ffffff"
  nav-item:
    backgroundColor: "transparent"
    textColor: "{colors.sidebar-text}"
    rounded: "{rounded.sm}"
    padding: "0.48rem 0.55rem"
  nav-item-current:
    backgroundColor: "{colors.accent}"
    textColor: "#ffffff"
  card:
    backgroundColor: "{colors.card}"
    textColor: "{colors.ink}"
    rounded: "{rounded.md}"
    padding: "0.95rem 1.1rem"
---

# Design System: 家宽流量监控

## Overview

**Creative North Star: "Clash Verge 隔壁的观测台"**

桌面工作台。左侧深海军认页，右侧浅灰读数。蓝只出现在当前栏和主按钮。工艺对齐 Clash Verge Rev：图标加文字的侧栏、圆角选中条、扁平面板。

密度是专家工具档。表格保持表格。缺口写「未知」，不写零，不写账单。

**Key Characteristics:**

- 双色：深侧栏 / 浅主区
- 蓝只打选中项和主操作
- 本地生成图标，不走 CDN
- 系统无衬线 + 等宽数字

## Colors

主色是侧栏海军和选中蓝。主区浅灰让数字从卡片和深色表里跳出来。

### Primary
- **选中蓝** (`#3b82f6`)：当前导航和主按钮。

### Neutral
- **侧栏海军** (`#222733`)：应用壳左栏。
- **主区灰** (`#c8c9d1`)：工作面。
- **卡片浅** (`#eef0f5`)：指标和表单面板。
- **表头海军** (`#2a3344`)：数据表。
- **正文墨** (`#1a1f28`)：浅底上的字。

### Named Rules
**The One Blue Rule.** 蓝只用于当前页和主操作。状态靠文案和圆点，不靠第二套彩色徽章。

## Typography

**Display Font:** Segoe UI / 微软雅黑
**Body Font:** Segoe UI / 微软雅黑
**Label/Mono Font:** Cascadia Mono / 更纱黑体等宽

Operate 表面用系统栈。数字用等宽和 `tabular-nums`。

### Hierarchy
- **Title** (650, 1.05rem)：侧栏产品名、区块标题。
- **Body** (400, 1rem, 1.55)：说明和表单。
- **Label** (400, 0.82rem)：指标标签、口号。
- **Mono** (650, 1.2rem)：指标值。

## Layout

固定 13.75rem 左侧栏。主区独立滚动。指标网格 `auto-fit`，最小 11.5rem。打印隐藏侧栏和按钮。

## Elevation & Depth

没有投影。层次靠色块：侧栏、浅卡、深表。

## Shapes

控件 8px，面板和指标卡 12px，产品标记约 14px。选中导航是圆角条，不是下划线。

## Components

### Buttons
- **Shape:** 8px
- **Primary:** `#3b82f6`，白字
- **Hover:** `#2563eb`
- **Focus:** 3px `#93c5fd` 外圈

### Cards / Containers
- 浅灰面板，12px 角，约 1rem 内边距
- 无描边，无阴影

### Inputs / Fields
- 白底，1px `#b7bec9` 边
- 标签在字段上方

### Navigation
- 左栏图标 22px + `titleZh`
- 当前项整行蓝底
- Recovery-only 不渲染五页按钮

### Data table
- 深海军表头和行，浅字
- 等宽数字

## Do's and Don'ts

### Do:
- **Do** 把产品名和页面标题放在左侧栏。
- **Do** 缺口、未知显示「未知」。
- **Do** 用本地图标，侧栏按钮同时有图和字。

### Don't:
- **Don't** 恢复顶栏横导航。
- **Don't** 把观测写成账单或把缺口画成零。
- **Don't** 引入 UI 框架、远程字体或 CDN。
- **Don't** 在主区重复当前页面标题。

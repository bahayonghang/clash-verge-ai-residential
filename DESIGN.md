---
name: 家宽流量监控
description: 深色侧栏工作台，neko 令牌
colors:
  background: "#0b0f19"
  background-light: "#f5f7fa"
  sidebar: "#0f1420"
  sidebar-light: "#ffffff"
  sidebar-text: "#f8fafc"
  accent: "#3b82f6"
  accent-light: "#0063ff"
  card: "#171c2b"
  card-light: "#ffffff"
  ink: "#f8fafc"
  muted: "#94a3b8"
  chart-1: "#3b82f6"
  chart-2: "#8b5cf6"
  chart-3: "#06b6d4"
  chart-4: "#10b981"
  chart-5: "#f59e0b"
  danger: "#ef4444"
  focus: "#3b82f6"
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
  sm: "0.5rem"
  md: "0.75rem"
  lg: "0.75rem"
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
  nav-item:
    backgroundColor: "transparent"
    textColor: "{colors.sidebar-text}"
    rounded: "{rounded.md}"
    padding: "0.75rem 0.75rem"
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

桌面工作台。左侧深色认页，右侧读数区。蓝只出现在当前栏和主按钮。工艺对齐 Clash Verge Rev 与 neko：图标加文字的侧栏、圆角选中条、扁平面板。

密度是专家工具档。表格保持表格。缺口写「未知」，不写零，不写账单。

**Key Characteristics:**

- 四款主题：Latte / Frappé / Macchiato / Mocha
- 蓝只打选中项和主操作
- 本地生成图标，不走 CDN
- 系统无衬线 + 等宽数字

## Colors

主色是侧栏和选中蓝。浅色 Latte 用 `#f5f7fa` 背景与 `#0063ff` 主色。深色三档共用 `#3b82f6` 与 `--chart-1..5`。

### Primary
- **选中蓝**（深色 `#3b82f6`，Latte `#0063ff`）：当前导航和主按钮。

### Neutral
- **侧栏**（Mocha `#0f1420`，Latte `#ffffff`）：应用壳左栏。
- **主区**（Mocha `#0b0f19`，Latte `#f5f7fa`）：工作面。
- **卡片**（Mocha `#171c2b`，Latte `#ffffff`）：指标和表单面板。

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

侧栏宽度 160–352px，默认 220px。主区独立滚动。指标网格自适应。打印隐藏侧栏和按钮。顶栏只放工具，不做主认页。

## Elevation & Depth

层次靠色块与细边：侧栏、卡片、表格。

## Shapes

控件与面板圆角 0.75rem。选中导航是圆角条，不是下划线。

## Components

### Buttons
- **Shape:** 0.5–0.75rem
- **Primary:** 主题 `--primary`，白字
- **Focus:** 3px ring

### Cards / Containers
- 卡片面板，0.75rem 角
- 细边，轻阴影

### Inputs / Fields
- 标签在字段上方
- 焦点用 ring

### Navigation
- 左栏图标 + `titleZh`
- 当前项整行蓝底
- Recovery-only 不渲染九段业务导航

### Data table
- 语义 `<table>`，`aria-sort`
- 等宽数字

## Do's and Don'ts

### Do:
- **Do** 把产品名和页面标题放在左侧栏。
- **Do** 缺口、未知显示「未知」。
- **Do** 用本地图标，侧栏按钮同时有图和字。

### Don't:
- **Don't** 恢复顶栏横导航。
- **Don't** 把观测写成账单或把缺口画成零。
- **Don't** 引入远程字体或 CDN。
- **Don't** 在主区重复当前页面标题。

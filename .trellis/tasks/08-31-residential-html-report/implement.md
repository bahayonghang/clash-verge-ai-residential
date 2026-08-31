# 执行计划

按序执行。验证命令默认 cwd 为仓库根。

## 检查单

1. **C3 HTML 渲染**
   - `ExportService::render_html`；升级 `write_html` 主阅读区；i18n 新键。
   - 测试：可读字段、`metadata_line` 仍在、无 `http://` / secret / `<script`、`render_html` 与落盘正文一致。
   - 验证：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml export_tests -- --nocapture`

2. **C3 家宽手动回看**
   - `is_residential_manual_report` + `load_latest_residential_manual`。
   - 测试：最新家宽行命中；自动 hour 与非家宽 manual 跳过。
   - 验证：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml load_latest_residential -- --nocapture`

3. **C2 command**
   - `AppFacade::render_report_html` / `get_latest_residential_manual`；`lib.rs` 注册。
   - 验证：`cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace` 中相关模块。

4. **Hook + 模型**
   - `isResidentialManualReport`；`restoreResidentialManual`；`decodeHtmlDocument`。
   - 测试：纯函数 + hook 辅助。

5. **家宽页 UI**
   - 创建/查看、时间行、Dialog + iframe。
   - 测试：`report-section.test.ts` / `index.test.tsx` 源码契约。

6. **门禁**
   - `npm --prefix residential-monitor run typecheck && npm --prefix residential-monitor run lint && npm --prefix residential-monitor test`
   - `cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check`
   - `cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings`
   - `cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace`

## 回滚点

- 步骤 1–2 为 C3 增量，可独立 revert。
- 步骤 3 增加 command，需与前端同回。
- 无 schema / migration。

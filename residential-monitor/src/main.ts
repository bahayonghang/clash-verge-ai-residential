import {
  decodeAlertCenter,
  decodeDiagnostics,
  decodeReportResult,
  decodeShellStatus,
  type AlertCenterPage,
  type AlertRule,
  type BootstrapDto,
  type CloseState,
  type DiagnosticsSnapshot,
  type LiveOverview,
  type NotifyCapability,
  type ReportQuery,
  type ReportResult,
  type RetentionPreview,
  type RouteId
} from "./dto";
import { decodeMonitorMessage } from "./ipc/decoder";
import {
  emptyMonitorState,
  markCloseAccepted,
  reduceMonitor,
  visibleRows,
  type MonitorState
} from "./ipc/reducer";
import { formatBytes, formatUtc, unknownOr } from "./format/units";

const HEALTH_ZH: Record<string, { title: string; action: string }> = {
  connecting: { title: "正在连接控制器", action: "等待连接完成" },
  connected: { title: "已连接", action: "无需操作" },
  tcp_unauthorized: { title: "TCP 鉴权失败", action: "检查 secret 后重试" },
  pipe_access_denied: { title: "管道访问被拒绝", action: "改用 TCP External Controller" },
  pipe_busy_timeout: { title: "管道忙超时", action: "稍后重试或改用 TCP" },
  endpoint_missing: { title: "控制器端点不存在", action: "检查地址或重新发现" },
  protocol_incompatible: { title: "协议不兼容", action: "启用 TCP External Controller" },
  pid_mismatch: { title: "管道进程身份不匹配", action: "重新发现后改用 TCP" },
  core_restarted: { title: "核心已重启", action: "等待重新建立会话" },
  cancelled: { title: "操作已取消", action: "可立即重连" },
  non_loopback: { title: "拒绝非回环地址", action: "改为 127.0.0.1" },
  storage_failure: { title: "存储故障", action: "打开恢复界面检查磁盘" },
  no_data: { title: "暂无采样", action: "确认采集已启动" }
};

function metric(label: string, value: number | null): string {
  return `<div class="metric"><span>${label}</span><strong>${formatBytes(value)}</strong></div>`;
}

function renderOverview(overview: LiveOverview): string {
  const health = HEALTH_ZH[overview.health.session] ?? {
    title: overview.health.session,
    action: "查看诊断"
  };
  const categories = Object.keys(overview.categoryUpload)
    .map((name) => `<li>${name}：${formatBytes(overview.categoryUpload[name] ?? null)}</li>`)
    .join("");
  const coverage = overview.coverageKind
    ? `<p class="gap">覆盖：${overview.coverageKind} / ${unknownOr(overview.coverageReason)}。缺口不显示为零。</p>`
    : `<p>覆盖：采集中。最后采样 ${formatUtc(overview.lastSampleUtc)}</p>`;
  return `
    <section class="grid" aria-label="实时口径">
      ${metric("控制器 meter 上行", overview.meterUpload)}
      ${metric("控制器 meter 下行", overview.meterDownload)}
      ${metric("可归因观测上行", overview.attributedUpload)}
      ${metric("可归因观测下行", overview.attributedDownload)}
      ${metric("其他连接上行", overview.otherUpload)}
      ${metric("未归因 gap 上行", overview.gapUpload)}
      ${metric("over-attributed 上行", overview.overUpload)}
      <div class="metric"><span>活跃连接</span><strong>${overview.activeCount}</strong></div>
    </section>
    <section class="panel">
      <h2>重点分类</h2>
      <ul>${categories || "<li>无</li>"}</ul>
      ${coverage}
      <p class="status" data-state="${overview.health.session}">${health.title}。下一步：${health.action}</p>
    </section>
  `;
}

function renderLive(state: MonitorState): string {
  const rows = visibleRows(state.connections, 0, 20, 4)
    .map((row) => {
      const mark = state.closeMarks.get(row.identity);
      const closeLabel =
        mark === "accepted" ? "已发送关闭请求" : mark === "closed" ? "已关闭" : mark === "unconfirmed" ? "未确认" : "关闭";
      return `<tr>
        <td>${unknownOr(row.host)}</td>
        <td>${unknownOr(row.processName)}</td>
        <td>${unknownOr(row.primary)}</td>
        <td>${formatBytes(row.upload)}</td>
        <td>${formatBytes(row.download)}</td>
        <td>${unknownOr(row.network)}</td>
        <td><button type="button" data-close="${row.identity}" ${mark ? "disabled" : ""}>${closeLabel}</button></td>
      </tr>`;
    })
    .join("");
  return `
    <section class="panel">
      <h2>实时连接</h2>
      <p>列表按稳定 identity 排序。关闭全部连接入口不存在。</p>
      <table class="data">
        <thead><tr><th>域名</th><th>进程</th><th>主分类</th><th>上行</th><th>下行</th><th>网络</th><th>操作</th></tr></thead>
        <tbody>${rows || `<tr><td colspan="7">无数据</td></tr>`}</tbody>
      </table>
    </section>
  `;
}

function defaultReportQuery(): ReportQuery {
  const end = Math.floor(Date.now() / 1000);
  return {
    rangeStartUtc: end - 3600,
    rangeEndUtc: end,
    displayTimezone: "local",
    granularity: "hour",
    filters: { category: null, host: null, process: null, rule: null, chain: null, network: null },
    grouping: "host",
    targetPolicy: "historical",
    comparison: { previousEqualWindow: true },
    sort: { field: "download", descending: true },
    page: { limit: 200, after: null },
    topN: 20,
    includeSessions: false
  };
}

function renderReports(report: ReportResult | null, statusZh: string): string {
  const seriesRows =
    report?.series
      .map(
        (point) =>
          `<tr><td>${formatUtc(point.bucketUtc)}</td><td>${formatBytes(point.upload)}</td><td>${formatBytes(point.download)}</td></tr>`
      )
      .join("") ?? "";
  const rankRows =
    report?.rankings
      .map((row) => {
        const max = report.rankings[0]?.download || 1;
        const width = Math.max(4, Math.round((row.download / max) * 100));
        return `<tr><td>${row.label}</td><td>${formatBytes(row.upload)}</td><td>${formatBytes(row.download)}</td><td><span class="bar" style="width:${width}%"></span></td></tr>`;
      })
      .join("") ?? "";
  const capability = report
    ? `<p>${report.drilldownCapability.noteZh} 数据层 ${report.dataTier}。${report.policyMetadata.noteZh}</p>
       <p>覆盖 ${report.coverage.status}，缺口 ${report.coverage.gapSec} 秒。单位 ${report.unit}。</p>`
    : "";
  return `
    <section class="panel">
      <h2>分析报告</h2>
      <p>图表与数据表使用同一 ReportResult。观测下界，不是账单。</p>
      <label>预设
        <select id="report-preset">
          <option value="hour">最近一小时</option>
          <option value="day">指定日（近 24 小时）</option>
          <option value="7">近 7 日</option>
          <option value="30">近 30 日</option>
          <option value="month">自然月（近 30 日）</option>
        </select>
      </label>
      <label>粒度
        <select id="report-granularity">
          <option value="hour">小时</option>
          <option value="day">日</option>
          <option value="month">月</option>
        </select>
      </label>
      <label>排名维度
        <select id="report-grouping">
          <option value="host">域名</option>
          <option value="process">进程</option>
          <option value="rule">规则</option>
          <option value="chain">链路</option>
          <option value="network">网络类型</option>
          <option value="category">分类</option>
        </select>
      </label>
      <button type="button" id="run-report">运行报告</button>
      <button type="button" id="export-csv">导出 CSV</button>
      <button type="button" id="export-json">导出 JSON</button>
      <button type="button" id="export-html">导出 HTML</button>
      <p class="status" data-state="${report ? "connected" : "no_data"}">${statusZh}</p>
      ${capability}
    </section>
    <section class="panel" aria-label="报告数字">
      <h2>总量（与导出同一 token）</h2>
      ${report ? `<p>上行 ${formatBytes(report.totals.upload)} / 下行 ${formatBytes(report.totals.download)} / 连接 ${report.totals.connectionCount}</p>` : "<p>尚未运行报告。</p>"}
    </section>
    <section class="panel">
      <h2>趋势图对应数据表</h2>
      <table class="data"><thead><tr><th>时间</th><th>上行</th><th>下行</th></tr></thead>
      <tbody>${seriesRows || `<tr><td colspan="3">无数据</td></tr>`}</tbody></table>
    </section>
    <section class="panel">
      <h2>精确 Top N</h2>
      <table class="data"><thead><tr><th>名称</th><th>上行</th><th>下行</th><th>占比条</th></tr></thead>
      <tbody>${rankRows || `<tr><td colspan="4">无数据或能力不支持</td></tr>`}</tbody></table>
    </section>
  `;
}

function renderSettings(boot: BootstrapDto): string {
  return `
    <section class="panel">
      <h2>设置向导</h2>
      <ol>
        <li>控制器发现与测试（仅 loopback TCP）</li>
        <li>重点目标选择与排序</li>
        <li>登录自启动需再次确认后才会写入本机</li>
        <li>保留与本地隐私说明：数据只留本机，secret 不进 SQLite</li>
        <li>通知能力预检：本阶段不发送系统通知</li>
      </ol>
      <label>控制器地址
        <input id="controller-address" value="${boot.settings.address || "127.0.0.1:9090"}" />
      </label>
      <label>TCP secret（不会回显到日志或 Channel）
        <input id="controller-secret" type="password" autocomplete="off" />
      </label>
      <label>重点目标（逗号分隔）
        <input id="targets" value="家宽" />
      </label>
      <p>凭据状态：${boot.settings.hasSecret ? "已配置" : "未配置"}，模式 ${boot.settings.secretMode}</p>
      <button type="button" id="save-settings">保存设置</button>
    </section>
    <section class="panel">
      <h2>数据管理</h2>
      <p>备份使用 Online Backup，不复制热库。恢复失败保留当前库。secret 不随备份迁移。</p>
      <button type="button" id="create-backup">创建备份</button>
      <button type="button" id="restore-backup">恢复备份</button>
      <button type="button" id="retention-preview">保留预览</button>
      <button type="button" id="run-retention">物化汇总（不自动删除）</button>
      <p id="data-note">自动 DELETE 在守恒门通过前保持关闭。不自动 VACUUM。</p>
    </section>
  `;
}

function renderRecovery(boot: BootstrapDto): string {
  const recovery = boot.recovery;
  if (!recovery) {
    return `<section class="panel"><p>恢复信息不可用。</p></section>`;
  }
  const backups = recovery.backups.map((item) => `<li>${item}</li>`).join("");
  return `
    <section class="panel recovery">
      <h2>Recovery Shell</h2>
      <p>应用版本 ${recovery.appVersion}，数据库版本 ${recovery.userVersion}，支持上限 ${recovery.supportedMax}。</p>
      <p>${recovery.future ? "数据库版本高于应用，已 fail closed。" : "数据库无法按普通 schema 打开。"}</p>
      <p>${recovery.restoreNoteZh}</p>
      <button type="button" id="restore-backup" ${recovery.restoreAvailable ? "" : "disabled"}>执行恢复</button>
      <h3>migration backup</h3>
      <ul>${backups || "<li>无</li>"}</ul>
    </section>
  `;
}

function renderUnavailable(name: string, until: string): string {
  return `<section class="panel"><h2>${name}</h2><p>此页面尚未交付，由 ${until} 接入。不显示伪数据。</p></section>`;
}

function renderAlerts(
  page: AlertCenterPage | null,
  statusZh: string,
  diagnostics: DiagnosticsSnapshot | null,
  notify: NotifyCapability | null
): string {
  const rows =
    page?.items
      .map((item) => {
        const observed =
          item.evidence.observedValue === null ? "未知" : String(item.evidence.observedValue);
        const recovered = item.resolvedUtc === null ? "—" : formatUtc(item.resolvedUtc);
        return `<tr>
          <td>${item.ruleId} v${item.ruleVersion}</td>
          <td>${item.status}</td>
          <td>${item.selectorIdentity}</td>
          <td>${observed}</td>
          <td>${item.evidence.coverageSummary}</td>
          <td>${recovered}</td>
          <td>${item.evidence.notEvaluableReason ?? "—"}</td>
        </tr>`;
      })
      .join("") ?? "";
  const notifyZh = notify
    ? `${notify.available ? "通知 seam 可用" : "通知不可用"}。${notify.reasonZh}`
    : "尚未检测通知能力。";
  const diag = diagnostics
    ? `<p>应用 ${diagnostics.appVersion}，schema ${diagnostics.sqliteUserVersion} / ${diagnostics.supportedSchema}，WAL ${diagnostics.journalMode} / ${diagnostics.synchronous}。</p>
       <p>传输 ${diagnostics.controllerTransportStatus}，覆盖 ${diagnostics.coverageSummary}，watermark ${diagnostics.writerWatermark}，活动告警 ${diagnostics.alertActive}，outbox ${diagnostics.outboxBacklog}。</p>
       <p>${diagnostics.backupRetentionNoteZh} ${diagnostics.reconnectHintZh}</p>`
    : "<p>尚未生成诊断。</p>";
  return `
    <section class="panel">
      <h2>告警中心</h2>
      <p>应用内记录是权威来源。系统通知是尽力送达。静默只抑制通知，不删除事件。</p>
      <p class="status">${statusZh}</p>
      <p>${notifyZh}</p>
      <button type="button" id="refresh-alerts">刷新告警</button>
      <button type="button" id="test-notification">测试通知</button>
      <button type="button" id="export-diagnostics">导出诊断</button>
    </section>
    <section class="panel">
      <h2>规则</h2>
      <p>速率单位 bps，周期用量单位 byte。恢复阈值必须小于触发阈值。时区用于自然日 / 月。</p>
      <label>规则 ID <input id="alert-rule-id" value="rate-home" /></label>
      <label>类型
        <select id="alert-kind">
          <option value="rate">速率</option>
          <option value="period-usage">周期用量</option>
          <option value="health">健康</option>
        </select>
      </label>
      <label>对象
        <select id="alert-selector-kind">
          <option value="primary-category">主分类</option>
          <option value="domain">域名</option>
          <option value="process">进程</option>
          <option value="health-kind">健康根因</option>
        </select>
      </label>
      <label>选择值 <input id="alert-selector-value" value="家宽" /></label>
      <label>方向
        <select id="alert-direction">
          <option value="download">下行</option>
          <option value="upload">上行</option>
          <option value="combined">合计</option>
        </select>
      </label>
      <label>触发阈值 <input id="alert-threshold" type="number" value="1000000" /></label>
      <label>恢复阈值 <input id="alert-recovery" type="number" value="400000" /></label>
      <label>周期
        <select id="alert-period">
          <option value="">无</option>
          <option value="rolling-1h">滚动 1 小时</option>
          <option value="local-day">本地自然日</option>
          <option value="local-month">本地自然月</option>
        </select>
      </label>
      <label>时区 <input id="alert-timezone" value="Asia/Shanghai" /></label>
      <button type="button" id="save-alert-rule">保存规则</button>
    </section>
    <section class="panel">
      <h2>活动与历史</h2>
      <table class="data">
        <thead><tr><th>规则</th><th>状态</th><th>对象</th><th>观测值</th><th>覆盖</th><th>恢复时间</th><th>不可评估</th></tr></thead>
        <tbody>${rows || `<tr><td colspan="7">无告警</td></tr>`}</tbody>
      </table>
    </section>
    <section class="panel">
      <h2>脱敏诊断</h2>
      ${diag}
    </section>
  `;
}



function navHtml(active: RouteId, routes: BootstrapDto["routes"]): string {
  return routes
    .map((route) => {
      const current = route.id === active ? "aria-current=\"page\"" : "";
      const disabled = route.available ? "" : "data-disabled=\"true\"";
      return `<button type="button" class="nav-item" data-route="${route.id}" ${current} ${disabled}>${route.titleZh}</button>`;
    })
    .join("");
}

function previewBootstrap(): BootstrapDto {
  return {
    schemaVersion: 1,
    branch: "normal-ready",
    routes: [
      { id: "overview", titleZh: "概览", available: true, unavailableUntil: null },
      { id: "live", titleZh: "实时连接", available: true, unavailableUntil: null },
      { id: "reports", titleZh: "分析报告", available: true, unavailableUntil: null },
      { id: "alerts", titleZh: "告警", available: true, unavailableUntil: null },
      { id: "settings-data", titleZh: "设置 / 数据管理", available: true, unavailableUntil: null }
    ],
    overview: {
      schemaVersion: 1,
      meterUpload: null,
      meterDownload: null,
      attributedUpload: null,
      attributedDownload: null,
      categoryUpload: {},
      categoryDownload: {},
      otherUpload: null,
      otherDownload: null,
      gapUpload: null,
      gapDownload: null,
      overUpload: null,
      overDownload: null,
      activeCount: 0,
      lastSampleUtc: null,
      coverageKind: null,
      coverageReason: null,
      health: { session: "no_data", storageOk: true, storageReason: null }
    },
    settings: {
      transport: "tcp",
      address: "",
      credentialTarget: "io.github.bahayonghang.residential-monitor/controller",
      hasSecret: false,
      secretMode: "none"
    },
    wizardComplete: false,
    recovery: null,
    launchMode: "interactive"
  };
}

async function invokeCommand<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  const api = (
    globalThis as {
      __TAURI_INTERNALS__?: { invoke: (cmd: string, args?: Record<string, unknown>) => Promise<T> };
    }
  ).__TAURI_INTERNALS__;
  if (!api) {
    throw new Error("not-tauri");
  }
  return api.invoke(name, args);
}

function renderApp(
  root: HTMLElement,
  boot: BootstrapDto,
  state: MonitorState,
  route: RouteId,
  report: ReportResult | null,
  reportStatus: string,
  alerts: AlertCenterPage | null,
  alertStatus: string,
  diagnostics: DiagnosticsSnapshot | null,
  notify: NotifyCapability | null
): void {
  const body =
    boot.branch === "recovery-only"
      ? renderRecovery(boot)
      : route === "overview"
        ? renderOverview(state.snapshot ?? boot.overview)
        : route === "live"
          ? renderLive(state)
          : route === "settings-data"
            ? renderSettings(boot)
            : route === "reports"
              ? renderReports(report, reportStatus)
              : route === "alerts"
                ? renderAlerts(alerts, alertStatus, diagnostics, notify)
                : renderUnavailable("告警", "C4");
  root.innerHTML = `
    <header class="top">
      <h1>家宽流量监控</h1>
      <p>观测下界，不是账单。secret 不会出现在此页面。</p>
    </header>
    <nav class="nav" aria-label="主导航">${navHtml(route, boot.routes)}</nav>
    <div id="view">${body}</div>
    ${state.errorZh ? `<p class="gap" role="alert">${state.errorZh}</p>` : ""}
  `;
}

async function main(): Promise<void> {
  const app = document.querySelector("#app");
  if (!(app instanceof HTMLElement)) {
    return;
  }
  decodeShellStatus({
    schemaVersion: 1,
    kind: "shellStatus",
    identifier: "io.github.bahayonghang.residential-monitor",
    phase: "c2-shell",
    messageZh: "桌面外壳与实时监控"
  });

  let boot = previewBootstrap();
  try {
    boot = await invokeCommand<BootstrapDto>("get_bootstrap");
  } catch {
    boot = previewBootstrap();
  }

  let route: RouteId = boot.branch === "recovery-only" ? "settings-data" : "overview";
  let state = emptyMonitorState();
  let report: ReportResult | null = null;
  let reportStatus = "尚未运行报告。";
  let alerts: AlertCenterPage | null = null;
  let alertStatus = "尚未加载告警。";
  let diagnostics: DiagnosticsSnapshot | null = null;
  let notify: NotifyCapability | null = null;
  state.snapshot = boot.overview;
  renderApp(app, boot, state, route, report, reportStatus, alerts, alertStatus, diagnostics, notify);

  const apply = (next: MonitorState, nextRoute = route): void => {
    state = next;
    route = nextRoute;
    renderApp(app, boot, state, route, report, reportStatus, alerts, alertStatus, diagnostics, notify);
  };

  const applyReport = (next: ReportResult | null, status: string): void => {
    report = next;
    reportStatus = status;
    renderApp(app, boot, state, route, report, reportStatus, alerts, alertStatus, diagnostics, notify);
  };

  const applyAlerts = (
    next: AlertCenterPage | null,
    status: string,
    nextDiag = diagnostics,
    nextNotify = notify
  ): void => {
    alerts = next;
    alertStatus = status;
    diagnostics = nextDiag;
    notify = nextNotify;
    renderApp(app, boot, state, route, report, reportStatus, alerts, alertStatus, diagnostics, notify);
  };

  const buildQuery = (): ReportQuery => {
    const query = defaultReportQuery();
    const preset = (document.querySelector("#report-preset") as HTMLSelectElement | null)?.value ?? "hour";
    const now = Math.floor(Date.now() / 1000);
    const spans: Record<string, number> = { hour: 3600, day: 86400, "7": 7 * 86400, "30": 30 * 86400, month: 30 * 86400 };
    query.rangeStartUtc = now - (spans[preset] ?? 3600);
    query.rangeEndUtc = now;
    query.granularity = ((document.querySelector("#report-granularity") as HTMLSelectElement | null)?.value ??
      "hour") as ReportQuery["granularity"];
    query.grouping = ((document.querySelector("#report-grouping") as HTMLSelectElement | null)?.value ??
      "host") as ReportQuery["grouping"];
    return query;
  };

  app.addEventListener("click", async (event) => {
    const target = event.target;
    if (!(target instanceof HTMLElement)) {
      return;
    }
    const nextRoute = target.dataset.route as RouteId | undefined;
    if (nextRoute) {
      apply(state, nextRoute);
      if (nextRoute === "alerts") {
        try {
          const page = decodeAlertCenter(
            await invokeCommand("list_alert_center", { status: null, after: null })
          );
          const diag = decodeDiagnostics(await invokeCommand("get_diagnostics"));
          applyAlerts(page, `已加载 ${page.items.length} 条。`, diag, notify);
        } catch {
          applyAlerts(alerts, "告警中心暂不可用。");
        }
      }
      return;
    }
    const closeId = target.dataset.close;
    if (closeId) {
      try {
        const result = await invokeCommand<CloseState>("close_connection", {
          identity: closeId,
          requestId: `ui-${Date.now()}`
        });
        if (result.mark === "accepted") {
          apply(markCloseAccepted(state, closeId));
        }
      } catch {
        apply({ ...state, errorZh: "关闭请求未发送。未向未隔离控制器发出 DELETE。" });
      }
    }
    if (target.id === "save-settings") {
      const address = (document.querySelector("#controller-address") as HTMLInputElement | null)?.value ?? "";
      const secret = (document.querySelector("#controller-secret") as HTMLInputElement | null)?.value;
      const targets = (document.querySelector("#targets") as HTMLInputElement | null)?.value ?? "";
      try {
        boot.settings = await invokeCommand("save_settings", {
          address,
          secret: secret && secret.length > 0 ? secret : null,
          sessionOnly: true
        });
        await invokeCommand("save_targets", {
          targets: targets
            .split(",")
            .map((item) => item.trim())
            .filter(Boolean)
        });
        apply(state, "overview");
      } catch {
        apply({ ...state, errorZh: "设置保存失败。请检查回环地址。" });
      }
    }
    if (target.id === "run-report") {
      applyReport(report, "正在查询…");
      try {
        const raw = await invokeCommand<unknown>("run_report", { query: buildQuery() });
        const decoded = decodeReportResult(raw);
        applyReport(decoded, `已生成快照 ${decoded.reportSnapshotToken.slice(0, 8)}。`);
      } catch (error) {
        applyReport(null, error instanceof Error ? error.message : "报告失败。");
      }
    }
    if (target.id === "export-csv" || target.id === "export-json" || target.id === "export-html") {
      if (!report) {
        applyReport(null, "请先运行报告。");
        return;
      }
      const format = target.id === "export-csv" ? "csv" : target.id === "export-json" ? "json" : "html";
      try {
        const picked = await invokeCommand<string | null>("pick_file", {
          purpose: "report-export",
          mode: "save"
        });
        if (!picked) {
          applyReport(report, "已取消导出。");
          return;
        }
        await invokeCommand("export_report", {
          token: report.reportSnapshotToken,
          spec: {
            format,
            includeSeries: true,
            includeRankings: true,
            includeSessions: false,
            redactHost: "none",
            redactProcess: "none"
          },
          path: picked
        });
        applyReport(report, `已导出 ${format.toUpperCase()}。`);
      } catch {
        applyReport(report, "导出失败。未覆盖已有文件。");
      }
    }
    if (target.id === "create-backup" || target.id === "restore-backup") {
      try {
        const picked = await invokeCommand<string | null>("pick_file", {
          purpose: target.id === "create-backup" ? "backup-create" : "backup-restore",
          mode: target.id === "create-backup" ? "save" : "open"
        });
        if (!picked) {
          return;
        }
        if (target.id === "create-backup") {
          await invokeCommand("create_backup", { path: picked });
          apply({ ...state, errorZh: null });
        } else {
          await invokeCommand("restore_backup", { path: picked });
        }
      } catch {
        apply({ ...state, errorZh: "备份或恢复失败。当前可用库未覆盖。" });
      }
    }
    if (target.id === "retention-preview" || target.id === "run-retention") {
      try {
        const preview = await invokeCommand<RetentionPreview>(
          target.id === "retention-preview" ? "retention_preview" : "run_retention",
          target.id === "run-retention" ? { delete: false } : undefined
        );
        const note = document.querySelector("#data-note");
        if (note) {
          note.textContent = `${preview.noteZh} raw ${preview.rawRows} 行，hourly ${preview.hourlyRows} 行。自动删除=${preview.autoDeleteEnabled}`;
        }
      } catch {
        apply({ ...state, errorZh: "保留预览失败。" });
      }
    }
    if (target.id === "refresh-alerts") {
      try {
        const page = decodeAlertCenter(
          await invokeCommand("list_alert_center", { status: null, after: null })
        );
        const diag = decodeDiagnostics(await invokeCommand("get_diagnostics"));
        applyAlerts(page, `已刷新 ${page.items.length} 条。`, diag, notify);
      } catch {
        applyAlerts(alerts, "刷新失败。");
      }
    }
    if (target.id === "test-notification") {
      try {
        const cap = await invokeCommand<NotifyCapability>("test_notification");
        applyAlerts(alerts, cap.available ? "已提交测试通知。" : cap.reasonZh, diagnostics, cap);
      } catch {
        applyAlerts(alerts, "测试通知失败。应用内记录仍可用。");
      }
    }
    if (target.id === "export-diagnostics") {
      try {
        const path = await invokeCommand<string | null>("pick_file", {
          purpose: "diagnostics-export",
          mode: "save"
        });
        if (path) {
          await invokeCommand("export_diagnostics", { path });
          applyAlerts(alerts, "诊断已导出。");
        }
      } catch {
        applyAlerts(alerts, "诊断导出失败。采集与告警未中断。");
      }
    }
    if (target.id === "save-alert-rule") {
      const period = (document.querySelector("#alert-period") as HTMLSelectElement | null)?.value ?? "";
      const rule: AlertRule = {
        ruleId: (document.querySelector("#alert-rule-id") as HTMLInputElement | null)?.value ?? "rate-home",
        version: 1,
        enabled: true,
        kind: ((document.querySelector("#alert-kind") as HTMLSelectElement | null)?.value ?? "rate") as AlertRule["kind"],
        selectorKind: ((document.querySelector("#alert-selector-kind") as HTMLSelectElement | null)?.value ??
          "primary-category") as AlertRule["selectorKind"],
        selectorValue: (document.querySelector("#alert-selector-value") as HTMLInputElement | null)?.value || null,
        direction: ((document.querySelector("#alert-direction") as HTMLSelectElement | null)?.value ??
          "download") as AlertRule["direction"],
        thresholdValue: Number((document.querySelector("#alert-threshold") as HTMLInputElement | null)?.value ?? "1"),
        recoveryThreshold: Number((document.querySelector("#alert-recovery") as HTMLInputElement | null)?.value ?? "0"),
        period: period === "" ? null : (period as AlertRule["period"]),
        timezone: (document.querySelector("#alert-timezone") as HTMLInputElement | null)?.value ?? "UTC",
        cooldownSec: 300,
        quietStartMin: null,
        quietEndMin: null,
        createdUtc: 0,
        updatedUtc: 0
      };
      try {
        await invokeCommand("upsert_alert_rule", { rule });
        applyAlerts(alerts, "规则已保存。新版本不会继承旧连续命中。");
      } catch {
        applyAlerts(alerts, "规则无效。请检查滞回、时区和周期。");
      }
    }
  });

  try {
    const first = await invokeCommand<unknown>("get_bootstrap");
    if (first && typeof first === "object") {
      const maybe = first as BootstrapDto;
      if (maybe.overview) {
        boot = maybe;
        state.snapshot = maybe.overview;
        apply(state, route);
      }
    }
  } catch {
    /* 预览态没有 Tauri */
  }

  window.addEventListener("message", (event) => {
    try {
      const message = decodeMonitorMessage(event.data);
      apply(reduceMonitor(state, message));
    } catch {
      /* 非 Channel 消息 */
    }
  });
}

void main();

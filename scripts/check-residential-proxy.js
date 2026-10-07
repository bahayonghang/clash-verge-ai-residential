"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { execFile } = require("node:child_process");
const { parseLocalToml } = require("./sync-local-config.js");

const root = path.resolve(__dirname, "..");
const DEFAULT_CONFIG_PATH = path.join(root, "clash-verge-ai-residential.local.toml");
const DEFAULT_TIMEOUT_SECONDS = 15;

function createPainter(stream = process.stdout) {
  const enabled = Boolean(
    stream &&
    stream.isTTY &&
    process.env.TERM !== "dumb" &&
    !process.env.NO_COLOR
  );
  const wrap = (code) => (text) => (enabled ? `\x1b[${code}m${text}\x1b[0m` : String(text));

  return {
    bold: wrap("1"),
    dim: wrap("2"),
    ok: wrap("32"),
    warn: wrap("33"),
    error: wrap("31"),
    cyan: wrap("36"),
    blue: wrap("34"),
    magenta: wrap("35"),
    gray: wrap("90")
  };
}

function maskPassword(password) {
  if (!password) return "******";
  return "******";
}

function runCurl(args, options = {}) {
  const timeoutMs = (options.timeoutSeconds || DEFAULT_TIMEOUT_SECONDS) * 1000;
  return new Promise((resolve) => {
    const startTime = Date.now();
    execFile(
      "curl",
      args,
      {
        timeout: timeoutMs + 3000,
        windowsHide: true,
        maxBuffer: 1024 * 1024
      },
      (error, stdout, stderr) => {
        const durationMs = Date.now() - startTime;
        if (error) {
          resolve({
            ok: false,
            exitCode: typeof error.code === "number" ? error.code : 1,
            stdout: (stdout || "").trim(),
            stderr: (stderr || error.message || "").trim(),
            durationMs
          });
        } else {
          resolve({
            ok: true,
            exitCode: 0,
            stdout: (stdout || "").trim(),
            stderr: (stderr || "").trim(),
            durationMs
          });
        }
      }
    );
  });
}

function parseCommandLineArgs(argv) {
  const args = argv.slice(2);
  let configPath = null;
  let timeoutSeconds = DEFAULT_TIMEOUT_SECONDS;
  let showHelp = false;

  for (let i = 0; i < args.length; i += 1) {
    const arg = args[i];
    if (arg === "--help" || arg === "-h") {
      showHelp = true;
    } else if (arg === "--timeout" || arg === "-t") {
      i += 1;
      const parsed = Number(args[i]);
      if (Number.isFinite(parsed) && parsed > 0) {
        timeoutSeconds = parsed;
      }
    } else if (!arg.startsWith("-") && !configPath) {
      configPath = path.resolve(process.cwd(), arg);
    }
  }

  return {
    configPath: configPath || DEFAULT_CONFIG_PATH,
    timeoutSeconds,
    showHelp
  };
}

function printHelp(paint) {
  console.log(`
${paint.bold("用法:")}
  node scripts/check-residential-proxy.js [配置文件路径] [选项]
  just check-proxy [配置文件路径]

${paint.bold("参数:")}
  ${paint.cyan("[配置文件路径]")}  目标 TOML 文件 (默认: clash-verge-ai-residential.local.toml)

${paint.bold("选项:")}
  ${paint.cyan("--timeout, -t <秒>")}  设置单次请求超时时间 (默认: ${DEFAULT_TIMEOUT_SECONDS} 秒)
  ${paint.cyan("--help, -h")}          显示本说明信息
`);
}

function loadProxyConfig(configPath) {
  if (!fs.existsSync(configPath)) {
    throw new Error(`配置文件不存在：${configPath}\n💡 请先执行 just render-local 生成配置文件并填写住宅代理信息。`);
  }

  const content = fs.readFileSync(configPath, "utf8");
  const parsed = parseLocalToml(content);
  const homeProxy = parsed.homeProxy;

  if (!homeProxy) {
    throw new Error("配置文件缺少 [home_proxy] 配置块。");
  }

  const requiredFields = ["server", "port", "username", "password"];
  for (const field of requiredFields) {
    if (!homeProxy[field]) {
      throw new Error(`[home_proxy] 缺少必需字段: ${field}`);
    }
  }

  if (homeProxy.server === "xxx" || homeProxy.username === "xxx" || homeProxy.password === "xxx") {
    throw new Error("住宅代理配置仍为示例占位符 (xxx)，请先在配置文件中填写实际的代理信息。");
  }

  return homeProxy;
}

async function testProxyConnectivity(homeProxy, timeoutSeconds) {
  const proxyUrl = `socks5h://${homeProxy.username}:${homeProxy.password}@${homeProxy.server}:${homeProxy.port}`;
  const displayCommand = `curl --proxy socks5h://${homeProxy.username}:${maskPassword(homeProxy.password)}@${homeProxy.server}:${homeProxy.port} https://ipinfo.io`;

  const curlResult = await runCurl([
    "--proxy", proxyUrl,
    "--silent",
    "--show-error",
    "--connect-timeout", String(Math.min(10, Math.floor(timeoutSeconds))),
    "--max-time", String(Math.floor(timeoutSeconds)),
    "https://ipinfo.io"
  ], { timeoutSeconds });

  if (!curlResult.ok) {
    return {
      ok: false,
      displayCommand,
      curlResult
    };
  }

  let basicInfo = null;
  try {
    basicInfo = JSON.parse(curlResult.stdout);
  } catch (error) {
    return {
      ok: false,
      displayCommand,
      curlResult,
      parseError: `响应不是合法 JSON: ${error.message}`
    };
  }

  const exitIp = basicInfo.ip;
  let detailData = null;

  if (exitIp) {
    // 优先通过代理查询 ipinfo widget demo 端点获取 ASN 类型与 Privacy 明细
    const detailProxyResult = await runCurl([
      "--proxy", proxyUrl,
      "--silent",
      "--connect-timeout", "6",
      "--max-time", "10",
      `https://ipinfo.io/widget/demo/${exitIp}`
    ], { timeoutSeconds: 10 });

    if (detailProxyResult.ok) {
      try {
        const parsed = JSON.parse(detailProxyResult.stdout);
        detailData = parsed.data || null;
      } catch {}
    }

    // 若代理端点未取到，降级为直连查询
    if (!detailData) {
      const detailDirectResult = await runCurl([
        "--silent",
        "--connect-timeout", "6",
        "--max-time", "10",
        `https://ipinfo.io/widget/demo/${exitIp}`
      ], { timeoutSeconds: 10 });

      if (detailDirectResult.ok) {
        try {
          const parsed = JSON.parse(detailDirectResult.stdout);
          detailData = parsed.data || null;
        } catch {}
      }
    }
  }

  return {
    ok: true,
    displayCommand,
    durationMs: curlResult.durationMs,
    basicInfo,
    detailData
  };
}

function evaluateIpSafety({ basicInfo, detailData, homeProxy }) {
  const exitIp = basicInfo?.ip || "";
  const server = homeProxy?.server || "";
  const ipMatchesServer = exitIp === server;

  let asnType = (detailData?.asn?.type || "").toLowerCase();
  let asnNumber = detailData?.asn?.asn || "";
  let asnName = detailData?.asn?.name || "";
  const org = basicInfo?.org || "";

  if (!asnNumber && org) {
    const match = org.match(/^(AS\d+)\s*(.*)$/);
    if (match) {
      asnNumber = match[1];
      asnName = match[2];
    }
  }

  let asnTypeEstimated = false;
  if (!asnType && org) {
    const lowerOrg = org.toLowerCase();
    const hostingKeywords = [
      "hosting", "cloud", "server", "datacenter", "digitalocean",
      "amazon", "alibaba", "tencent", "google", "microsoft", "oracle",
      "ovh", "linode", "vultr", "hetzner", "choopa", "m247", "leaseweb"
    ];
    const ispKeywords = [
      "telecom", "broadband", "cable", "communications", "verizon",
      "comcast", "at&t", "charter", "spectrum", "cox", "centurylink",
      "frontier", "bell", "rogers", "shaw", "vodafone", "deutsche telekom"
    ];
    if (hostingKeywords.some((kw) => lowerOrg.includes(kw))) {
      asnType = "hosting";
      asnTypeEstimated = true;
    } else if (ispKeywords.some((kw) => lowerOrg.includes(kw))) {
      asnType = "isp";
      asnTypeEstimated = true;
    }
  }

  const isIsp = asnType === "isp";
  const isHosting = asnType === "hosting" || Boolean(detailData?.is_hosting) || Boolean(detailData?.privacy?.hosting);
  const isBusiness = asnType === "business";

  const privacy = detailData?.privacy || null;
  let privacyChecked = false;
  let allPrivacyFalse = true;
  const privacyFlags = {
    vpn: false,
    proxy: false,
    tor: false,
    relay: false,
    hosting: false
  };

  if (privacy) {
    privacyChecked = true;
    for (const key of Object.keys(privacyFlags)) {
      const val = Boolean(privacy[key]);
      privacyFlags[key] = val;
      if (val) allPrivacyFalse = false;
    }
  }

  // 依据 ref/safe-claude/README.md 判定标准：
  // 1. ASN type 值为 ISP
  // 2. Privacy 检测值全部为 False
  let status = "PASS";
  let gradeText = "合格 (纯净静态住宅 IP)";
  const reasons = [];
  let advice = "";

  if (isHosting) {
    status = "FAIL";
    gradeText = "不合格 (机房 / 数据中心 IP)";
    reasons.push("ASN 类型为 Hosting 或检测为数据中心机房 IP");
    advice = "强烈建议不要用于 Claude 账号或 Claude Code！请参考 safe-claude 指南，联系代理服务商（如 iproyal 24h 内）更换为真正的家庭宽带 IP。";
  } else if (privacyChecked && !allPrivacyFalse) {
    status = "WARN";
    gradeText = "风险 (检出隐私或代理伪装标记)";
    const detected = Object.entries(privacyFlags).filter(([, v]) => v).map(([k]) => k.toUpperCase());
    reasons.push(`Privacy 检出标记项: ${detected.join(", ")}`);
    advice = "虽然 ASN 标注为家庭宽带，但检测到伪装代理或中继特征。若服务商支持更换，建议联系客服申请更换更纯净的住宅 IP。";
  } else if (isBusiness) {
    status = "WARN";
    gradeText = "注意 (企业商业宽带 IP)";
    reasons.push("ASN 类型为 Business，非纯居民住宅宽带");
    advice = "商业宽带风控严格度高于家庭住宅，建议配合前置链式代理，并留意 Claude 等服务使用情况。";
  } else if (isIsp && allPrivacyFalse) {
    status = "PASS";
    gradeText = "合格 (纯净静态住宅 IP)";
    advice = "完全符合 safe-claude 安全指南标准！ASN Type 为 ISP 且 Privacy 项均为 False，可放心用于 Claude Code 及风控敏感的 AI 业务。";
  } else {
    status = "WARN";
    gradeText = "未完全确认 (需人工核实)";
    advice = "未获取到完整 ASN 详情，建议点击下方 Ping0 与 IPinfo 链接人工核验 IP 类型与风控值。";
  }

  return {
    exitIp,
    server,
    ipMatchesServer,
    asnType,
    asnTypeEstimated,
    asnNumber,
    asnName: asnName || (detailData?.company?.name || ""),
    isIsp,
    isHosting,
    isBusiness,
    privacy,
    privacyChecked,
    privacyFlags,
    allPrivacyFalse,
    status,
    gradeText,
    reasons,
    advice,
    links: {
      ipinfo: `https://ipinfo.io/${exitIp}`,
      ping0: `https://ping0.cc/ip/${exitIp}`
    }
  };
}

function renderReport({ configPath, homeProxy, connectivity, evaluation, paint }) {
  const line = paint.dim("─".repeat(62));
  const doubleLine = paint.cyan("═".repeat(62));

  console.log("");
  console.log(doubleLine);
  console.log(`  ${paint.bold("🏠 静态住宅代理配置检测与安全评估 (safe-claude 规范)")}`);
  console.log(doubleLine);

  // [1/3] 代理配置
  console.log(`\n${paint.cyan("▶ [1/3] 代理配置信息")}`);
  console.log(line);
  console.log(`  • 配置文件: ${path.relative(process.cwd(), configPath)}`);
  console.log(`  • 代理名称: ${paint.bold(homeProxy.name || "未命名")} (${homeProxy.type || "socks5"})`);
  console.log(`  • 目标端点: ${paint.bold(`${homeProxy.server}:${homeProxy.port}`)}`);
  console.log(`  • 认证账号: ${homeProxy.username} / ${maskPassword(homeProxy.password)}`);
  console.log(`  • 前置代理: ${homeProxy["dialer-proxy"] || "无"} (dialer-proxy)`);

  // [2/3] 连通性测试
  console.log(`\n${paint.cyan("▶ [2/3] 连通性测试 (curl)")}`);
  console.log(line);
  console.log(`  • 执行命令: ${paint.dim(connectivity.displayCommand)}`);

  if (!connectivity.ok) {
    const err = connectivity.curlResult?.stderr || connectivity.parseError || "连接未知错误";
    console.log(`  • 测试状态: ${paint.error("✗ 连接失败")}`);
    console.log(`  • 错误详情: ${paint.error(err)}`);
    console.log(`\n${paint.warn("💡 排查建议（参考 ref/safe-claude/README.md）:")}`);
    console.log(`  1. 国内网络通常无法直连境外住宅代理 IP，需要科学上网前置代理支撑。`);
    console.log(`  2. 请确认已在本地开启科学上网（Clash Verge TUN 虚拟网卡模式或系统代理）。`);
    console.log(`  3. 请核对配置文件的 server、port、username、password 是否正确且套餐未到期。`);
    console.log(doubleLine);
    return;
  }

  const { basicInfo, durationMs } = connectivity;
  const ipMatchDesc = evaluation.ipMatchesServer
    ? paint.ok(" (与配置端点一致)")
    : paint.warn(` (与配置端点 ${homeProxy.server} 不同，网关轮换)`);

  console.log(`  • 握手状态: ${paint.ok("✓ 连接成功")} ${paint.dim(`(耗时: ${durationMs} ms)`)}`);
  console.log(`  • 出口 IP : ${paint.bold(basicInfo.ip)}${ipMatchDesc}`);
  console.log(`  • 归属地区: ${[basicInfo.city, basicInfo.region, basicInfo.country].filter(Boolean).join(", ")} ${paint.dim(`(${basicInfo.timezone || "时区未知"})`)}`);
  console.log(`  • 组织信息: ${basicInfo.org || "未知"}`);

  // [3/3] 安全与风控判定
  console.log(`\n${paint.cyan("▶ [3/3] 安全与风控判定 (依据 ref/safe-claude/README.md)")}`);
  console.log(line);

  // ASN Type 行
  let asnDisplay = paint.gray("未检测到");
  if (evaluation.asnType === "isp") {
    asnDisplay = `${paint.ok("✓ ISP (家庭宽带)")} ${paint.ok("[符合要求]")}`;
  } else if (evaluation.asnType === "business") {
    asnDisplay = `${paint.warn("⚠ Business (商业/企业宽带)")} ${paint.warn("[中度风险]")}`;
  } else if (evaluation.asnType === "hosting") {
    asnDisplay = `${paint.error("✗ Hosting (数据中心机房)")} ${paint.error("[极高风险 - 易封号]")}`;
  } else if (evaluation.asnType) {
    asnDisplay = `${paint.warn(evaluation.asnType.toUpperCase())} ${paint.dim("(估算)")}`;
  }
  console.log(`  • ASN 类型: ${asnDisplay}`);
  if (evaluation.asnNumber || evaluation.asnName) {
    console.log(`  • 自治域名: ${[evaluation.asnNumber, evaluation.asnName].filter(Boolean).join(" ")}`);
  }

  // Privacy 检测矩阵
  console.log("  • Privacy 伪装检测:");
  if (evaluation.privacyChecked) {
    const items = [
      { key: "vpn", label: "VPN" },
      { key: "proxy", label: "Proxy" },
      { key: "tor", label: "Tor" },
      { key: "relay", label: "Relay" },
      { key: "hosting", label: "Hosting" }
    ];
    items.forEach((item, index) => {
      const isLast = index === items.length - 1;
      const branch = isLast ? "    └─" : "    ├─";
      const isFlagged = evaluation.privacyFlags[item.key];
      const valText = isFlagged
        ? paint.error("✗ True  [检出异常]")
        : paint.ok("✓ False [安全纯净]");
      console.log(`${branch} ${item.label.padEnd(8)}: ${valText}`);
    });
  } else {
    console.log(`    ${paint.warn("⚠ 无法获取 ipinfo 详细隐私检测数据，已根据 org 评估。")}`);
  }

  // 综合判定结论
  console.log("\n" + line);
  let statusBadge = paint.ok(`✓ ${evaluation.gradeText}`);
  if (evaluation.status === "WARN") {
    statusBadge = paint.warn(`⚠ ${evaluation.gradeText}`);
  } else if (evaluation.status === "FAIL") {
    statusBadge = paint.error(`✗ ${evaluation.gradeText}`);
  }

  console.log(`  ${paint.bold("判定结论:")} ${statusBadge}`);
  if (evaluation.reasons.length > 0) {
    for (const reason of evaluation.reasons) {
      console.log(`  ${paint.dim("原因说明:")} ${reason}`);
    }
  }
  console.log(`  ${paint.bold("操作建议:")} ${evaluation.advice}`);

  console.log(`\n  ${paint.dim("人工核验链接（safe-claude 推荐）:")}`);
  console.log(`    • IPinfo: ${paint.cyan(evaluation.links.ipinfo)}`);
  console.log(`    • Ping0:  ${paint.cyan(evaluation.links.ping0)}`);
  console.log(doubleLine + "\n");
}

async function checkResidentialProxy(options = {}) {
  const configPath = options.configPath || DEFAULT_CONFIG_PATH;
  const timeoutSeconds = options.timeoutSeconds || DEFAULT_TIMEOUT_SECONDS;
  const paint = options.paint || createPainter(process.stdout);

  const homeProxy = loadProxyConfig(configPath);
  const connectivity = await testProxyConnectivity(homeProxy, timeoutSeconds);

  let evaluation = null;
  if (connectivity.ok) {
    evaluation = evaluateIpSafety({
      basicInfo: connectivity.basicInfo,
      detailData: connectivity.detailData,
      homeProxy
    });
  }

  renderReport({
    configPath,
    homeProxy,
    connectivity,
    evaluation,
    paint
  });

  if (!connectivity.ok) {
    return { success: false, status: "DISCONNECTED" };
  }

  return {
    success: evaluation.status !== "FAIL",
    status: evaluation.status,
    evaluation,
    connectivity
  };
}

async function main() {
  const paint = createPainter(process.stdout);
  const { configPath, timeoutSeconds, showHelp } = parseCommandLineArgs(process.argv);

  if (showHelp) {
    printHelp(paint);
    process.exit(0);
  }

  try {
    const result = await checkResidentialProxy({
      configPath,
      timeoutSeconds,
      paint
    });
    if (!result.success) {
      process.exitCode = 1;
    }
  } catch (error) {
    console.error(`\n${paint.error("✗ 检测失败:")} ${error.message}\n`);
    process.exitCode = 1;
  }
}

if (require.main === module) {
  main();
}

module.exports = {
  createPainter,
  maskPassword,
  runCurl,
  parseCommandLineArgs,
  loadProxyConfig,
  testProxyConnectivity,
  evaluateIpSafety,
  renderReport,
  checkResidentialProxy,
  main
};

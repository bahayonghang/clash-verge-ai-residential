"use strict";

const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const DEFAULT_TEMPLATE_PATH = path.join(root, "clash-verge-ai-residential.js");
const DEFAULT_CONFIG_PATH = path.join(root, "clash-verge-ai-residential.local.toml");
const DEFAULT_EXAMPLE_PATH = path.join(
  root,
  "clash-verge-ai-residential.local.toml.example"
);
const DEFAULT_OUTPUT_PATH = path.join(root, "clash-verge-ai-residential.local.js");
const REQUIRED_KEYS = [
  "name",
  "type",
  "server",
  "port",
  "username",
  "password",
  "udp",
  "dialer-proxy"
];
const SWITCH_CONFIG_FIELDS = Object.freeze([
  { table: "routing", key: "openai_shared_dependencies", constant: "ROUTE_OPENAI_SHARED_DEPENDENCIES", type: "boolean" },
  { table: "routing", key: "openai_core", constant: "ROUTE_OPENAI_CORE", type: "boolean" },
  { table: "routing", key: "claude_shared_dependencies", constant: "ROUTE_CLAUDE_SHARED_DEPENDENCIES", type: "boolean" },
  { table: "routing", key: "antigravity_google_auth", constant: "ROUTE_ANTIGRAVITY_GOOGLE_AUTH", type: "boolean" },
  { table: "routing", key: "antigravity_project_apis", constant: "ROUTE_ANTIGRAVITY_PROJECT_APIS", type: "boolean" },
  { table: "routing", key: "antigravity_update_and_telemetry", constant: "ROUTE_ANTIGRAVITY_UPDATE_AND_TELEMETRY", type: "boolean" },
  { table: "routing", key: "gemini_web_core", constant: "ROUTE_GEMINI_WEB_CORE", type: "boolean" },
  { table: "routing", key: "cursor_core", constant: "ROUTE_CURSOR_CORE", type: "boolean" },
  { table: "routing", key: "cursor_repository_indexing", constant: "ROUTE_CURSOR_REPOSITORY_INDEXING", type: "boolean" },
  { table: "routing", key: "grok_core", constant: "ROUTE_GROK_CORE", type: "boolean" },
  { table: "routing", key: "cursor_process_fallback", constant: "ROUTE_CURSOR_PROCESS_FALLBACK", type: "boolean" },
  { table: "routing", key: "claude_code_auxiliary", constant: "ROUTE_CLAUDE_CODE_AUXILIARY", type: "boolean" },
  { table: "routing", key: "ai_process_fallback", constant: "ENABLE_AI_PROCESS_FALLBACK", type: "boolean" },
  { table: "routing", key: "anthropic_ip_fallback", constant: "ENABLE_ANTHROPIC_IP_FALLBACK", type: "boolean" },
  { table: "routing", key: "shared_realtime_infrastructure", constant: "ROUTE_SHARED_REALTIME_INFRASTRUCTURE", type: "boolean" },
  { table: "routing", key: "global_realtime_ports", constant: "ROUTE_GLOBAL_REALTIME_PORTS", type: "boolean" },
  { table: "routing", key: "public_encrypted_dns", constant: "ROUTE_PUBLIC_ENCRYPTED_DNS", type: "boolean" },
  { table: "runtime", key: "allow_final_rule_upstream_fallback", constant: "ALLOW_FINAL_RULE_UPSTREAM_FALLBACK", type: "boolean" },
  { table: "runtime", key: "allow_heuristic_upstream_fallback", constant: "ALLOW_HEURISTIC_UPSTREAM_FALLBACK", type: "boolean" },
  { table: "runtime", key: "preserve_unmanaged_nameserver_policy", constant: "PRESERVE_UNMANAGED_NAMESERVER_POLICY", type: "boolean" },
  { table: "runtime", key: "enable_domain_sniffer", constant: "ENABLE_DOMAIN_SNIFFER", type: "boolean" },
  { table: "runtime", key: "harden_existing_tun_dns_hijack", constant: "HARDEN_EXISTING_TUN_DNS_HIJACK", type: "boolean" },
  { table: "runtime", key: "enable_tun_strict_route", constant: "ENABLE_TUN_STRICT_ROUTE", type: "boolean" },
  { table: "runtime", key: "warn_on_reachable_udp_disabled", constant: "WARN_ON_REACHABLE_UDP_DISABLED", type: "boolean" }
].map((field) => Object.freeze(field)));
const SWITCH_TABLES = Object.freeze([
  ...new Set(SWITCH_CONFIG_FIELDS.map((field) => field.table))
]);
const SUPPORTED_TABLES = new Set(["home_proxy", ...SWITCH_TABLES]);

function configurationError(message) {
  return new Error(`本地 TOML 配置无效：${message}`);
}

function stripComment(line) {
  let quote = null;
  let escaped = false;

  for (let index = 0; index < line.length; index += 1) {
    const character = line[index];

    if (quote === "\"") {
      if (escaped) {
        escaped = false;
      } else if (character === "\\") {
        escaped = true;
      } else if (character === quote) {
        quote = null;
      }
      continue;
    }

    if (quote === "'") {
      if (character === quote) quote = null;
      continue;
    }

    if (character === "\"" || character === "'") {
      quote = character;
    } else if (character === "#") {
      return line.slice(0, index);
    }
  }

  return line;
}

function parseBasicString(value, lineNumber) {
  if (value[0] !== "\"") {
    throw configurationError(`第 ${lineNumber} 行的字符串必须以双引号开始`);
  }

  let result = "";
  for (let index = 1; index < value.length; index += 1) {
    const character = value[index];
    if (character === "\"") {
      if (index !== value.length - 1) {
        throw configurationError(`第 ${lineNumber} 行的字符串结束后包含额外内容`);
      }
      return result;
    }
    if (character !== "\\") {
      if (character.codePointAt(0) < 0x20) {
        throw configurationError(`第 ${lineNumber} 行的字符串包含控制字符`);
      }
      result += character;
      continue;
    }

    index += 1;
    const escape = value[index];
    if (escape === undefined) {
      throw configurationError(`第 ${lineNumber} 行的字符串转义不完整`);
    }

    const escapes = {
      b: "\b",
      t: "\t",
      n: "\n",
      f: "\f",
      r: "\r",
      '\"': "\"",
      "\\": "\\"
    };
    if (Object.hasOwn(escapes, escape)) {
      result += escapes[escape];
      continue;
    }

    const hexLength = escape === "u" ? 4 : escape === "U" ? 8 : 0;
    if (hexLength === 0) {
      throw configurationError(`第 ${lineNumber} 行包含不支持的 \\${escape} 转义`);
    }

    const hex = value.slice(index + 1, index + 1 + hexLength);
    if (!new RegExp(`^[0-9a-fA-F]{${hexLength}}$`).test(hex)) {
      throw configurationError(`第 ${lineNumber} 行的 Unicode 转义无效`);
    }

    const codePoint = Number.parseInt(hex, 16);
    if (codePoint > 0x10ffff || (codePoint >= 0xd800 && codePoint <= 0xdfff)) {
      throw configurationError(`第 ${lineNumber} 行的 Unicode 转义不是有效标量值`);
    }
    result += String.fromCodePoint(codePoint);
    index += hexLength;
  }

  throw configurationError(`第 ${lineNumber} 行的字符串必须以双引号闭合`);
}

function parseLiteralString(value, lineNumber) {
  if (value[0] !== "'") {
    throw configurationError(`第 ${lineNumber} 行的字符串必须以单引号开始`);
  }

  const closingQuote = value.indexOf("'", 1);
  if (closingQuote === -1) {
    throw configurationError(`第 ${lineNumber} 行的字符串必须以单引号闭合`);
  }
  if (closingQuote !== value.length - 1) {
    throw configurationError(`第 ${lineNumber} 行的字符串结束后包含额外内容`);
  }
  return value.slice(1, -1);
}

function parseValue(value, lineNumber) {
  if (value.startsWith("\"")) return parseBasicString(value, lineNumber);
  if (value.startsWith("'")) return parseLiteralString(value, lineNumber);
  if (value === "true") return true;
  if (value === "false") return false;

  if (/^[0-9](?:_?[0-9])*$/.test(value)) {
    return Number(value.replaceAll("_", ""));
  }

  throw configurationError(`第 ${lineNumber} 行只支持字符串、整数和布尔值`);
}

function parseLocalToml(source) {
  const values = {
    homeProxy: {}
  };
  for (const table of SWITCH_TABLES) values[table] = {};
  const seenTables = new Set();
  let currentTable = null;

  for (const [offset, rawLine] of source.replace(/^\uFEFF/, "").split(/\r?\n/).entries()) {
    const lineNumber = offset + 1;
    const line = stripComment(rawLine).trim();
    if (!line) continue;

    const tableMatch = line.match(/^\[([A-Za-z0-9_-]+)\]$/);
    if (tableMatch) {
      const table = tableMatch[1];
      if (!SUPPORTED_TABLES.has(table)) {
        throw configurationError(`第 ${lineNumber} 行包含未知配置表 [${table}]`);
      }
      if (seenTables.has(table)) {
        throw configurationError(`第 ${lineNumber} 行重复定义 [${table}]`);
      }
      seenTables.add(table);
      currentTable = table;
      continue;
    }

    if (line.startsWith("[") || line.endsWith("]")) {
      throw configurationError(`第 ${lineNumber} 行不是受支持的配置表声明`);
    }

    if (!currentTable) {
      throw configurationError(`第 ${lineNumber} 行必须位于配置表内`);
    }

    const assignment = line.match(/^([A-Za-z0-9_-]+)\s*=\s*(.*)$/);
    if (!assignment) {
      throw configurationError(`第 ${lineNumber} 行不是有效的 key = value`);
    }

    const [, key, rawValue] = assignment;
    const switchField = SWITCH_CONFIG_FIELDS.find(
      (field) => field.table === currentTable && field.key === key
    );
    if (currentTable === "home_proxy" && !REQUIRED_KEYS.includes(key)) {
      throw configurationError(`第 ${lineNumber} 行包含未知字段 home_proxy.${key}`);
    }
    if (currentTable !== "home_proxy" && !switchField) {
      throw configurationError(`第 ${lineNumber} 行包含未知字段 ${currentTable}.${key}`);
    }

    const target = currentTable === "home_proxy" ? values.homeProxy : values[currentTable];
    if (Object.hasOwn(target, key)) {
      throw configurationError(`第 ${lineNumber} 行重复定义字段 ${currentTable}.${key}`);
    }
    const parsedValue = parseValue(rawValue.trim(), lineNumber);
    if (switchField && typeof parsedValue !== switchField.type) {
      throw configurationError(`第 ${lineNumber} 行字段 ${currentTable}.${key} 必须是 true 或 false`);
    }
    target[key] = parsedValue;
  }

  if (!seenTables.has("home_proxy")) {
    throw configurationError("缺少 [home_proxy] 表");
  }
  return values;
}

function parseHomeProxyToml(source) {
  return parseLocalToml(source).homeProxy;
}

function detectEol(source) {
  const crlfCount = (source.match(/\r\n/g) || []).length;
  const lineFeedCount = (source.match(/\n/g) || []).length;
  return crlfCount > lineFeedCount - crlfCount ? "\r\n" : "\n";
}

function validateExampleSwitchDefaults(exampleConfig) {
  for (const field of SWITCH_CONFIG_FIELDS) {
    const table = exampleConfig[field.table];
    if (!table || !Object.hasOwn(table, field.key)) {
      throw new Error(
        `示例 TOML 缺少 ${field.table}.${field.key}；请先补齐 example 再同步`
      );
    }
    if (typeof table[field.key] !== field.type) {
      throw new Error(
        `示例 TOML 字段 ${field.table}.${field.key} 类型与 ${field.constant} 声明不符`
      );
    }
  }
}

// 定位每个已声明表头在文本中的区块：表头行号与区块内最后一个非空行号。
// 行标注复用 parseLocalToml 的表头正则，保证补全器与解析器看到相同结构。
function locateTableBlocks(lines) {
  const blocks = [];

  for (const [index, rawLine] of lines.entries()) {
    const line = stripComment(rawLine).trim();
    const tableMatch = line.match(/^\[([A-Za-z0-9_-]+)\]$/);
    if (tableMatch) {
      blocks.push({ table: tableMatch[1], headerIndex: index, lastNonEmptyIndex: index });
      continue;
    }
    if (blocks.length > 0 && line) {
      blocks[blocks.length - 1].lastNonEmptyIndex = index;
    }
  }

  return blocks;
}

// 文本级补全：只为 SWITCH_CONFIG_FIELDS 声明的开关键追加缺失行，
// 不重排、不改写用户已有键值、注释与空行；home_proxy 凭据仍要求手填。
function completeLocalToml(localSource, localConfig, exampleConfig) {
  const missingFields = SWITCH_CONFIG_FIELDS.filter(
    (field) => !Object.hasOwn(localConfig[field.table], field.key)
  );
  if (missingFields.length === 0) return null;

  const hasBom = localSource.startsWith("\uFEFF");
  const body = hasBom ? localSource.slice(1) : localSource;
  const eol = detectEol(body);
  const hadTrailingNewline = /(?:\r?\n)$/.test(body);
  const lines = body.replace(/\r?\n$/, "").split(/\r?\n/);
  if (lines.length === 1 && lines[0] === "") lines.pop();

  const blocks = locateTableBlocks(lines);
  const blockByTable = new Map(blocks.map((block) => [block.table, block]));

  // 按表分组缺失键，保持 SWITCH_CONFIG_FIELDS 声明顺序。
  const missingByTable = new Map();
  for (const field of missingFields) {
    if (!missingByTable.has(field.table)) missingByTable.set(field.table, []);
    missingByTable.get(field.table).push(field);
  }

  const insertions = [];
  for (const [table, fields] of missingByTable) {
    const newLines = fields.map(
      (field) => `${field.key} = ${exampleConfig[table][field.key]}`
    );

    const block = blockByTable.get(table);
    if (block) {
      // 插入区块内最后一个非空行之后，避免打断表头下方的说明注释。
      insertions.push({ index: block.lastNonEmptyIndex + 1, lines: newLines });
    } else {
      // 整表缺失：文件末尾追加，空行分隔后重建表头。
      const leadingBlank = lines.length > 0 && lines[lines.length - 1] !== ""
        ? [""]
        : [];
      insertions.push({
        index: lines.length,
        lines: [...leadingBlank, `[${table}]`, ...newLines],
        appendTable: true
      });
    }
  }

  // 从后往前插入，避免前面的插入点行号失真；同一插入点（例如已有表区块
  // 尾部恰为文件末尾）必须先 splice 整表追加、后 splice 键追加，
  // 键追加才会落在新表头之前而不是新表内部。
  insertions.sort((a, b) => {
    if (b.index !== a.index) return b.index - a.index;
    return (b.appendTable ? 1 : 0) - (a.appendTable ? 1 : 0);
  });
  for (const insertion of insertions) {
    lines.splice(insertion.index, 0, ...insertion.lines);
  }

  const completed = (hasBom ? "\uFEFF" : "") +
    lines.join(eol) +
    (hadTrailingNewline ? eol : "");
  return {
    source: completed,
    addedKeys: missingFields.map((field) => `${field.table}.${field.key}`)
  };
}

function extractHomeProxyName(templateSource) {
  const match = templateSource.match(/const\s+HOME_PROXY_NAME\s*=\s*("(?:\\.|[^"\\])*")\s*;/);
  if (!match) {
    throw new Error("无法从模板读取 HOME_PROXY_NAME");
  }
  return JSON.parse(match[1]);
}

function validateHomeProxyConfig(config, homeProxyName) {
  for (const key of REQUIRED_KEYS) {
    if (!Object.hasOwn(config, key)) {
      throw configurationError(`缺少字段 ${key}`);
    }
  }

  if (config.name !== homeProxyName) {
    throw configurationError(`name 必须与模板中的 HOME_PROXY_NAME（${homeProxyName}）一致`);
  }
  if (config.type !== "socks5") {
    throw configurationError('type 必须为 "socks5"');
  }
  if (typeof config.server !== "string" || !config.server) {
    throw configurationError("server 必须是非空字符串");
  }
  if (!Number.isInteger(config.port) || config.port < 1 || config.port > 65535) {
    throw configurationError("port 必须是 1-65535 的整数");
  }
  for (const key of ["username", "password", "dialer-proxy"]) {
    if (typeof config[key] !== "string") {
      throw configurationError(`${key} 必须是字符串`);
    }
  }
  if (!config["dialer-proxy"]) {
    throw configurationError("dialer-proxy 不能为空");
  }
  if (typeof config.udp !== "boolean") {
    throw configurationError("udp 必须是 true 或 false");
  }
}

function validateLocalConfig(config, homeProxyName) {
  if (!config || typeof config !== "object" || Array.isArray(config)) {
    throw configurationError("配置根必须是对象");
  }
  validateHomeProxyConfig(config.homeProxy, homeProxyName);

  for (const table of SWITCH_TABLES) {
    const values = config[table];
    if (!values || typeof values !== "object" || Array.isArray(values)) {
      throw configurationError(`[${table}] 必须是配置表`);
    }
    const knownFields = new Map(
      SWITCH_CONFIG_FIELDS
        .filter((field) => field.table === table)
        .map((field) => [field.key, field])
    );
    for (const [key, value] of Object.entries(values)) {
      const field = knownFields.get(key);
      if (!field) {
        throw configurationError(`包含未知字段 ${table}.${key}`);
      }
      if (typeof value !== field.type) {
        throw configurationError(`字段 ${table}.${key} 必须是 true 或 false`);
      }
    }
  }
}

function renderHomeProxyTemplate(config) {
  return [
    "const HOME_PROXY_TEMPLATE = {",
    "  name: HOME_PROXY_NAME,",
    `  type: ${JSON.stringify(config.type)},`,
    `  server: ${JSON.stringify(config.server)},`,
    `  port: ${config.port},`,
    `  username: ${JSON.stringify(config.username)},`,
    `  password: ${JSON.stringify(config.password)},`,
    `  udp: ${config.udp},`,
    `  "dialer-proxy": ${JSON.stringify(config["dialer-proxy"])}`,
    "};"
  ].join("\n");
}

function injectHomeProxyTemplate(templateSource, renderedTemplate) {
  const pattern = /const\s+HOME_PROXY_TEMPLATE\s*=\s*\{[\s\S]*?\n\};/g;
  const matches = templateSource.match(pattern);
  if (!matches || matches.length !== 1) {
    throw new Error("模板中必须且只能包含一个 HOME_PROXY_TEMPLATE");
  }
  return templateSource.replace(pattern, renderedTemplate);
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function booleanConstantPattern(constantName) {
  return new RegExp(
    `^([ \\t]*const[ \\t]+${escapeRegExp(constantName)}[ \\t]*=[ \\t]*)(true|false)([ \\t]*;[ \\t]*)(\\r?)$`,
    "gm"
  );
}

function injectBooleanConstants(templateSource, config) {
  let output = templateSource;

  for (const field of SWITCH_CONFIG_FIELDS) {
    const table = config[field.table];
    if (!Object.hasOwn(table, field.key)) continue;

    const pattern = booleanConstantPattern(field.constant);
    const matches = output.match(pattern);
    if (!matches || matches.length !== 1) {
      throw new Error(`模板中必须且只能包含一个布尔常量 ${field.constant}`);
    }
    output = output.replace(
      pattern,
      (match, before, current, after, carriageReturn) =>
        `${before}${table[field.key]}${after}${carriageReturn}`
    );
  }

  return output;
}

function writeFileAtomically(outputPath, content) {
  const temporaryPath = `${outputPath}.${process.pid}.tmp`;
  try {
    fs.writeFileSync(temporaryPath, content, { encoding: "utf8", mode: 0o600 });
    fs.renameSync(temporaryPath, outputPath);
  } catch (error) {
    fs.rmSync(temporaryPath, { force: true });
    throw error;
  }
}

function syncLocalConfig({
  templatePath = DEFAULT_TEMPLATE_PATH,
  configPath = DEFAULT_CONFIG_PATH,
  outputPath = DEFAULT_OUTPUT_PATH,
  examplePath = DEFAULT_EXAMPLE_PATH
} = {}) {
  const resolvedTemplatePath = path.resolve(templatePath);
  const resolvedConfigPath = path.resolve(configPath);
  const resolvedOutputPath = path.resolve(outputPath);
  if (resolvedOutputPath === resolvedTemplatePath) {
    throw new Error("本地输出路径不能覆盖公开模板");
  }
  if (resolvedOutputPath === resolvedConfigPath) {
    throw new Error("本地输出路径不能覆盖 TOML 配置");
  }
  if (!fs.existsSync(resolvedConfigPath)) {
    throw new Error(`找不到本地配置：${configPath}`);
  }

  const templateSource = fs.readFileSync(resolvedTemplatePath, "utf8");
  const exampleConfig = parseLocalToml(
    fs.readFileSync(path.resolve(examplePath), "utf8")
  );
  validateExampleSwitchDefaults(exampleConfig);

  const localSource = fs.readFileSync(resolvedConfigPath, "utf8");
  let config = parseLocalToml(localSource);
  let addedKeys = [];

  // 本地 TOML 缺失的开关键按 example 默认值补全后再渲染，
  // 已有键值、注释与行尾风格保持逐字不变。
  const completion = completeLocalToml(localSource, config, exampleConfig);
  if (completion) {
    writeFileAtomically(resolvedConfigPath, completion.source);
    config = parseLocalToml(completion.source);
    addedKeys = completion.addedKeys;
  }

  validateLocalConfig(config, extractHomeProxyName(templateSource));

  const banner = [
    "/*",
    ` * 由 ${path.basename(resolvedTemplatePath)} 与 ${path.basename(resolvedConfigPath)} 自动生成。`,
    " * 请编辑 TOML 后重新生成，不要直接修改此文件。",
    " */",
    ""
  ].join("\n");
  const renderedTemplate = renderHomeProxyTemplate(config.homeProxy);
  const output = banner + injectBooleanConstants(
    injectHomeProxyTemplate(templateSource, renderedTemplate),
    config
  );
  writeFileAtomically(resolvedOutputPath, output);

  return {
    configPath: resolvedConfigPath,
    outputPath: resolvedOutputPath,
    addedKeys,
    addedDefaults: addedKeys.map((key) => {
      const separator = key.indexOf(".");
      const table = key.slice(0, separator);
      const field = key.slice(separator + 1);
      return { key, value: config[table][field] };
    })
  };
}

// 轻量 ANSI 着色：遵循 NO_COLOR 与 FORCE_COLOR 约定，仅在交互终端启用；
// Windows 下要求宿主是 Windows Terminal、VS Code 等现代终端，避免传统
// conhost 打印转义符原文。
function colorEnabled(stream) {
  if (process.env.NO_COLOR) return false;
  if (process.env.FORCE_COLOR === "0") return false;
  if (process.env.FORCE_COLOR) return true;
  if (stream.isTTY !== true) return false;
  if (process.platform === "win32") {
    return Boolean(
      process.env.WT_SESSION ||
        process.env.TERM_PROGRAM ||
        process.env.ConEmuANSI === "ON" ||
        process.env.TERM
    );
  }
  return true;
}

function createPainter(stream) {
  const enabled = colorEnabled(stream);
  const wrap = (code) => (text) => (enabled ? `\x1b[${code}m${text}\x1b[0m` : text);
  return {
    ok: wrap("32"),
    warn: wrap("33"),
    error: wrap("31"),
    cyan: wrap("36"),
    dim: wrap("2")
  };
}

function main() {
  const [configArgument, outputArgument] = process.argv.slice(2);
  const result = syncLocalConfig({
    configPath: configArgument ? path.resolve(process.cwd(), configArgument) : DEFAULT_CONFIG_PATH,
    outputPath: outputArgument ? path.resolve(process.cwd(), outputArgument) : DEFAULT_OUTPUT_PATH
  });
  const paint = createPainter(process.stdout);
  const configName = path.basename(result.configPath);
  const outputName = path.basename(result.outputPath);

  if (result.addedDefaults.length > 0) {
    console.log(
      `${paint.warn("+")} 已按示例默认值补全 ${result.addedDefaults.length} 个缺失开关到 ${paint.cyan(configName)}：`
    );
    for (const { key, value } of result.addedDefaults) {
      console.log(`    ${paint.cyan(key)} ${paint.dim(`= ${value}`)}`);
    }
  }
  console.log(
    `${paint.ok("✓")} 已同步 ${paint.cyan(configName)} ${paint.dim("→")} ${paint.cyan(outputName)}`
  );
}

if (require.main === module) {
  try {
    main();
  } catch (error) {
    const paint = createPainter(process.stderr);
    console.error(`${paint.error("✗")} ${error.message}`);
    process.exitCode = 1;
  }
}

module.exports = {
  SWITCH_CONFIG_FIELDS,
  parseLocalToml,
  parseHomeProxyToml,
  completeLocalToml,
  detectEol,
  validateLocalConfig,
  validateHomeProxyConfig,
  renderHomeProxyTemplate,
  injectBooleanConstants,
  syncLocalConfig
};

"use strict";

const fs = require("node:fs");
const path = require("node:path");

const root = path.resolve(__dirname, "..");
const DEFAULT_TEMPLATE_PATH = path.join(root, "clash-verge-ai-residential.js");
const DEFAULT_CONFIG_PATH = path.join(root, "clash-verge-ai-residential.local.toml");
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

function parseHomeProxyToml(source) {
  const values = {};
  let inHomeProxyTable = false;

  for (const [offset, rawLine] of source.replace(/^\uFEFF/, "").split(/\r?\n/).entries()) {
    const lineNumber = offset + 1;
    const line = stripComment(rawLine).trim();
    if (!line) continue;

    if (line === "[home_proxy]") {
      if (inHomeProxyTable) {
        throw configurationError(`第 ${lineNumber} 行重复定义 [home_proxy]`);
      }
      inHomeProxyTable = true;
      continue;
    }

    if (/^\[.*\]$/.test(line)) {
      throw configurationError(`第 ${lineNumber} 行只能使用 [home_proxy] 表`);
    }

    if (!inHomeProxyTable) {
      throw configurationError(`第 ${lineNumber} 行必须位于 [home_proxy] 表内`);
    }

    const assignment = line.match(/^([A-Za-z0-9_-]+)\s*=\s*(.*)$/);
    if (!assignment) {
      throw configurationError(`第 ${lineNumber} 行不是有效的 key = value`);
    }

    const [, key, rawValue] = assignment;
    if (!REQUIRED_KEYS.includes(key)) {
      throw configurationError(`第 ${lineNumber} 行包含未知字段 ${key}`);
    }
    if (Object.hasOwn(values, key)) {
      throw configurationError(`第 ${lineNumber} 行重复定义字段 ${key}`);
    }
    values[key] = parseValue(rawValue.trim(), lineNumber);
  }

  if (!inHomeProxyTable) {
    throw configurationError("缺少 [home_proxy] 表");
  }
  return values;
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
  outputPath = DEFAULT_OUTPUT_PATH
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
  const config = parseHomeProxyToml(fs.readFileSync(resolvedConfigPath, "utf8"));
  validateHomeProxyConfig(config, extractHomeProxyName(templateSource));

  const banner = [
    "/*",
    ` * 由 ${path.basename(resolvedTemplatePath)} 与 ${path.basename(resolvedConfigPath)} 自动生成。`,
    " * 请编辑 TOML 后运行 `just render-local`，不要直接修改此文件。",
    " */",
    ""
  ].join("\n");
  const output = banner + injectHomeProxyTemplate(templateSource, renderHomeProxyTemplate(config));
  writeFileAtomically(resolvedOutputPath, output);

  return { configPath: resolvedConfigPath, outputPath: resolvedOutputPath };
}

function main() {
  const [configArgument, outputArgument] = process.argv.slice(2);
  const result = syncLocalConfig({
    configPath: configArgument ? path.resolve(process.cwd(), configArgument) : DEFAULT_CONFIG_PATH,
    outputPath: outputArgument ? path.resolve(process.cwd(), outputArgument) : DEFAULT_OUTPUT_PATH
  });
  console.log(`已同步 ${path.basename(result.configPath)} -> ${path.basename(result.outputPath)}`);
}

if (require.main === module) {
  try {
    main();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

module.exports = {
  parseHomeProxyToml,
  validateHomeProxyConfig,
  renderHomeProxyTemplate,
  syncLocalConfig
};

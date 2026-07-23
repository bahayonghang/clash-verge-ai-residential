"use strict";

const assert = require("node:assert/strict");
const childProcess = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const {
  SWITCH_CONFIG_FIELDS,
  parseHomeProxyToml,
  parseLocalToml,
  syncLocalConfig,
  validateHomeProxyConfig
} = require("../scripts/sync-local-config.js");

const root = path.resolve(__dirname, "..");
const templatePath = path.join(root, "clash-verge-ai-residential.js");
const examplePath = path.join(root, "clash-verge-ai-residential.local.toml.example");
const switchDocumentPaths = [
  path.join(root, "docs", "configuration.md"),
  path.join(root, "docs", "local-configuration.md")
];

function withTemporaryDirectory(fn) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "clash-verge-ai-residential-"));
  try {
    fn(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function readBooleanConstant(source, constantName) {
  const pattern = new RegExp(
    `^const[ \\t]+${escapeRegExp(constantName)}[ \\t]*=[ \\t]*(true|false);[ \\t]*$`,
    "gm"
  );
  const matches = [...source.matchAll(pattern)];
  assert.equal(matches.length, 1, `应且仅应找到一个 ${constantName}`);
  return matches[0][1] === "true";
}

function parseRule(rule) {
  if (typeof rule !== "string") return null;
  const parts = rule.split(",");
  if (parts.length < 3) return null;
  return {
    type: parts[0],
    value: parts[1],
    target: parts[parts.length - 1]
  };
}

function ruleMatchesHost(rules, host, target) {
  return rules.some((rule) => {
    const parsed = parseRule(rule);
    if (!parsed || parsed.target !== target) return false;
    if (parsed.type === "DOMAIN") return host === parsed.value;
    if (parsed.type === "DOMAIN-SUFFIX") {
      return host === parsed.value || host.endsWith(`.${parsed.value}`);
    }
    if (parsed.type === "DOMAIN-REGEX") return new RegExp(parsed.value).test(host);
    return false;
  });
}

const validHomeProxyToml = `# This file intentionally contains test-only values.
[home_proxy]
name = "家宽-SOCKS5"
type = "socks5"
server = "home.example.test"
port = 1080
username = "home-user"
password = "home-pass"
udp = true
dialer-proxy = "🚀节点选择"
`;

test("旧版仅含 home_proxy 的 TOML 仍可生成本地脚本且不修改公开模板", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    const originalTemplate = fs.readFileSync(templatePath, "utf8");
    fs.writeFileSync(configPath, validHomeProxyToml, "utf8");

    syncLocalConfig({ templatePath, configPath, outputPath });

    const output = fs.readFileSync(outputPath, "utf8");
    assert.match(output, /由 clash-verge-ai-residential\.js 与 proxy\.local\.toml 自动生成/);
    assert.match(output, /server: "home\.example\.test"/);
    assert.match(output, /port: 1080/);
    assert.match(output, /username: "home-user"/);
    assert.match(output, /"dialer-proxy": "🚀节点选择"/);
    assert.match(output, /const ROUTE_CURSOR_CORE = false;/);
    assert.doesNotMatch(output, /server: "xxx"/);
    assert.equal(fs.readFileSync(templatePath, "utf8"), originalTemplate);

    const parsed = parseLocalToml(validHomeProxyToml);
    assert.deepEqual(parsed.routing, {});
    assert.deepEqual(parsed.runtime, {});
    assert.deepEqual(parseHomeProxyToml(validHomeProxyToml), parsed.homeProxy);
  });
});

test("部分 TOML 开关会注入生成脚本，并恢复窄范围 Cursor 核心路由", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    const originalTemplate = fs.readFileSync(templatePath, "utf8");
    const source = `${validHomeProxyToml}
[routing]
cursor_core = true

[runtime]
enable_tun_strict_route = true
`;
    fs.writeFileSync(configPath, source, "utf8");

    syncLocalConfig({ templatePath, configPath, outputPath });

    const output = fs.readFileSync(outputPath, "utf8");
    assert.match(output, /const ROUTE_CURSOR_CORE = true;/);
    assert.match(output, /const ENABLE_TUN_STRICT_ROUTE = true;/);
    assert.match(output, /const ROUTE_GEMINI_WEB_CORE = true;/);
    assert.equal(fs.readFileSync(templatePath, "utf8"), originalTemplate);

    const probeSource = [
      '"use strict";',
      "const script = require(process.argv[1]);",
      "process.stdout.write(JSON.stringify({",
      "  cursorCore: script.constants.ROUTE_CURSOR_CORE,",
      "  aiGroup: script.constants.AI_GROUP,",
      "  residentialDoh: script.constants.RESIDENTIAL_DOH,",
      "  suffixes: script.constants.CURSOR_SUFFIX_DOMAINS,",
      "  exact: script.constants.CURSOR_EXACT_DOMAINS,",
      "  regexes: script.constants.CURSOR_DOMAIN_REGEXES,",
      "  rules: script.buildInjectedRules(),",
      "  policy: script.buildNameserverPolicy({})",
      "}));"
    ].join("\n");
    const probe = JSON.parse(childProcess.execFileSync(
      process.execPath,
      ["-e", probeSource, outputPath],
      { encoding: "utf8" }
    ));
    const publicScript = require(templatePath);
    assert.equal(publicScript.constants.ROUTE_CURSOR_CORE, false);
    assert.equal(probe.cursorCore, true);

    const publicRules = publicScript.buildInjectedRules();
    const expectedCursorRules = [
      ...probe.suffixes.map(
        (domain) => `DOMAIN-SUFFIX,${domain},${probe.aiGroup}`
      ),
      ...probe.exact.map(
        (domain) => `DOMAIN,${domain},${probe.aiGroup}`
      ),
      ...probe.regexes.map(
        (pattern) => `DOMAIN-REGEX,${pattern},${probe.aiGroup}`
      )
    ];
    assert.equal(probe.rules.length - publicRules.length, expectedCursorRules.length);
    for (const rule of expectedCursorRules) assert.equal(probe.rules.includes(rule), true);

    for (const host of [
      "api2.cursor.sh",
      "agent.api5.cursor.sh",
      "repo42.cursor.sh",
      "us-eu.gcpp.cursor.sh",
      "api.cursor.com"
    ]) {
      assert.equal(
        ruleMatchesHost(probe.rules, host, probe.aiGroup),
        true,
        `本地开关应覆盖 Cursor 核心主机：${host}`
      );
    }
    for (const host of [
      "marketplace.cursorapi.com",
      "downloads.cursor.com",
      "www.cursor.com",
      "docs.cursor.com"
    ]) {
      assert.equal(
        ruleMatchesHost(probe.rules, host, probe.aiGroup),
        false,
        `本地开关不应覆盖 Cursor 非 AI 主机：${host}`
      );
    }

    assert.deepEqual(probe.policy["+.api2.cursor.sh"], probe.residentialDoh);
    assert.equal("marketplace.cursorapi.com" in probe.policy, false);
  });
});

test("同步前会拒绝不完整或不安全的代理配置", () => {
  const incompleteToml = validHomeProxyToml.replace("port = 1080\n", "");
  assert.throws(
    () => validateHomeProxyConfig(parseHomeProxyToml(incompleteToml), "家宽-SOCKS5"),
    /缺少字段 port/
  );

  const invalidPortToml = validHomeProxyToml.replace("port = 1080", "port = 65536");
  assert.throws(
    () => validateHomeProxyConfig(parseHomeProxyToml(invalidPortToml), "家宽-SOCKS5"),
    /port 必须是 1-65535 的整数/
  );
});

test("解析器拒绝未知表、未知键、重复定义和非布尔开关", () => {
  const invalidCases = [
    [
      `${validHomeProxyToml}\n[unknown]\nenabled = true\n`,
      /未知配置表 \[unknown\]/
    ],
    [
      `${validHomeProxyToml}\n[routing]\nunknown = true\n`,
      /未知字段 routing\.unknown/
    ],
    [
      `${validHomeProxyToml}\n[routing]\ncursor_core = "true"\n`,
      /routing\.cursor_core 必须是 true 或 false/
    ],
    [
      `${validHomeProxyToml}\n[routing]\ncursor_core = true\ncursor_core = false\n`,
      /重复定义字段 routing\.cursor_core/
    ],
    [
      `${validHomeProxyToml}\n[routing]\ncursor_core = true\n[routing]\n`,
      /重复定义 \[routing\]/
    ]
  ];

  withTemporaryDirectory((directory) => {
    const originalTemplate = fs.readFileSync(templatePath, "utf8");
    for (const [index, [source, expectedError]] of invalidCases.entries()) {
      const configPath = path.join(directory, `invalid-${index}.toml`);
      const outputPath = path.join(directory, `invalid-${index}.js`);
      fs.writeFileSync(configPath, source, "utf8");
      assert.throws(
        () => syncLocalConfig({ templatePath, configPath, outputPath }),
        expectedError
      );
      assert.equal(fs.existsSync(outputPath), false);
    }
    assert.equal(fs.readFileSync(templatePath, "utf8"), originalTemplate);
  });
});

test("布尔常量锚点缺失或重复时失败且不写入半成品", () => {
  withTemporaryDirectory((directory) => {
    const originalTemplate = fs.readFileSync(templatePath, "utf8");
    const configPath = path.join(directory, "proxy.local.toml");
    fs.writeFileSync(
      configPath,
      `${validHomeProxyToml}\n[routing]\ncursor_core = true\n`,
      "utf8"
    );

    for (const [name, alteredTemplate] of [
      [
        "missing.js",
        originalTemplate.replace("const ROUTE_CURSOR_CORE = false;", "")
      ],
      [
        "duplicate.js",
        `${originalTemplate}\nconst ROUTE_CURSOR_CORE = false;\n`
      ]
    ]) {
      const alteredTemplatePath = path.join(directory, name);
      const outputPath = path.join(directory, `${name}.local.js`);
      fs.writeFileSync(alteredTemplatePath, alteredTemplate, "utf8");
      assert.throws(
        () => syncLocalConfig({
          templatePath: alteredTemplatePath,
          configPath,
          outputPath
        }),
        /必须且只能包含一个布尔常量 ROUTE_CURSOR_CORE/
      );
      assert.equal(fs.existsSync(outputPath), false);
    }

    assert.equal(fs.readFileSync(templatePath, "utf8"), originalTemplate);
  });
});

test("生产映射覆盖全部用户布尔开关，并与示例及文档默认值一致", () => {
  const templateSource = fs.readFileSync(templatePath, "utf8");
  assert.deepEqual(
    [...new Set(SWITCH_CONFIG_FIELDS.map((field) => field.type))],
    ["boolean"]
  );
  assert.deepEqual(
    [...new Set(SWITCH_CONFIG_FIELDS.map((field) => field.table))].sort(),
    ["routing", "runtime"]
  );
  const sectionStart = templateSource.indexOf("// 1. 用户配置");
  const sectionEnd = templateSource.indexOf("// 2. AI 域名清单");
  assert.notEqual(sectionStart, -1);
  assert.notEqual(sectionEnd, -1);
  const userConfigSource = templateSource.slice(sectionStart, sectionEnd);
  const declarationNames = [...userConfigSource.matchAll(
    /^const[ \t]+([A-Z][A-Z0-9_]*)[ \t]*=[ \t]*(?:true|false);[ \t]*$/gm
  )].map((match) => match[1]).sort();
  const mappedNames = SWITCH_CONFIG_FIELDS.map((field) => field.constant).sort();
  assert.deepEqual(mappedNames, declarationNames);

  const example = parseLocalToml(fs.readFileSync(examplePath, "utf8"));
  const documentSources = switchDocumentPaths.map((documentPath) => ({
    path: documentPath,
    source: fs.readFileSync(documentPath, "utf8")
  }));
  for (const field of SWITCH_CONFIG_FIELDS) {
    const defaultValue = readBooleanConstant(templateSource, field.constant);
    assert.equal(
      Object.hasOwn(example[field.table], field.key),
      true,
      `示例 TOML 缺少 ${field.table}.${field.key}`
    );
    assert.equal(
      example[field.table][field.key],
      defaultValue,
      `示例 TOML 默认值与 ${field.constant} 不一致`
    );

    const rowPattern = new RegExp(
      "\\|\\s*`" + escapeRegExp(`${field.table}.${field.key}`) + "`\\s*" +
      "\\|\\s*`" + escapeRegExp(field.constant) + "`\\s*" +
      "\\|\\s*`" + defaultValue + "`\\s*\\|"
    );
    for (const document of documentSources) {
      assert.match(
        document.source,
        rowPattern,
        `${path.relative(root, document.path)} 缺少或写错 ${field.table}.${field.key}`
      );
    }
  }
});

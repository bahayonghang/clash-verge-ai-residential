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

test("旧版仅含 home_proxy 的 TOML 会补全缺失开关并生成本地脚本", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    const originalTemplate = fs.readFileSync(templatePath, "utf8");
    fs.writeFileSync(configPath, validHomeProxyToml, "utf8");

    const result = syncLocalConfig({ templatePath, configPath, outputPath });

    const output = fs.readFileSync(outputPath, "utf8");
    assert.match(output, /由 clash-verge-ai-residential\.js 与 proxy\.local\.toml 自动生成/);
    assert.match(output, /server: "home\.example\.test"/);
    assert.match(output, /const ROUTE_CURSOR_CORE = true;/);
    assert.match(output, /const ROUTE_CURSOR_REPOSITORY_INDEXING = false;/);
    assert.match(output, /const ROUTE_GROK_CORE = true;/);
    assert.match(output, /const ROUTE_OPENAI_AUTH = false;/);
    assert.match(output, /const ROUTE_OPENAI_WEB_ASSETS = false;/);
    assert.doesNotMatch(output, /server: "xxx"/);
    assert.equal(fs.readFileSync(templatePath, "utf8"), originalTemplate);

    // 缺失的开关键按示例默认值补全进本地 TOML，且缺失的整表被追加。
    assert.equal(result.addedKeys.includes("routing.cursor_core"), true);
    assert.equal(result.addedKeys.includes("routing.cursor_repository_indexing"), true);
    assert.equal(result.addedKeys.includes("routing.grok_core"), true);
    assert.equal(result.addedKeys.includes("routing.openai_auth"), true);
    assert.equal(result.addedKeys.includes("routing.openai_web_assets"), true);
    assert.equal(result.addedKeys.includes("routing.grok_web_assets"), true);
    assert.equal(result.addedKeys.includes("routing.vertex_ai_endpoints"), true);
    assert.equal(result.addedKeys.includes("runtime.enable_domain_sniffer"), true);
    assert.equal(
      result.addedDefaults.find((entry) => entry.key === "routing.cursor_repository_indexing").value,
      false
    );
    assert.equal(
      result.addedDefaults.find((entry) => entry.key === "routing.grok_core").value,
      true
    );
    assert.equal(
      result.addedDefaults.find((entry) => entry.key === "runtime.enable_domain_sniffer").value,
      true
    );
    const completedToml = fs.readFileSync(configPath, "utf8");
    assert.match(completedToml, /\[routing\][\s\S]*cursor_core = true/);
    assert.match(completedToml, /\[routing\][\s\S]*cursor_repository_indexing = false/);
    assert.match(completedToml, /\[routing\][\s\S]*grok_core = true/);
    assert.match(completedToml, /\[runtime\][\s\S]*enable_domain_sniffer = true/);
    assert.match(completedToml, /# This file intentionally contains test-only values\./);

    const parsed = parseLocalToml(validHomeProxyToml);
    assert.deepEqual(parsed.routing, {});
    assert.deepEqual(parsed.runtime, {});
    assert.deepEqual(parseHomeProxyToml(validHomeProxyToml), parsed.homeProxy);
  });
});

test("部分 TOML 开关会注入生成脚本，并可关闭默认开启的 Cursor/Grok 核心路由", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    const originalTemplate = fs.readFileSync(templatePath, "utf8");
    const source = `${validHomeProxyToml}
[routing]
cursor_core = false
grok_core = false

[runtime]
enable_tun_strict_route = true
`;
    fs.writeFileSync(configPath, source, "utf8");

    syncLocalConfig({ templatePath, configPath, outputPath });

    const output = fs.readFileSync(outputPath, "utf8");
    assert.match(output, /const ROUTE_CURSOR_CORE = false;/);
    assert.match(output, /const ROUTE_GROK_CORE = false;/);
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
      "  indexing: script.constants.ROUTE_CURSOR_REPOSITORY_INDEXING,",
      "  indexingRegexes: script.constants.CURSOR_REPOSITORY_INDEXING_DOMAIN_REGEXES,",
      "  grokSuffixes: script.constants.GROK_SUFFIX_DOMAINS,",
      "  grokExact: script.constants.GROK_EXACT_DOMAINS,",
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
    assert.equal(publicScript.constants.ROUTE_CURSOR_CORE, true);
    assert.equal(publicScript.constants.ROUTE_CURSOR_REPOSITORY_INDEXING, false);
    assert.equal(probe.cursorCore, false);
    assert.equal(probe.indexing, false);

    const publicRules = publicScript.buildInjectedRules();
    const expectedDisabledRules = [
      ...probe.suffixes.map(
        (domain) => `DOMAIN-SUFFIX,${domain},${probe.aiGroup}`
      ),
      ...probe.exact.map(
        (domain) => `DOMAIN,${domain},${probe.aiGroup}`
      ),
      ...probe.grokSuffixes.map(
        (domain) => `DOMAIN-SUFFIX,${domain},${probe.aiGroup}`
      ),
      ...probe.grokExact.map(
        (domain) => `DOMAIN,${domain},${probe.aiGroup}`
      )
    ];
    assert.equal(publicRules.length - probe.rules.length, expectedDisabledRules.length);
    for (const rule of expectedDisabledRules) {
      assert.equal(publicRules.includes(rule), true);
      assert.equal(probe.rules.includes(rule), false);
    }

    for (const host of [
      "api2.cursor.sh",
      "agent.api5.cursor.sh",
      "authenticate.cursor.sh",
      "repo42.cursor.sh",
      "adminportal42.cursor.sh",
      "us-eu.gcpp.cursor.sh",
      "vm.cursorvm.com",
      "api.cursor.com",
      "cli-chat-proxy.grok.com"
    ]) {
      assert.equal(
        ruleMatchesHost(probe.rules, host, probe.aiGroup),
        false,
        `本地关闭后 Cursor/Grok 核心主机应离开家宽：${host}`
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

    assert.equal("+.api2.cursor.sh" in probe.policy, false);
    assert.equal("+.grok.com" in probe.policy, false);
    assert.equal("marketplace.cursorapi.com" in probe.policy, false);
  });
});

test("本地 cursor_repository_indexing 缺字段补 false，显式 true 恢复 repo 家宽", () => {
  function probeGeneratedScript(outputPath) {
    const probeSource = [
      '"use strict";',
      "const script = require(process.argv[1]);",
      "process.stdout.write(JSON.stringify({",
      "  indexing: script.constants.ROUTE_CURSOR_REPOSITORY_INDEXING,",
      "  cursorCore: script.constants.ROUTE_CURSOR_CORE,",
      "  aiGroup: script.constants.AI_GROUP,",
      "  rules: script.buildInjectedRules()",
      "}));"
    ].join("\n");
    return JSON.parse(childProcess.execFileSync(
      process.execPath,
      ["-e", probeSource, outputPath],
      { encoding: "utf8" }
    ));
  }

  withTemporaryDirectory((directory) => {
    const originalTemplate = fs.readFileSync(templatePath, "utf8");
    const publicScript = require(templatePath);
    assert.equal(publicScript.constants.ROUTE_CURSOR_REPOSITORY_INDEXING, false);
    assert.equal(
      ruleMatchesHost(
        publicScript.buildInjectedRules(),
        "repo42.cursor.sh",
        publicScript.constants.AI_GROUP
      ),
      false
    );

    const missingPath = path.join(directory, "missing.toml");
    const missingOutput = path.join(directory, "missing.local.js");
    fs.writeFileSync(missingPath, validHomeProxyToml, "utf8");
    const missingResult = syncLocalConfig({
      templatePath,
      configPath: missingPath,
      outputPath: missingOutput
    });
    assert.equal(
      missingResult.addedKeys.includes("routing.cursor_repository_indexing"),
      true
    );
    assert.equal(
      missingResult.addedDefaults.find(
        (entry) => entry.key === "routing.cursor_repository_indexing"
      ).value,
      false
    );
    assert.match(
      fs.readFileSync(missingPath, "utf8"),
      /cursor_repository_indexing = false/
    );
    const missingProbe = probeGeneratedScript(missingOutput);
    assert.equal(missingProbe.indexing, false);
    assert.equal(missingProbe.cursorCore, true);
    assert.equal(
      ruleMatchesHost(missingProbe.rules, "repo42.cursor.sh", missingProbe.aiGroup),
      false
    );
    assert.equal(
      ruleMatchesHost(missingProbe.rules, "repo99.cursor.sh", missingProbe.aiGroup),
      false
    );
    assert.equal(
      ruleMatchesHost(missingProbe.rules, "api2.cursor.sh", missingProbe.aiGroup),
      true
    );

    for (const [label, enabled] of [["on", true], ["off", false]]) {
      const configPath = path.join(directory, `${label}.toml`);
      const outputPath = path.join(directory, `${label}.local.js`);
      fs.writeFileSync(
        configPath,
        `${validHomeProxyToml}\n[routing]\ncursor_repository_indexing = ${enabled}\n`,
        "utf8"
      );
      syncLocalConfig({ templatePath, configPath, outputPath });
      const output = fs.readFileSync(outputPath, "utf8");
      assert.match(
        output,
        new RegExp(`const ROUTE_CURSOR_REPOSITORY_INDEXING = ${enabled};`)
      );
      const probe = probeGeneratedScript(outputPath);
      assert.equal(probe.indexing, enabled);
      assert.equal(probe.cursorCore, true);
      assert.equal(
        ruleMatchesHost(probe.rules, "repo42.cursor.sh", probe.aiGroup),
        enabled,
        `显式 ${enabled} 时应决定 repo42 是否走家宽`
      );
      assert.equal(
        ruleMatchesHost(probe.rules, "repo99.cursor.sh", probe.aiGroup),
        enabled,
        `显式 ${enabled} 时应决定 repo99 是否走家宽`
      );
      assert.equal(
        ruleMatchesHost(probe.rules, "api2.cursor.sh", probe.aiGroup),
        true,
        "显式开关不得把 api2 移出 cursor_core"
      );
      assert.equal(
        ruleMatchesHost(probe.rules, "adminportal42.cursor.sh", probe.aiGroup),
        true
      );
      assert.equal(
        ruleMatchesHost(probe.rules, "marketplace.cursorapi.com", probe.aiGroup),
        false
      );
    }

    assert.equal(fs.readFileSync(templatePath, "utf8"), originalTemplate);
    assert.match(originalTemplate, /const ROUTE_CURSOR_REPOSITORY_INDEXING = false;/);
  });
});

test("本地 openai_core = false 让 GPT 域名不再走家宽，Claude 等核心域名不受影响", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    const originalTemplate = fs.readFileSync(templatePath, "utf8");
    const source = `${validHomeProxyToml}
[routing]
openai_core = false
`;
    fs.writeFileSync(configPath, source, "utf8");

    syncLocalConfig({ templatePath, configPath, outputPath });

    const output = fs.readFileSync(outputPath, "utf8");
    assert.match(output, /const ROUTE_OPENAI_CORE = false;/);
    assert.equal(fs.readFileSync(templatePath, "utf8"), originalTemplate);

    const probeSource = [
      '"use strict";',
      "const script = require(process.argv[1]);",
      "process.stdout.write(JSON.stringify({",
      "  openaiCore: script.constants.ROUTE_OPENAI_CORE,",
      "  aiGroup: script.constants.AI_GROUP,",
      "  residentialDoh: script.constants.RESIDENTIAL_DOH,",
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
    assert.equal(publicScript.constants.ROUTE_OPENAI_CORE, true);
    assert.equal(probe.openaiCore, false);

    for (const host of ["chatgpt.com", "api.openai.com", "oaiusercontent.com"]) {
      assert.equal(
        ruleMatchesHost(probe.rules, host, probe.aiGroup),
        false,
        `本地开关应让 GPT 域名离开家宽：${host}`
      );
      assert.equal(
        `+.${host}` in probe.policy,
        false,
        `DNS policy 不应包含 GPT suffix 键：+.${host}`
      );
    }
    for (const host of publicScript.constants.OPENAI_CORE_EXACT_DOMAINS) {
      assert.equal(
        ruleMatchesHost(probe.rules, host, probe.aiGroup),
        false,
        `本地开关应让 GPT exact 主机离开家宽：${host}`
      );
      assert.equal(host in probe.policy, false, `DNS policy 不应包含 GPT exact 裸键：${host}`);
    }
    for (const host of [
      "claude.ai",
      "api.anthropic.com",
      "mcp-proxy.anthropic.com",
      "antigravity.google"
    ]) {
      assert.equal(
        ruleMatchesHost(probe.rules, host, probe.aiGroup),
        true,
        `核心域名仍应走家宽：${host}`
      );
    }
    assert.deepEqual(probe.policy["api.anthropic.com"], probe.residentialDoh);
  });
});

test("本地 openai_auth = true 只启用第一方认证，网页资源与共享依赖保持关闭", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    const originalTemplate = fs.readFileSync(templatePath, "utf8");
    const source = `${validHomeProxyToml}
[routing]
openai_auth = true
`;
    fs.writeFileSync(configPath, source, "utf8");

    const result = syncLocalConfig({ templatePath, configPath, outputPath });
    assert.equal(result.addedKeys.includes("routing.openai_auth"), false);
    assert.equal(result.addedKeys.includes("routing.openai_web_assets"), true);

    const output = fs.readFileSync(outputPath, "utf8");
    assert.match(output, /const ROUTE_OPENAI_AUTH = true;/);
    assert.match(output, /const ROUTE_OPENAI_WEB_ASSETS = false;/);
    assert.equal(fs.readFileSync(templatePath, "utf8"), originalTemplate);

    const probeSource = [
      '"use strict";',
      "const script = require(process.argv[1]);",
      "process.stdout.write(JSON.stringify({",
      "  openaiAuth: script.constants.ROUTE_OPENAI_AUTH,",
      "  openaiWebAssets: script.constants.ROUTE_OPENAI_WEB_ASSETS,",
      "  aiGroup: script.constants.AI_GROUP,",
      "  residentialDoh: script.constants.RESIDENTIAL_DOH,",
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

    assert.equal(publicScript.constants.ROUTE_OPENAI_AUTH, false);
    assert.equal(publicScript.constants.ROUTE_OPENAI_WEB_ASSETS, false);
    assert.equal(probe.openaiAuth, true);
    assert.equal(probe.openaiWebAssets, false);
    for (const host of [
      "auth.openai.com",
      "setup.auth.openai.com",
      "tenant.auth.openai.com",
      "auth0.openai.com"
    ]) {
      assert.equal(
        ruleMatchesHost(probe.rules, host, probe.aiGroup),
        true,
        `认证主机应走家宽：${host}`
      );
    }
    for (const host of [
      "oaistatic.com",
      "cdn.oaistatic.com",
      "www.openai.com",
      "child.auth0.openai.com",
      "oaistatsig.com",
      "intercom.io",
      "challenges.cloudflare.com"
    ]) {
      assert.equal(
        ruleMatchesHost(probe.rules, host, probe.aiGroup),
        false,
        `认证开关不应扩大到：${host}`
      );
    }
    assert.deepEqual(probe.policy["+.auth.openai.com"], probe.residentialDoh);
    assert.deepEqual(probe.policy["auth0.openai.com"], probe.residentialDoh);
    assert.equal("+.auth0.openai.com" in probe.policy, false);
    assert.equal("+.oaistatic.com" in probe.policy, false);
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
    ],
    [
      `${validHomeProxyToml}\n[routing]\ncursor_repository_indexing = "false"\n`,
      /routing\.cursor_repository_indexing 必须是 true 或 false/
    ],
    [
      `${validHomeProxyToml}\n[routing]\ncursor_repository_indexing = true\ncursor_repository_indexing = false\n`,
      /重复定义字段 routing\.cursor_repository_indexing/
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

    for (const [name, alteredTemplate, expectedError] of [
      [
        "missing.js",
        originalTemplate.replace("const ROUTE_CURSOR_CORE = true;", ""),
        /必须且只能包含一个布尔常量 ROUTE_CURSOR_CORE/
      ],
      [
        "duplicate.js",
        `${originalTemplate}\nconst ROUTE_CURSOR_CORE = true;\n`,
        /必须且只能包含一个布尔常量 ROUTE_CURSOR_CORE/
      ],
      [
        "indexing-missing.js",
        originalTemplate.replace("const ROUTE_CURSOR_REPOSITORY_INDEXING = false;", ""),
        /必须且只能包含一个布尔常量 ROUTE_CURSOR_REPOSITORY_INDEXING/
      ],
      [
        "indexing-duplicate.js",
        `${originalTemplate}\nconst ROUTE_CURSOR_REPOSITORY_INDEXING = false;\n`,
        /必须且只能包含一个布尔常量 ROUTE_CURSOR_REPOSITORY_INDEXING/
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
        expectedError
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

// ---------------------------------------------------------------------------
// 本地 TOML 缺失键自动补全
// ---------------------------------------------------------------------------

const partialSwitchToml = `${validHomeProxyToml}
# 自定义注释：只保留部分开关
[routing]
openai_core = false
cursor_core = true
`;

test("缺失开关键按示例默认值补全，用户已有键值与注释逐字保留", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    fs.writeFileSync(configPath, partialSwitchToml, "utf8");

    const result = syncLocalConfig({ templatePath, configPath, outputPath });

    assert.equal(result.addedKeys.includes("routing.grok_core"), true);
    assert.equal(result.addedKeys.includes("routing.grok_web_assets"), true);
    assert.equal(result.addedKeys.includes("routing.openai_auth"), true);
    assert.equal(result.addedKeys.includes("routing.openai_web_assets"), true);
    assert.equal(result.addedKeys.includes("routing.vertex_ai_endpoints"), true);
    assert.equal(result.addedKeys.includes("routing.public_encrypted_dns"), true);
    assert.equal(result.addedKeys.includes("runtime.enable_domain_sniffer"), true);
    assert.equal(result.addedKeys.includes("routing.cursor_core"), false);
    assert.equal(result.addedKeys.includes("routing.cursor_repository_indexing"), true);
    assert.equal(
      result.addedDefaults.find((entry) => entry.key === "routing.public_encrypted_dns").value,
      false
    );

    const completed = fs.readFileSync(configPath, "utf8");
    assert.match(completed, /# 自定义注释：只保留部分开关/);
    assert.match(completed, /# This file intentionally contains test-only values\./);

    const reparsed = parseLocalToml(completed);
    assert.equal(reparsed.routing.openai_core, false, "用户已有值不应被覆盖");
    assert.equal(reparsed.routing.openai_auth, false);
    assert.equal(reparsed.routing.openai_web_assets, false);
    assert.equal(reparsed.routing.cursor_core, true, "用户已有值不应被覆盖");
    assert.equal(reparsed.routing.cursor_repository_indexing, false);
    assert.equal(reparsed.routing.grok_core, true);
    assert.equal(reparsed.routing.public_encrypted_dns, false);
    assert.equal(reparsed.runtime.enable_domain_sniffer, true);
    assert.equal(reparsed.runtime.warn_on_reachable_udp_disabled, true);

    // 缺失键插在所属表区块内，而不是落在 home_proxy 或其他表之下。
    const routingBlock = completed.slice(
      completed.indexOf("[routing]"),
      completed.indexOf("[runtime]")
    );
    assert.match(routingBlock, /^openai_core = false$/m);
    assert.match(routingBlock, /^openai_auth = false$/m);
    assert.match(routingBlock, /^openai_web_assets = false$/m);
    assert.match(routingBlock, /^grok_core = true$/m);
    assert.match(completed, /\[runtime\]\r?\nallow_final_rule_upstream_fallback = true/);

    const output = fs.readFileSync(outputPath, "utf8");
    assert.match(output, /const ROUTE_GROK_CORE = true;/);
    assert.match(output, /const ROUTE_OPENAI_CORE = false;/);
    assert.match(output, /const ROUTE_OPENAI_AUTH = false;/);
    assert.match(output, /const ROUTE_OPENAI_WEB_ASSETS = false;/);
    assert.match(output, /const ENABLE_TUN_STRICT_ROUTE = false;/);
  });
});

test("补全幂等：无缺失时不再改写本地 TOML", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    fs.writeFileSync(configPath, partialSwitchToml, "utf8");

    const first = syncLocalConfig({ templatePath, configPath, outputPath });
    assert.equal(first.addedKeys.length > 0, true);

    const completedSource = fs.readFileSync(configPath, "utf8");
    const second = syncLocalConfig({ templatePath, configPath, outputPath });
    assert.deepEqual(second.addedKeys, []);
    assert.equal(fs.readFileSync(configPath, "utf8"), completedSource);
  });
});

test("与示例一致的完整 TOML 不被补全改写", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    const exampleSource = fs.readFileSync(examplePath, "utf8");
    const completeToml = exampleSource.replace(
      /server = "xxx"\nport = 443\nusername = "xxx"\npassword = "xxx"/,
      'server = "home.example.test"\nport = 1080\nusername = "home-user"\npassword = "home-pass"'
    );
    fs.writeFileSync(configPath, completeToml, "utf8");

    const result = syncLocalConfig({ templatePath, configPath, outputPath });
    assert.deepEqual(result.addedKeys, []);
    assert.equal(fs.readFileSync(configPath, "utf8"), completeToml);
  });
});

test("CRLF 本地 TOML 补全后保持 CRLF 行尾", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    fs.writeFileSync(configPath, partialSwitchToml.replace(/\n/g, "\r\n"), "utf8");

    syncLocalConfig({ templatePath, configPath, outputPath });

    const completed = fs.readFileSync(configPath, "utf8");
    assert.equal(/(?<!\r)\n/.test(completed), false, "不应出现孤立 LF 行尾");
    assert.match(completed, /\[runtime\]\r\nallow_final_rule_upstream_fallback = true/);
  });
});

test("home_proxy 凭据缺键不自动补全，仍要求用户手填", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    fs.writeFileSync(
      configPath,
      validHomeProxyToml.replace("password = \"home-pass\"\n", ""),
      "utf8"
    );

    assert.throws(
      () => syncLocalConfig({ templatePath, configPath, outputPath }),
      /缺少字段 password/
    );
  });
});

test("示例 TOML 缺少声明的开关键时同步失败并提示补齐", () => {
  withTemporaryDirectory((directory) => {
    const configPath = path.join(directory, "proxy.local.toml");
    const outputPath = path.join(directory, "proxy.local.js");
    const brokenExamplePath = path.join(directory, "broken.example.toml");
    const exampleSource = fs.readFileSync(examplePath, "utf8");
    fs.writeFileSync(
      brokenExamplePath,
      exampleSource.replace(/grok_core = true\r?\n/, ""),
      "utf8"
    );
    fs.writeFileSync(configPath, validHomeProxyToml, "utf8");

    assert.throws(
      () => syncLocalConfig({
        templatePath,
        configPath,
        outputPath,
        examplePath: brokenExamplePath
      }),
      /示例 TOML 缺少 routing\.grok_core/
    );
  });
});

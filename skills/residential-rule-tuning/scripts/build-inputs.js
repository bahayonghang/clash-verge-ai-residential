"use strict";

const fs = require("node:fs");
const path = require("node:path");

const REPO_ROOT = path.resolve(__dirname, "..", "..", "..");

const SUPPORTED_SWITCH_BUILDERS = Object.freeze({
  openai_core: (constants) => [
    ...constants.OPENAI_CORE_SUFFIX_DOMAINS,
    ...constants.OPENAI_CORE_EXACT_DOMAINS
  ],
  openai_auth: (constants) => [
    ...constants.OPENAI_AUTH_SUFFIX_DOMAINS,
    ...constants.OPENAI_AUTH_EXACT_DOMAINS
  ],
  openai_web_assets: (constants) => [...constants.OPENAI_WEB_ASSET_SUFFIX_DOMAINS],
  gemini_web_core: (constants) => [
    ...constants.GEMINI_WEB_SUFFIX_DOMAINS,
    ...constants.GEMINI_WEB_EXACT_DOMAINS
  ],
  vertex_ai_endpoints: (constants) => [
    ...constants.VERTEX_AI_EXACT_DOMAINS,
    ...constants.VERTEX_AI_DOMAIN_REGEXES
  ],
  cursor_core: (constants) => [
    ...constants.CURSOR_SUFFIX_DOMAINS,
    ...constants.CURSOR_EXACT_DOMAINS
  ],
  cursor_repository_indexing: (constants) => [
    ...constants.CURSOR_REPOSITORY_INDEXING_DOMAIN_REGEXES
  ],
  grok_core: (constants) => [
    ...constants.GROK_SUFFIX_DOMAINS,
    ...constants.GROK_STRICT_EXACT_DOMAINS,
    ...constants.GROK_EXACT_DOMAINS
  ],
  grok_web_assets: (constants) => [...constants.GROK_EXACT_DOMAINS]
});

function uniqueStrings(values) {
  return [...new Set(values.filter((item) => typeof item === "string" && item.length > 0))];
}

function routingSwitchKeys(repoRoot) {
  const { SWITCH_CONFIG_FIELDS } = require(path.join(repoRoot, "scripts", "sync-local-config.js"));
  return SWITCH_CONFIG_FIELDS
    .filter((field) => field.table === "routing")
    .map((field) => field.key);
}

function loadExtension(repoRoot) {
  return require(path.join(repoRoot, "clash-verge-ai-residential.js"));
}

function buildInputs(repoRoot) {
  const root = repoRoot || REPO_ROOT;
  const extension = loadExtension(root);
  const routingKeys = routingSwitchKeys(root);
  const supported = {};
  for (const [key, builder] of Object.entries(SUPPORTED_SWITCH_BUILDERS)) {
    if (!routingKeys.includes(key)) {
      throw new Error(`受支持开关 ${key} 不在 routing 表中`);
    }
    supported[key] = uniqueStrings(builder(extension.constants));
  }
  const unsupported = routingKeys.filter((key) => !Object.prototype.hasOwnProperty.call(supported, key));
  if (Object.keys(supported).length + unsupported.length !== routingKeys.length) {
    throw new Error(
      `开关完整性检查失败：supported=${Object.keys(supported).length} unsupported=${unsupported.length} routing=${routingKeys.length}`
    );
  }
  return {
    rules: {
      schemaVersion: 1,
      group: extension.constants.AI_GROUP,
      rules: extension.buildInjectedRules()
    },
    switches: {
      schemaVersion: 1,
      supported,
      unsupported
    },
    routingCount: routingKeys.length
  };
}

function main(argv, repoRoot) {
  const outDir = argv[2] ? path.resolve(argv[2]) : process.cwd();
  fs.mkdirSync(outDir, { recursive: true });
  const built = buildInputs(repoRoot || REPO_ROOT);
  fs.writeFileSync(
    path.join(outDir, "rules.json"),
    `${JSON.stringify(built.rules, null, 2)}\n`
  );
  fs.writeFileSync(
    path.join(outDir, "switches.json"),
    `${JSON.stringify(built.switches, null, 2)}\n`
  );
}

if (require.main === module) {
  try {
    main(process.argv);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

module.exports = {
  SUPPORTED_SWITCH_BUILDERS,
  buildInputs,
  main,
  routingSwitchKeys
};

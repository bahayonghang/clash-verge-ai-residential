"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const {
  checkTemplateSafety
} = require("../scripts/check-template-safety.js");

function withTemporaryDirectory(fn) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "template-safety-"));
  try {
    fn(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

function writeSafeTemplate(directory, server = "xxx") {
  fs.writeFileSync(
    path.join(directory, "clash-verge-ai-residential.js"),
    `"use strict";\n\nconst HOME_PROXY_TEMPLATE = {\n  server: "${server}",\n  username: "xxx",\n  password: "xxx"\n};\n`,
    "utf8"
  );
}

function forbiddenBearerToken() {
  return ["Bearer", "a".repeat(24)].join(" ");
}

function forbiddenSecrets() {
  return [
    ["-----BEGIN", "PRIVATE KEY-----"].join(" "),
    ["ghp", "_", "a".repeat(20)].join(""),
    ["AKIA", "A".repeat(16)].join(""),
    forbiddenBearerToken()
  ];
}

test("安全模板和可提交文本通过扫描", () => {
  withTemporaryDirectory((directory) => {
    writeSafeTemplate(directory);
    for (const extension of [".json", ".jsonl", ".md", ".py", ".toml", ".yml", ".yaml"]) {
      fs.writeFileSync(path.join(directory, `safe${extension}`), "safe fixture\n", "utf8");
    }

    assert.deepEqual(checkTemplateSafety(directory), []);
  });
});

test("公共模板中的真实代理字段会被拒绝", () => {
  withTemporaryDirectory((directory) => {
    writeSafeTemplate(directory, "proxy.example.test");

    assert.deepEqual(checkTemplateSafety(directory), [
      "HOME_PROXY_TEMPLATE.server 不能在公共模板中保存真实值"
    ]);
  });
});

test("所有可提交文本格式都会扫描常见凭据", () => {
  withTemporaryDirectory((directory) => {
    writeSafeTemplate(directory);
    const extensions = [".js", ".json", ".jsonl", ".md", ".py", ".toml", ".yml", ".yaml"];
    for (const extension of extensions) {
      fs.writeFileSync(
        path.join(directory, `secret${extension}`),
        forbiddenBearerToken(),
        "utf8"
      );
    }

    const failures = checkTemplateSafety(directory);
    assert.equal(failures.length, extensions.length);
    for (const extension of extensions) {
      assert.equal(
        failures.some((failure) => failure.startsWith(`secret${extension}:`)),
        true,
        `应扫描 ${extension}`
      );
    }
  });
});

test("每类受保护凭据模式都会被拒绝", () => {
  withTemporaryDirectory((directory) => {
    writeSafeTemplate(directory);
    const secrets = forbiddenSecrets();
    for (const [index, secret] of secrets.entries()) {
      fs.writeFileSync(path.join(directory, `secret-${index}.md`), secret, "utf8");
    }

    const failures = checkTemplateSafety(directory);
    assert.equal(failures.length, secrets.length);
    for (const expectedName of [
      "private key",
      "GitHub token",
      "AWS access key",
      "generic bearer token"
    ]) {
      assert.equal(
        failures.some((failure) => failure.endsWith(`疑似 ${expectedName}`)),
        true,
        `应检测 ${expectedName}`
      );
    }
  });
});

test("扫描忽略本地凭据、生成脚本和依赖目录", () => {
  withTemporaryDirectory((directory) => {
    writeSafeTemplate(directory);
    fs.mkdirSync(path.join(directory, ".git"));
    fs.mkdirSync(path.join(directory, "node_modules"));
    fs.mkdirSync(path.join(directory, "generated"));
    const ignoredPaths = [
      path.join(directory, "clash-verge-ai-residential.local.toml"),
      path.join(directory, "generated", "profile.local.js"),
      path.join(directory, ".git", "config.toml"),
      path.join(directory, "node_modules", "package.js")
    ];
    for (const ignoredPath of ignoredPaths) {
      fs.writeFileSync(ignoredPath, forbiddenBearerToken(), "utf8");
    }

    assert.deepEqual(checkTemplateSafety(directory), []);
  });
});

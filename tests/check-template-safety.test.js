"use strict";

const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const {
  checkTemplateSafety,
  runCli
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
  writeExampleToml(directory);
}

function writeExampleToml(directory, fields = {}) {
  const server = Object.prototype.hasOwnProperty.call(fields, "server")
    ? fields.server
    : "xxx";
  const username = Object.prototype.hasOwnProperty.call(fields, "username")
    ? fields.username
    : "xxx";
  const password = Object.prototype.hasOwnProperty.call(fields, "password")
    ? fields.password
    : "xxx";
  fs.writeFileSync(
    path.join(directory, "clash-verge-ai-residential.local.toml.example"),
    `[home_proxy]\nserver = "${server}"\nusername = "${username}"\npassword = "${password}"\n`,
    "utf8"
  );
}

function withMutedConsole(fn) {
  const originalError = console.error;
  const originalLog = console.log;
  console.error = () => {};
  console.log = () => {};
  try {
    return fn();
  } finally {
    console.error = originalError;
    console.log = originalLog;
  }
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
    for (const extension of [".json", ".jsonl", ".md", ".py", ".toml", ".yml", ".yaml", ".rs", ".ts"]) {
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
    const extensions = [".js", ".json", ".jsonl", ".md", ".py", ".toml", ".yml", ".yaml", ".rs", ".ts"];
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
    fs.mkdirSync(path.join(directory, "ref", "neko-master"), { recursive: true });
    const ignoredPaths = [
      path.join(directory, "clash-verge-ai-residential.local.toml"),
      path.join(directory, "generated", "profile.local.js"),
      path.join(directory, ".git", "config.toml"),
      path.join(directory, "node_modules", "package.js"),
      path.join(directory, "ref", "neko-master", "fixture.md")
    ];
    for (const ignoredPath of ignoredPaths) {
      fs.writeFileSync(ignoredPath, forbiddenBearerToken(), "utf8");
    }

    assert.deepEqual(checkTemplateSafety(directory), []);
  });
});

test("example TOML 把 password 改成非占位后扫描失败", () => {
  withTemporaryDirectory((directory) => {
    writeSafeTemplate(directory);
    const examplePath = path.join(
      __dirname,
      "..",
      "clash-verge-ai-residential.local.toml.example"
    );
    const original = fs.readFileSync(examplePath, "utf8");
    const mutated = original.replace(
      /^(password = )"xxx"(\r?)$/m,
      "$1\"not-a-placeholder\"$2"
    );
    assert.notEqual(mutated, original);
    fs.writeFileSync(
      path.join(directory, "clash-verge-ai-residential.local.toml.example"),
      mutated,
      "utf8"
    );

    const failures = checkTemplateSafety(directory);
    assert.equal(failures.length, 1);
    assert.equal(
      failures[0],
      "clash-verge-ai-residential.local.toml.example [home_proxy].password 不能在公共示例中保存真实值"
    );
    assert.equal(withMutedConsole(() => runCli(directory)), 1);
  });
});

test("example TOML 的 server 与 username 非占位也会失败", () => {
  for (const field of ["server", "username"]) {
    withTemporaryDirectory((directory) => {
      writeSafeTemplate(directory);
      writeExampleToml(directory, { [field]: "not-a-placeholder" });
      const failures = checkTemplateSafety(directory);
      assert.equal(failures.length, 1, field);
      assert.equal(
        failures[0],
        `clash-verge-ai-residential.local.toml.example [home_proxy].${field} 不能在公共示例中保存真实值`
      );
    });
  }
});

test("example TOML 允许空字符串占位", () => {
  withTemporaryDirectory((directory) => {
    writeSafeTemplate(directory);
    writeExampleToml(directory, { server: "", username: "", password: "" });

    assert.deepEqual(checkTemplateSafety(directory), []);
    assert.equal(withMutedConsole(() => runCli(directory)), 0);
  });
});

test("缺少 example TOML 会失败", () => {
  withTemporaryDirectory((directory) => {
    fs.writeFileSync(
      path.join(directory, "clash-verge-ai-residential.js"),
      `"use strict";\n\nconst HOME_PROXY_TEMPLATE = {\n  server: "xxx",\n  username: "xxx",\n  password: "xxx"\n};\n`,
      "utf8"
    );

    assert.deepEqual(checkTemplateSafety(directory), [
      "clash-verge-ai-residential.local.toml.example 缺失"
    ]);
  });
});

test("SECURITY.md Scope 包含 ResiWatch", () => {
  const text = fs.readFileSync(path.join(__dirname, "..", "SECURITY.md"), "utf8");
  assert.match(text, /ResiWatch|residential-monitor/);
});

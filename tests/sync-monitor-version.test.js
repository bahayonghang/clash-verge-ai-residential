"use strict";

const assert = require("node:assert/strict");
const childProcess = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const {
  syncMonitorVersion
} = require("../scripts/sync-monitor-version.js");

function withTemporaryDirectory(fn) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "sync-monitor-version-"));
  try {
    fn(directory);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
}

function writeTree(root, version, options = {}) {
  const eol = options.eol || "\n";
  const monitor = path.join(root, "residential-monitor");
  const tauri = path.join(monitor, "src-tauri");
  fs.mkdirSync(tauri, { recursive: true });
  fs.writeFileSync(
    path.join(monitor, "package.json"),
    `{\n  "name": "residential-monitor",\n  "version": "${version}"\n}\n`,
    "utf8"
  );
  const lock = [
    "{",
    '  "name": "residential-monitor",',
    `  "version": "${options.lockVersion || version}",`,
    '  "packages": {',
    '    "": {',
    '      "name": "residential-monitor",',
    `      "version": "${options.lockPackageVersion || options.lockVersion || version}"`,
    "    },",
    '    "node_modules/yocto-queue": {',
    '      "version": "0.1.0"',
    "    }",
    "  }",
    "}",
    ""
  ].join(eol);
  fs.writeFileSync(path.join(monitor, "package-lock.json"), lock, "utf8");
  fs.writeFileSync(
    path.join(tauri, "tauri.conf.json"),
    `{${eol}  "productName": "ResiWatch",${eol}  "version": "${options.tauriVersion || version}"${eol}}${eol}`,
    "utf8"
  );
  const cargoToml = [
    "[package]",
    'name = "residential-monitor"',
    `version = "${options.cargoVersion || version}"`,
    "",
    "[[bin]]",
    'name = "residential-monitor"',
    'path = "src/main.rs"',
    "",
    "[dependencies]",
    'hyper-util = { version = "0.1", features = [] }',
    ""
  ].join(eol);
  fs.writeFileSync(path.join(tauri, "Cargo.toml"), cargoToml, "utf8");
  const cargoLock = [
    "[[package]]",
    'name = "residential-monitor"',
    `version = "${options.cargoLockVersion || options.cargoVersion || version}"`,
    "",
    "[[package]]",
    'name = "vswhom"',
    'version = "0.1.0"',
    ""
  ].join(eol);
  fs.writeFileSync(path.join(tauri, "Cargo.lock"), cargoLock, "utf8");
}

test("写入模式把 package.json 版本同步到 Tauri、Cargo 与 lockfile", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.1.0");
    fs.writeFileSync(
      path.join(directory, "residential-monitor", "package.json"),
      '{\n  "name": "residential-monitor",\n  "version": "0.2.0"\n}\n',
      "utf8"
    );

    const result = syncMonitorVersion(directory);
    assert.equal(result.version, "0.2.0");
    assert.equal(result.changed.length, 4);

    const tauri = JSON.parse(
      fs.readFileSync(
        path.join(directory, "residential-monitor", "src-tauri", "tauri.conf.json"),
        "utf8"
      )
    );
    assert.equal(tauri.version, "0.2.0");

    const cargoToml = fs.readFileSync(
      path.join(directory, "residential-monitor", "src-tauri", "Cargo.toml"),
      "utf8"
    );
    assert.match(cargoToml, /\[package\]\r?\nname = "residential-monitor"\r?\nversion = "0.2.0"/);
    assert.match(cargoToml, /\[\[bin\]\]\r?\nname = "residential-monitor"\r?\npath = /);
    assert.match(cargoToml, /hyper-util = \{ version = "0.1"/);

    const cargoLock = fs.readFileSync(
      path.join(directory, "residential-monitor", "src-tauri", "Cargo.lock"),
      "utf8"
    );
    assert.match(cargoLock, /name = "residential-monitor"\r?\nversion = "0.2.0"/);
    assert.match(cargoLock, /name = "vswhom"\r?\nversion = "0.1.0"/);

    const lock = JSON.parse(
      fs.readFileSync(
        path.join(directory, "residential-monitor", "package-lock.json"),
        "utf8"
      )
    );
    assert.equal(lock.version, "0.2.0");
    assert.equal(lock.packages[""].version, "0.2.0");
    assert.equal(lock.packages["node_modules/yocto-queue"].version, "0.1.0");
  });
});

test("检查模式发现漂移时不写文件", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.1.0");
    fs.writeFileSync(
      path.join(directory, "residential-monitor", "package.json"),
      '{\n  "name": "residential-monitor",\n  "version": "0.2.0"\n}\n',
      "utf8"
    );
    const cargoPath = path.join(
      directory,
      "residential-monitor",
      "src-tauri",
      "Cargo.toml"
    );
    const before = fs.readFileSync(cargoPath, "utf8");

    const result = syncMonitorVersion(directory, { check: true });
    assert.equal(result.check, true);
    assert.equal(result.version, "0.2.0");
    assert.ok(result.changed.some((item) => item.label.endsWith("Cargo.toml")));
    assert.equal(fs.readFileSync(cargoPath, "utf8"), before);
  });
});

test("已对齐时写入模式不改内容", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.2.0");
    const cargoPath = path.join(
      directory,
      "residential-monitor",
      "src-tauri",
      "Cargo.toml"
    );
    const before = fs.readFileSync(cargoPath, "utf8");
    const result = syncMonitorVersion(directory);
    assert.deepEqual(result.changed, []);
    assert.equal(fs.readFileSync(cargoPath, "utf8"), before);
  });
});

test("保留 Cargo.toml 的 CRLF", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.1.0", { eol: "\r\n" });
    fs.writeFileSync(
      path.join(directory, "residential-monitor", "package.json"),
      '{\n  "name": "residential-monitor",\n  "version": "1.2.3"\n}\n',
      "utf8"
    );
    syncMonitorVersion(directory);
    const cargoToml = fs.readFileSync(
      path.join(directory, "residential-monitor", "src-tauri", "Cargo.toml"),
      "utf8"
    );
    assert.equal(cargoToml.includes("\r\n"), true);
    assert.match(cargoToml, /version = "1.2.3"/);
  });
});

test("非法 SemVer 失败", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.1.0");
    fs.writeFileSync(
      path.join(directory, "residential-monitor", "package.json"),
      '{\n  "name": "residential-monitor",\n  "version": "latest"\n}\n',
      "utf8"
    );
    assert.throws(
      () => syncMonitorVersion(directory),
      /不是有效 SemVer/
    );
  });
});

const MONITOR_NATIVE_COMMANDS = [
  "npm --prefix residential-monitor ci",
  "npm --prefix residential-monitor run check",
  "cargo fmt --manifest-path residential-monitor/src-tauri/Cargo.toml --check",
  "cargo clippy --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace --all-targets -- -D warnings",
  "cargo test --manifest-path residential-monitor/src-tauri/Cargo.toml --workspace",
  "npm run check:secrets"
];

const CI_WORKFLOW_PATH = path.join(__dirname, "..", ".github", "workflows", "ci.yml");

function readCiWorkflowYaml() {
  return fs.readFileSync(CI_WORKFLOW_PATH, "utf8").replace(/\r\n/g, "\n");
}

function unquoteYamlScalar(value) {
  const trimmed = value.trim();
  if (
    (trimmed.startsWith("\"") && trimmed.endsWith("\"")) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed.slice(1, -1);
  }
  return trimmed;
}

function indentOf(line) {
  return /^ */.exec(line)[0].length;
}

// 按 GitHub Actions 缩进解析 job/step，直接读仓库 ci.yml，不引入 YAML 依赖。
function parseWorkflowJobs(yaml) {
  const lines = yaml.split("\n");
  const jobsIndex = lines.indexOf("jobs:");
  assert.notEqual(jobsIndex, -1, "workflow 缺少 jobs");

  const jobStarts = [];
  for (let index = jobsIndex + 1; index < lines.length; index += 1) {
    const match = lines[index].match(/^  ([A-Za-z0-9_-]+):\s*$/);
    if (match) {
      jobStarts.push({ id: match[1], index });
    }
  }

  const jobs = {};
  for (let jobIndex = 0; jobIndex < jobStarts.length; jobIndex += 1) {
    const start = jobStarts[jobIndex].index;
    const end = jobIndex + 1 < jobStarts.length
      ? jobStarts[jobIndex + 1].index
      : lines.length;
    jobs[jobStarts[jobIndex].id] = parseJob(lines.slice(start, end));
  }
  return jobs;
}

function parseJob(jobLines) {
  const idMatch = jobLines[0].match(/^  ([A-Za-z0-9_-]+):\s*$/);
  const job = {
    id: idMatch ? idMatch[1] : "",
    name: null,
    needs: [],
    if: null,
    runsOn: null,
    steps: []
  };

  for (let index = 1; index < jobLines.length; index += 1) {
    const line = jobLines[index];
    if (line === "    steps:") {
      job.steps = parseSteps(jobLines.slice(index + 1));
      break;
    }

    const nameMatch = line.match(/^    name:\s*(.*)$/);
    if (nameMatch) {
      job.name = unquoteYamlScalar(nameMatch[1]);
      continue;
    }
    const ifMatch = line.match(/^    if:\s*(.*)$/);
    if (ifMatch) {
      job.if = unquoteYamlScalar(ifMatch[1]);
      continue;
    }
    const runsOnMatch = line.match(/^    runs-on:\s*(.*)$/);
    if (runsOnMatch) {
      job.runsOn = unquoteYamlScalar(runsOnMatch[1]);
      continue;
    }
    const needsInline = line.match(/^    needs:\s*\[([^\]]*)\]\s*$/);
    if (needsInline) {
      job.needs = needsInline[1]
        .split(",")
        .map((item) => unquoteYamlScalar(item))
        .filter(Boolean);
      continue;
    }
    if (/^    needs:\s*$/.test(line)) {
      index += 1;
      while (index < jobLines.length) {
        const item = jobLines[index].match(/^      - ([A-Za-z0-9_-]+)\s*$/);
        if (!item) {
          index -= 1;
          break;
        }
        job.needs.push(item[1]);
        index += 1;
      }
    }
  }
  return job;
}

function parseSteps(stepLines) {
  const steps = [];
  let current = null;
  let block = null;

  function finishBlock() {
    if (!current || !block) {
      return;
    }
    const indent = block.minIndent || 0;
    current.run = block.lines
      .map((raw) => (raw === "" ? "" : raw.slice(indent)))
      .join("\n")
      .replace(/\n+$/u, "");
    current.multiline = true;
    block = null;
  }

  function finishStep() {
    finishBlock();
    if (current) {
      steps.push(current);
      current = null;
    }
  }

  for (const line of stepLines) {
    if (/^      - /.test(line)) {
      finishStep();
      current = {
        name: null,
        shell: null,
        run: null,
        multiline: false,
        uses: null
      };
      const nameMatch = line.match(/^      - name:\s*(.*)$/);
      if (nameMatch) {
        current.name = unquoteYamlScalar(nameMatch[1]);
      }
      continue;
    }

    if (block) {
      if (line.trim() === "") {
        block.lines.push("");
        continue;
      }
      const indent = indentOf(line);
      if (indent > 8) {
        if (block.minIndent === null) {
          block.minIndent = indent;
        }
        block.lines.push(line);
        continue;
      }
      finishBlock();
    }

    if (!current) {
      continue;
    }

    const shellMatch = line.match(/^        shell:\s*(.*)$/);
    if (shellMatch) {
      current.shell = unquoteYamlScalar(shellMatch[1]);
      continue;
    }
    const usesMatch = line.match(/^        uses:\s*(.*)$/);
    if (usesMatch) {
      current.uses = unquoteYamlScalar(usesMatch[1]);
      continue;
    }
    if (/^        run:\s*\|[+-]?\s*$/.test(line)) {
      block = { minIndent: null, lines: [] };
      continue;
    }
    const runMatch = line.match(/^        run:\s*(.+)$/);
    if (runMatch) {
      current.run = unquoteYamlScalar(runMatch[1]);
      current.multiline = false;
    }
  }
  finishStep();
  return steps;
}

function runPwshFile(scriptPath) {
  return childProcess.spawnSync(
    "pwsh",
    ["-NoProfile", "-File", scriptPath],
    { encoding: "utf8", windowsHide: true }
  );
}

function pwshAvailable() {
  const result = childProcess.spawnSync(
    "pwsh",
    ["-NoProfile", "-Command", "exit 0"],
    { encoding: "utf8", windowsHide: true }
  );
  return result.error == null && result.status === 0;
}

function githubPwshWrapper(body) {
  return [
    "$ErrorActionPreference = 'stop'",
    body,
    "if ((Test-Path -LiteralPath variable:\\LASTEXITCODE)) { exit $LASTEXITCODE }",
    ""
  ].join("\n");
}

test("CI monitor 作业包含版本对齐检查", () => {
  const yaml = readCiWorkflowYaml();
  assert.match(yaml, /node scripts\/sync-monitor-version\.js --check/);
});

test("CI monitor 原生命令各自独立为 pwsh step", () => {
  const jobs = parseWorkflowJobs(readCiWorkflowYaml());
  const monitor = jobs.monitor;
  assert.ok(monitor, "缺少 monitor job");

  const alignmentIndex = monitor.steps.findIndex((step) => (
    step.run === "node scripts/sync-monitor-version.js --check"
  ));
  assert.notEqual(alignmentIndex, -1, "缺少版本对齐 step");

  const nativeSteps = monitor.steps.slice(alignmentIndex + 1);
  assert.equal(nativeSteps.length, MONITOR_NATIVE_COMMANDS.length);

  for (const [index, command] of MONITOR_NATIVE_COMMANDS.entries()) {
    const step = nativeSteps[index];
    assert.equal(step.shell, "pwsh", `命令应使用 pwsh：${command}`);
    assert.equal(step.multiline, false, `命令不得放在多行脚本：${command}`);
    assert.equal(step.run, command);
  }

  const combined = monitor.steps.filter((step) => {
    if (!step.run) {
      return false;
    }
    return MONITOR_NATIVE_COMMANDS.filter((command) => step.run.includes(command)).length > 1;
  });
  assert.equal(combined.length, 0, "多条 monitor 原生命令不得出现在同一 run 块");
});

test("CI required-checks 聚合 test、monitor 与 docs", () => {
  const jobs = parseWorkflowJobs(readCiWorkflowYaml());
  const required = jobs["required-checks"];
  assert.ok(required, "缺少 required-checks job");
  assert.equal(required.name, "Required checks");
  assert.equal(required.if, "always()");
  assert.deepEqual(required.needs, ["test", "monitor", "docs"]);

  const docs = jobs.docs;
  assert.ok(docs, "缺少 docs job");
  assert.equal(docs.runsOn, "ubuntu-latest");
  const docsRuns = docs.steps.map((step) => step.run).filter(Boolean);
  assert.equal(docsRuns.includes("npm --prefix docs ci"), true);
  assert.equal(docsRuns.includes("npm --prefix docs run build"), true);
  assert.equal(
    docs.steps.filter((step) => step.run && step.run.includes("npm --prefix docs")).length,
    2
  );
  assert.ok(docs.steps.some((step) => step.uses === "actions/checkout@v7"));
  assert.ok(docs.steps.some((step) => step.uses === "actions/setup-node@v7"));
  assert.match(readCiWorkflowYaml().split(/^  docs:\s*$/m)[1], /node-version:\s*"22"/);

  const verify = required.steps.find((step) => (
    step.name === "Verify all test jobs passed"
  ));
  assert.ok(verify && verify.run, "缺少聚合结果判断 step");
  const resultJobs = [...verify.run.matchAll(/needs\.([A-Za-z0-9_-]+)\.result/g)]
    .map((match) => match[1]);
  assert.deepEqual([...new Set(resultJobs)].sort(), ["docs", "monitor", "test"]);
  assert.match(verify.run, /needs\.test\.result[^\n]*success/);
  assert.match(verify.run, /needs\.monitor\.result[^\n]*success/);
  assert.match(verify.run, /needs\.docs\.result[^\n]*success/);
});

test("pwsh 多命令脚本会把中间 native 失败覆盖为成功", { skip: !pwshAvailable() }, () => {
  withTemporaryDirectory((directory) => {
    const scriptPath = path.join(directory, "old-semantics.ps1");
    fs.writeFileSync(
      scriptPath,
      githubPwshWrapper([
        "node -e \"process.exit(7)\"",
        "Write-Output \"SURVIVED_AFTER_EXIT_7\"",
        "node -e \"process.exit(0)\""
      ].join("\n")),
      "utf8"
    );
    const result = runPwshFile(scriptPath);
    assert.equal(result.status, 0);
    assert.match(result.stdout, /SURVIVED_AFTER_EXIT_7/);
  });
});

test("pwsh 单命令脚本保留非零退出码", { skip: !pwshAvailable() }, () => {
  withTemporaryDirectory((directory) => {
    const scriptPath = path.join(directory, "single-command.ps1");
    fs.writeFileSync(
      scriptPath,
      githubPwshWrapper("node -e \"process.exit(7)\""),
      "utf8"
    );
    const result = runPwshFile(scriptPath);
    assert.equal(result.status, 7);
  });
});

test("缺少 tauri.conf.json 失败", () => {
  withTemporaryDirectory((directory) => {
    writeTree(directory, "0.2.0");
    fs.unlinkSync(
      path.join(directory, "residential-monitor", "src-tauri", "tauri.conf.json")
    );
    assert.throws(
      () => syncMonitorVersion(directory),
      /缺少 residential-monitor\/src-tauri\/tauri.conf.json/
    );
  });
});

"use strict";

// 独立内核仅监听回环地址；不读取用户配置，不向住宅节点发起连接。
const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const net = require("node:net");
const { spawn, spawnSync } = require("node:child_process");
const { setTimeout: delay } = require("node:timers/promises");

async function freePort() {
  const server = net.createServer();
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const port = server.address().port;
  await new Promise((resolve) => server.close(resolve));
  return port;
}

async function main() {
  const binary = process.argv[2];
  assert.ok(binary, "需要指定已有 Mihomo 可执行文件");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "resi-empty-fallback-"));
  const port = await freePort();
  const config = {
    "external-controller": `127.0.0.1:${port}`,
    "allow-lan": false,
    "log-level": "warning",
    mode: "rule",
    dns: { enable: false },
    tun: { enable: false },
    proxies: [
      { name: "家宽-SOCKS5", type: "socks5", server: "127.0.0.1", port: 1, "dialer-proxy": "Proxy" },
      { name: "Airport", type: "socks5", server: "127.0.0.1", port: 2 }
    ],
    "proxy-groups": [{
      name: "Proxy", type: "select", "include-all-proxies": true,
      filter: "^NoAirportMatches$", "exclude-filter": "^家宽-SOCKS5$",
      "empty-fallback": "家宽-SOCKS5"
    }],
    rules: ["MATCH,REJECT"]
  };
  const configPath = path.join(directory, "config.json");
  fs.writeFileSync(configPath, JSON.stringify(config));
  let child;
  let log = "";
  try {
    const version = spawnSync(binary, ["-v"], { encoding: "utf8" });
    const check = spawnSync(binary, ["-t", "-d", directory, "-f", configPath], { encoding: "utf8", timeout: 15000 });
    process.stdout.write(version.stdout);
    process.stdout.write(JSON.stringify({ configExit: check.status, output: check.stdout + check.stderr }) + "\n");
    assert.equal(check.status, 0, "虚构配置未通过内核解析");
    child = spawn(binary, ["-d", directory, "-f", configPath], { stdio: ["ignore", "pipe", "pipe"] });
    child.stdout.on("data", (data) => { log += data; });
    child.stderr.on("data", (data) => { log += data; });
    let group;
    for (let attempt = 0; attempt < 50; attempt += 1) {
      try {
        const response = await fetch(`http://127.0.0.1:${port}/proxies/Proxy`, { signal: AbortSignal.timeout(500) });
        if (response.ok) { group = await response.json(); break; }
      } catch {}
      if (child.exitCode !== null) break;
      await delay(100);
    }
    assert.ok(group, `独立内核未就绪：${log}`);
    assert.equal(group.now, "家宽-SOCKS5");
    assert.deepEqual(group.all, ["家宽-SOCKS5"]);
    process.stdout.write(JSON.stringify({ group: group.name, now: group.now, all: group.all, homeDialer: "Proxy", trafficSent: false }) + "\n");
  } finally {
    if (child && child.exitCode === null) {
      const exited = new Promise((resolve) => child.once("exit", resolve));
      child.kill();
      await exited;
    }
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });

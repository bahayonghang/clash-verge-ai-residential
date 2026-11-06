"use strict";

// 回环 DNS/SOCKS5 服务验证真实内核选路，不使用公网或用户配置。
const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const net = require("node:net");
const dgram = require("node:dgram");
const vm = require("node:vm");
const { spawn, spawnSync } = require("node:child_process");
const { setTimeout: delay } = require("node:timers/promises");

const provider = "AI-家宽-DNS-REGEX";
const patterns = ["^[a-z0-9-]+-aiplatform\\.googleapis\\.com$", "^repo[0-9]+\\.cursor\\.sh$"];
const hosts = [
  "us-central1-aiplatform.googleapis.com", "europe-west4-aiplatform.googleapis.com", "repo42.cursor.sh",
  "maps.googleapis.com", "fonts.googleapis.com", "storage.googleapis.com", "repofoo.cursor.sh",
  "repo42.cursor.sh.example.test", "foo.us-central1-aiplatform.googleapis.com"
];
const sockets = new Set();
function track(socket) {
  sockets.add(socket);
  socket.on("error", () => {});
  socket.once("close", () => sockets.delete(socket));
  return socket;
}
async function listen(server) {
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  return server.address().port;
}
async function unusedPort() {
  const server = net.createServer();
  const port = await listen(server);
  await new Promise((resolve) => server.close(resolve));
  return port;
}
function makeDns(label, queries) {
  return net.createServer((client) => {
    track(client);
    let buffer = Buffer.alloc(0);
    client.on("data", (data) => {
      buffer = Buffer.concat([buffer, data]);
      while (buffer.length >= 2 && buffer.length >= buffer.readUInt16BE(0) + 2) {
        const length = buffer.readUInt16BE(0);
        const query = buffer.subarray(2, length + 2);
        buffer = buffer.subarray(length + 2);
        let offset = 12;
        const labels = [];
        while (query[offset]) {
          const size = query[offset++];
          labels.push(query.toString("ascii", offset, offset + size));
          offset += size;
        }
        offset += 1;
        assert.equal(query.readUInt16BE(offset), 16, "探测只使用 TXT 查询");
        queries.push({ host: labels.join("."), resolver: label });
        const question = query.subarray(12, offset + 4);
        const header = Buffer.alloc(12);
        query.copy(header, 0, 0, 2);
        header.writeUInt16BE(0x8180, 2);
        header.writeUInt16BE(1, 4);
        header.writeUInt16BE(1, 6);
        const answer = Buffer.alloc(13 + label.length);
        answer.writeUInt16BE(0xc00c, 0);
        answer.writeUInt16BE(16, 2);
        answer.writeUInt16BE(1, 4);
        answer.writeUInt16BE(label.length + 1, 10);
        answer[12] = label.length;
        answer.write(label, 13, "ascii");
        const response = Buffer.concat([header, question, answer]);
        const frame = Buffer.alloc(2);
        frame.writeUInt16BE(response.length);
        client.write(Buffer.concat([frame, response]));
      }
    });
  });
}
function makeSocks(label, allowedPorts, connections) {
  return net.createServer((client) => {
    track(client);
    let buffer = Buffer.alloc(0);
    let stage = 0;
    function receive(data) {
      buffer = Buffer.concat([buffer, data]);
      if (stage === 0) {
        if (buffer.length < 2 || buffer.length < 2 + buffer[1]) return;
        assert.equal(buffer[0], 5);
        buffer = buffer.subarray(2 + buffer[1]);
        client.write(Buffer.from([5, 0]));
        stage = 1;
      }
      if (buffer.length < 5) return;
      assert.equal(buffer[1], 1, "只允许 SOCKS5 CONNECT");
      const domain = buffer[3] === 3;
      assert.ok(domain || buffer[3] === 1, "只允许本地 IPv4/域名目标");
      const end = domain ? 5 + buffer[4] : 8;
      if (buffer.length < end + 2) return;
      const host = domain ? buffer.toString("ascii", 5, end) : [...buffer.subarray(4, 8)].join(".");
      const port = buffer.readUInt16BE(end);
      assert.equal(host, "127.0.0.1", "禁止连接公网目标");
      assert.ok(allowedPorts.has(port), "禁止连接测试以外的端口");
      connections.push({ via: label, port });
      client.removeListener("data", receive);
      const remaining = buffer.subarray(end + 2);
      client.pause();
      const outbound = track(net.connect(port, "127.0.0.1", () => {
        client.write(Buffer.from([5, 0, 0, 1, 127, 0, 0, 1, 0, 0]));
        if (remaining.length) outbound.write(remaining);
        client.pipe(outbound).pipe(client);
        client.resume();
      }));
      client.once("close", () => outbound.destroy());
      outbound.once("close", () => client.destroy());
    }
    client.on("data", receive);
  });
}
async function queryDns(port, host) {
  const socket = dgram.createSocket("udp4");
  try {
    const header = Buffer.alloc(12);
    header.writeUInt16BE(Math.floor(Math.random() * 65536), 0);
    header.writeUInt16BE(0x0100, 2);
    header.writeUInt16BE(1, 4);
    const labels = host.split(".").flatMap((part) => [Buffer.from([part.length]), Buffer.from(part)]);
    const packet = Buffer.concat([header, ...labels, Buffer.from([0, 0, 16, 0, 1])]);
    return await new Promise((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error(`DNS 查询超时：${host}`)), 4000);
      socket.once("error", (error) => { clearTimeout(timer); reject(error); });
      socket.once("message", (message) => {
        clearTimeout(timer);
        assert.equal(message.readUInt16BE(0), header.readUInt16BE(0));
        resolve(message.toString("ascii").endsWith("residential") ? "residential" :
          message.toString("ascii").endsWith("airport") ? "airport" : "unexpected");
      });
      socket.send(packet, port, "127.0.0.1");
    });
  } finally { socket.close(); }
}

async function main() {
  const binary = process.argv[2];
  const sourcePath = process.argv[3];
  assert.ok(binary, "需要指定已有 Mihomo 可执行文件");
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "resi-regex-dns-"));
  const queries = [];
  const connections = [];
  const residentialDns = makeDns("residential", queries);
  const airportDns = makeDns("airport", queries);
  const services = [residentialDns, airportDns];
  let child;
  try {
    process.stdout.write(spawnSync(binary, ["-v"], { encoding: "utf8" }).stdout);
    const residentialPort = await listen(residentialDns);
    const airportPort = await listen(airportDns);
    const home = makeSocks("home", new Set([residentialPort]), connections);
    services.push(home);
    const homePort = await listen(home);
    const airport = makeSocks("airport", new Set([homePort, airportPort]), connections);
    services.push(airport);
    const airportProxyPort = await listen(airport);
    const homeResolver = `tcp://127.0.0.1:${residentialPort}#AI-家宽`;
    const airportResolver = `tcp://127.0.0.1:${airportPort}#Proxy`;
    for (const [vertex, cursor] of [[true, false], [false, true], [true, true], [false, false]]) {
      const controllerPort = await unusedPort();
      const dnsPort = await unusedPort();
      const active = patterns.filter((_, index) => index === 0 ? vertex : cursor);
      let config = {
        proxies: [
          { name: "Airport", type: "socks5", server: "127.0.0.1", port: airportProxyPort, udp: true },
          { name: "家宽-SOCKS5", type: "socks5", server: "127.0.0.1", port: homePort, udp: true, "dialer-proxy": "Proxy" }
        ],
        "proxy-groups": [
          { name: "Proxy", type: "select", proxies: ["Airport"] },
          { name: "AI-家宽", type: "select", proxies: ["家宽-SOCKS5"] }
        ],
        rules: ["MATCH,REJECT"]
      };
      if (sourcePath) {
        const sandbox = { module: { exports: {} }, console: { info() {}, warn() {}, error() {} } };
        const source = fs.readFileSync(sourcePath, "utf8")
          .replace("const ROUTE_VERTEX_AI_ENDPOINTS = true;", `const ROUTE_VERTEX_AI_ENDPOINTS = ${vertex};`)
          .replace("const ROUTE_CURSOR_REPOSITORY_INDEXING = false;", `const ROUTE_CURSOR_REPOSITORY_INDEXING = ${cursor};`);
        vm.runInNewContext(source, sandbox);
        config = sandbox.module.exports.main(config, "isolated-probe");
        // 只替换 resolver 地址和移除地理库依赖；保留生成的 provider、策略和 SOCKS 链路。
        for (const key of Object.keys(config.dns["nameserver-policy"])) {
          if (key.startsWith("geosite:")) delete config.dns["nameserver-policy"][key];
          else if (config.dns["nameserver-policy"][key].some((entry) => entry.includes("#AI-家宽"))) {
            config.dns["nameserver-policy"][key] = [homeResolver];
          }
        }
      } else {
        if (active.length) config["rule-providers"] = {
          [provider]: { type: "inline", behavior: "classical", payload: active.map((pattern) => `DOMAIN-REGEX,${pattern}`) }
        };
        config.dns = { "nameserver-policy": active.length ? { [`rule-set:${provider}`]: [homeResolver] } : {} };
      }
      Object.assign(config, { "external-controller": `127.0.0.1:${controllerPort}`, "allow-lan": false, "log-level": "warning", tun: { enable: false } });
      Object.assign(config.dns, {
        enable: true, ipv6: false, listen: `127.0.0.1:${dnsPort}`, "enhanced-mode": "fake-ip",
        "respect-rules": true, nameserver: [airportResolver], "default-nameserver": ["127.0.0.1"],
        "proxy-server-nameserver": [`tcp://127.0.0.1:${airportPort}#DIRECT`],
        "direct-nameserver": [`tcp://127.0.0.1:${airportPort}#DIRECT`]
      });
      const configPath = path.join(directory, "config.json");
      fs.writeFileSync(configPath, JSON.stringify(config));
      const check = spawnSync(binary, ["-t", "-d", directory, "-f", configPath], { encoding: "utf8", timeout: 10000 });
      assert.equal(check.status, 0, check.stdout + check.stderr);
      let log = "";
      child = spawn(binary, ["-d", directory, "-f", configPath], { stdio: ["ignore", "pipe", "pipe"] });
      child.stdout.on("data", (data) => { log += data; });
      child.stderr.on("data", (data) => { log += data; });
      let ready = false;
      for (let attempt = 0; attempt < 50; attempt += 1) {
        try {
          const response = await fetch(`http://127.0.0.1:${controllerPort}/version`, { signal: AbortSignal.timeout(500) });
          if (response.ok) { ready = true; break; }
        } catch {}
        if (child.exitCode !== null) break;
        await delay(100);
      }
      assert.ok(ready, log);
      const rows = [];
      for (const host of hosts) {
        const expected = active.some((pattern) => new RegExp(pattern).test(host)) ? "residential" : "airport";
        const startQueries = queries.length;
        const startConnections = connections.length;
        const actual = await queryDns(dnsPort, host);
        assert.equal(actual, expected, host);
        assert.ok(queries.slice(startQueries).some((query) => query.host === host && query.resolver === expected), "必须实际访问 resolver");
        if (expected === "residential") {
          assert.ok(connections.slice(startConnections).some((item) => item.via === "home" && item.port === residentialPort));
          assert.ok(connections.slice(startConnections).some((item) => item.via === "airport" && item.port === homePort));
        } else {
          assert.ok(connections.slice(startConnections).some((item) => item.via === "airport" && item.port === airportPort));
        }
        rows.push({ host, resolver: actual });
      }
      process.stdout.write(JSON.stringify({ source: sourcePath ? "generated" : "preflight", vertex, cursor, configExit: check.status, queries: rows }) + "\n");
      const exited = new Promise((resolve) => child.once("exit", resolve));
      child.kill();
      await exited;
      child = null;
    }
  } finally {
    if (child && child.exitCode === null) {
      const exited = new Promise((resolve) => child.once("exit", resolve));
      child.kill();
      await exited;
    }
    for (const socket of sockets) socket.destroy();
    await Promise.all(services.map((server) => new Promise((resolve) => server.close(resolve))));
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
main().catch((error) => { console.error(error); process.exitCode = 1; });

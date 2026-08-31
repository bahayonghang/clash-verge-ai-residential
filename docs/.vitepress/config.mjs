import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { defineConfig } from "vitepress";

const docsDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = path.resolve(docsDir, "..");

const zhSidebar = [
  {
    text: "使用与配置",
    items: [
      { text: "概述", link: "/" },
      { text: "本地配置", link: "/local-configuration" },
      { text: "配置开关", link: "/configuration" },
      { text: "路由范围", link: "/routing-scope" },
      { text: "多 Profile", link: "/multi-profile" },
      { text: "DNS 与泄漏模型", link: "/dns-and-leak-model" },
      { text: "故障排查", link: "/troubleshooting" }
    ]
  },
  {
    text: "Agent",
    items: [
      { text: "Domain 文档", link: "/agents/domain" },
      { text: "Issue tracker", link: "/agents/issue-tracker" },
      { text: "分诊标签", link: "/agents/triage-labels" },
      { text: "家宽规则优化", link: "/agents/residential-rule-tuning" }
    ]
  }
];

const enSidebar = [
  {
    text: "Usage",
    items: [
      { text: "Overview", link: "/en/" },
      { text: "Local configuration", link: "/en/local-configuration" },
      { text: "Configuration", link: "/en/configuration" },
      { text: "Routing scope", link: "/en/routing-scope" },
      { text: "Multi-profile", link: "/en/multi-profile" },
      { text: "DNS and leak model", link: "/en/dns-and-leak-model" },
      { text: "Troubleshooting", link: "/en/troubleshooting" }
    ]
  },
  {
    text: "Agent",
    items: [
      { text: "Domain docs", link: "/en/agents/domain" },
      { text: "Issue tracker", link: "/en/agents/issue-tracker" },
      { text: "Triage labels", link: "/en/agents/triage-labels" },
      { text: "Residential rule tuning", link: "/en/agents/residential-rule-tuning" }
    ]
  }
];

export default defineConfig({
  srcExclude: ["adr/**"],
  title: "Clash Verge AI 家宽路由",
  description: "只把核心 AI 流量送进住宅 SOCKS5 链路",
  lastUpdated: false,
  vite: {
    server: {
      fs: {
        allow: [repoRoot]
      }
    },
    plugins: [
      {
        name: "docs-repo-assets-and-adr-block",
        configureServer(server) {
          const assetsDir = path.join(repoRoot, "assets");
          server.middlewares.use((req, res, next) => {
            const url = (req.url || "").split("?")[0];
            if (url === "/adr" || url.startsWith("/adr/")) {
              res.statusCode = 404;
              res.setHeader("Content-Type", "text/plain; charset=utf-8");
              res.end("Not found");
              return;
            }
            if (!url.startsWith("/assets/")) {
              next();
              return;
            }
            const name = decodeURIComponent(url.slice("/assets/".length));
            if (!name || name.includes("/") || name.includes("\\") || name.includes("..")) {
              next();
              return;
            }
            const file = path.join(assetsDir, name);
            if (!fs.existsSync(file)) {
              next();
              return;
            }
            res.setHeader("Content-Type", "image/png");
            fs.createReadStream(file).pipe(res);
          });
        }
      }
    ]
  },
  themeConfig: {
    search: {
      provider: "local"
    },
    socialLinks: [
      {
        icon: "github",
        link: "https://github.com/bahayonghang/clash-verge-ai-residential"
      }
    ]
  },
  locales: {
    root: {
      label: "简体中文",
      lang: "zh-CN",
      themeConfig: {
        nav: [
          { text: "使用与配置", link: "/local-configuration" },
          { text: "Agent", link: "/agents/domain" }
        ],
        sidebar: zhSidebar,
        outlineTitle: "本页目录"
      }
    },
    en: {
      label: "English",
      lang: "en-US",
      title: "Clash Verge AI Residential",
      description: "Route only core AI traffic through a residential SOCKS5 chain",
      themeConfig: {
        nav: [
          { text: "Usage", link: "/en/local-configuration" },
          { text: "Agent", link: "/en/agents/domain" }
        ],
        sidebar: enSidebar,
        outlineTitle: "On this page"
      }
    }
  }
});

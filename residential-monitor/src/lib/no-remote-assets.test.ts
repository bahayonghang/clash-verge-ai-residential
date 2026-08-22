import { describe, expect, it } from "vitest";

const sources = import.meta.glob(
  [
    "../app.tsx",
    "../main.tsx",
    "../components/**/*.ts",
    "../components/**/*.tsx",
    "../hooks/**/*.ts",
    "../lib/utils.ts",
    "../lib/time-range.ts",
    "../lib/health.ts",
    "../styles/globals.css"
  ],
  { query: "?raw", eager: true, import: "default" }
) as Record<string, string>;

const htmlFiles = import.meta.glob(["../../index.html"], {
  query: "?raw",
  eager: true,
  import: "default"
}) as Record<string, string>;

const CATPPUCCIN_HEX = [
  "#181825",
  "#1e1e2e",
  "#313244",
  "#cdd6f4",
  "#a6adc8",
  "#89b4fa",
  "#74c7ec",
  "#11111b",
  "#f38ba8",
  "#a6e3a1",
  "#b4befe",
  "#6c7086",
  "#94e2d5",
  "#cba6f7",
  "#eff1f5",
  "#4c4f69",
  "#1e66f5"
];

describe("新壳资源约束", () => {
  it("index.html 只引用 /src/main.tsx，不含外部 URL", () => {
    const html = Object.values(htmlFiles).join("\n");
    expect(html.length).toBeGreaterThan(0);
    expect(html).toContain('src="/src/main.tsx"');
    expect(html).not.toContain('src="/src/main.ts"');
    expect(html).not.toContain("styles.css");
    expect(html).not.toMatch(/https?:\/\//i);
  });

  it("新增文件不含外部 URL", () => {
    const hits = Object.entries(sources)
      .filter(([, text]) => /https?:\/\//i.test(text) || /url\(\s*['"]https?:/i.test(text))
      .map(([file]) => file);
    expect(hits).toEqual([]);
  });

  it("新增文件不含 Catppuccin 硬编码色值", () => {
    const hits = Object.entries(sources)
      .filter(([, text]) => CATPPUCCIN_HEX.some((hex) => text.toLowerCase().includes(hex)))
      .map(([file]) => file);
    expect(hits).toEqual([]);
  });

  it("应用壳不含 PagePending 与 styles.css 引用", () => {
    const hits = Object.entries(sources)
      .filter(
        ([, text]) =>
          text.includes("PagePending") ||
          text.includes("page-pending") ||
          text.includes("styles.css")
      )
      .map(([file]) => file);
    expect(hits).toEqual([]);
  });
});

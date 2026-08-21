import { describe, expect, it } from "vitest";

const sources = import.meta.glob(["./index.tsx"], {
  query: "?raw",
  eager: true,
  import: "default"
}) as Record<string, string>;

describe("概览页查询次数", () => {
  it("三次 useReport：host / chain / process，趋势复用 host，无第四次", () => {
    const source = Object.values(sources)[0] ?? "";
    expect(source.match(/useReport\(/g)?.length).toBe(3);
    expect(source).toContain('grouping: "host"');
    expect(source).toContain('grouping: "chain"');
    expect(source).toContain('grouping: "process"');
    expect(source).toContain("趋势图复用 host");
    expect(source).not.toContain('grouping: "rule"');
  });
});

import { describe, expect, it } from "vitest";

const sources = import.meta.glob(["../../../app.tsx", "./dimension-page.tsx"], {
  query: "?raw",
  eager: true,
  import: "default"
}) as Record<string, string>;

describe("聚合页骨架", () => {
  it("四个维度由同一 DimensionPage 按 kind 参数化，app 不复制四份页面", () => {
    const app = Object.entries(sources).find(([file]) => file.endsWith("app.tsx"))?.[1] ?? "";
    const page = Object.entries(sources).find(([file]) => file.endsWith("dimension-page.tsx"))?.[1] ?? "";
    expect(app).toContain("case \"host\"");
    expect(app).toContain("case \"rule\"");
    expect(app).toContain("case \"chain\"");
    expect(app).toContain("case \"process\"");
    expect(app).toContain("kind={route}");
    expect(app).toContain("overview={overview}");
    expect(app.match(/DimensionPage/g)?.length).toBe(2);
    expect(page).toContain("kind: DimensionKind");
    expect(page).toContain("setSelected(null)");
    expect(page).not.toContain("function HostPage");
    expect(page).not.toContain("function RulePage");
  });
});

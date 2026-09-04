import { describe, expect, it } from "vitest";
import { LIST_PAGE_DEFAULT } from "../../../ipc/live-session";
import { livePageStatusText } from "./index";
import indexSource from "./index.tsx?raw";

describe("Live 页翻页工具条", () => {
  it("工具条显示页码与 matchedCount，空表不用当前页长度冒充总数", () => {
    expect(indexSource).toContain("live.page.status");
    expect(indexSource).toContain("loadNext");
    expect(indexSource).toContain("loadPrev");
    expect(indexSource).toContain("live.page?.matchedCount");
    expect(indexSource).not.toMatch(/rowCount:\s*rows\.length/);
    expect(indexSource).not.toMatch(/useLivePage\(\{[^}]*cursor:/);
    const matched = LIST_PAGE_DEFAULT + 1;
    const text = livePageStatusText("zh", 2, matched);
    expect(text).toContain("2");
    expect(text).toContain(String(matched));
    expect(text).not.toContain(` / ${LIST_PAGE_DEFAULT} `);
    expect(livePageStatusText("en", 2, matched)).toContain(String(matched));
  });

  it("消失 id 仍从 Channel 身份集检测", () => {
    expect(indexSource).toContain("new Set(stream.connections)");
    expect(indexSource).toContain("promoteAcceptedToClosed");
  });
});

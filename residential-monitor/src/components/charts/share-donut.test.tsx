import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { ShareDonut } from "./share-donut";

describe("ShareDonut", () => {
  it("无数据时渲染虚线空态，不画 0 高度图", () => {
    const html = renderToStaticMarkup(
      <ShareDonut data={[]} loading={false} emptyHint="无数据或能力不支持" />
    );
    expect(html).toContain("无数据或能力不支持");
    expect(html).toContain("border-dashed");
    expect(html).not.toContain("recharts");
  });
});

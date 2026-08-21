import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { rankingShare } from "../../format/rank";
import { TopListItem } from "./top-list-item";

describe("TopListItem 占比分母", () => {
  it("进度条宽度按 totals 计算，不按可见行之和", () => {
    expect(rankingShare(50, 200)).toBe(0.25);
    expect(rankingShare(50, 50)).toBe(1);
    const html = renderToStaticMarkup(
      <TopListItem rank={1} icon={null} title="a.example" value={50} total={200} />
    );
    expect(html).toContain("width:25%");
  });
});

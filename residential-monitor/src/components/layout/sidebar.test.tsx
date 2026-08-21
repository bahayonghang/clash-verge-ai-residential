import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { BUSINESS_ROUTES } from "../../nav-icons";
import { TooltipProvider } from "../ui/tooltip";
import { Sidebar } from "./sidebar";

const noop = (): void => undefined;
const resize = {
  onPointerDown: noop,
  onPointerMove: noop,
  onPointerUp: noop,
  onPointerCancel: noop,
  onKeyDown: noop,
  onKeyUp: noop,
  onBlur: noop
};

function renderSidebar(over: { locale?: "zh" | "en"; recovery?: boolean; width?: number; route?: "settings-data" | "overview" } = {}): string {
  return renderToStaticMarkup(
    <TooltipProvider>
      <Sidebar
        locale={over.locale ?? "zh"}
        route={over.route ?? "settings-data"}
        recovery={over.recovery ?? false}
        healthSession="no_data"
        healthLabel="无数据"
        width={over.width ?? 220}
        onRouteChange={noop}
        resize={resize}
      />
    </TooltipProvider>
  );
}

describe("Sidebar", () => {
  it("recovery-only 不渲染九段业务导航", () => {
    const html = renderToStaticMarkup(
      <TooltipProvider>
        <Sidebar
          locale="zh"
          route="settings-data"
          recovery
          healthSession="no_data"
          healthLabel="无数据"
          width={220}
          onRouteChange={noop}
          resize={resize}
        />
      </TooltipProvider>
    );
    expect(html).toContain("Recovery Shell");
    for (const id of BUSINESS_ROUTES) {
      expect(html).not.toContain(`data-route="${id}"`);
    }
    expect(html).toContain('data-route="settings-data"');
  });

  it("英文品牌为两行锁，导航与底栏标签单行完整", () => {
    const html = renderSidebar({ locale: "en", width: 220 });
    expect(html).toContain('data-brand="en-stack"');
    expect(html).toContain("Residential");
    expect(html).toContain("Traffic Monitor");
    expect(html).toContain("Observed lower bound, not a bill.");
    expect(html).not.toContain("The secret does not appear on this page.");
    expect(html).toContain("Live connections");
    expect(html).toContain("Settings / data");
    expect(html).toContain("min-w-0 truncate");
  });

  it("中文产品名单行，不使用英文两行锁", () => {
    const html = renderSidebar({ locale: "zh", width: 220 });
    expect(html).toContain("家宽流量监控");
    expect(html).not.toContain('data-brand="en-stack"');
    expect(html).toContain("设置 / 数据管理");
  });

  it("业务导航带色井与间距，关于与设置无 tint", () => {
    const html = renderSidebar({ locale: "zh", route: "overview" });
    for (const id of BUSINESS_ROUTES) {
      expect(html).toContain(`data-nav-tint="${id}"`);
    }
    expect(html).toContain("shell-nav-well");
    expect(html).toContain("gap-[length:var(--nav-item-gap)]");
    expect(html).toMatch(/data-nav-tint="overview"[^>]*aria-current="page"/);
    expect(html).toContain("bg-primary");
    const bottom = html.slice(html.indexOf('aria-label="侧栏底部"'));
    expect(bottom).toContain("关于");
    expect(bottom).toContain("设置 / 数据管理");
    expect(bottom).not.toContain("data-nav-tint");
    expect(bottom).not.toContain("shell-nav-well");
    expect(bottom).not.toContain('aria-current="page"');
  });
});

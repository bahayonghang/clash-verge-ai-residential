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
});

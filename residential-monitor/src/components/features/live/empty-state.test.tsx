import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { LiveEmptyKind } from "../../../ipc/live-empty";
import { EmptyState, LiveRecoveryActions, emptyCopy } from "./empty-state";

const noop = (): void => undefined;

function html(kind: LiveEmptyKind): string {
  return renderToStaticMarkup(
    <EmptyState
      kind={kind}
      locale="zh"
      healthTitle="控制器已断开"
      healthAction="检查 Verge / mihomo 后立即重连"
      onGoSettings={noop}
      onResubscribe={noop}
    />
  );
}

describe("EmptyState 六态", () => {
  it("未配置、未连接、暂停、已连接无行、订阅缺口、有行", () => {
    expect(html("unconfigured")).toContain("尚未配置控制器");
    expect(html("unconfigured")).toContain("去设置页");
    expect(html("disconnected")).toContain("控制器已断开");
    expect(html("disconnected")).toContain("下一步");
    expect(html("paused")).toContain("采集已暂停");
    expect(html("connectedEmpty")).toContain("当前没有活跃连接");
    expect(html("needResync")).toContain("重新订阅");
    expect(html("hasRows")).toBe("");
    expect(emptyCopy("disconnected", "zh", "控制器已断开", "立即重连")).toContain("立即重连");
    expect(
      renderToStaticMarkup(
        <LiveRecoveryActions kind="needResync" locale="zh" onGoSettings={noop} onResubscribe={noop} />
      )
    ).toContain("重新订阅");
    expect(
      renderToStaticMarkup(
        <LiveRecoveryActions kind="connectedEmpty" locale="zh" onGoSettings={noop} onResubscribe={noop} />
      )
    ).toBe("");
  });
});

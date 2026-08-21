import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { AlertInstance } from "../../../dto";
import { CenterList } from "./center-list";

function instance(over: Partial<AlertInstance> = {}): AlertInstance {
  return {
    instanceId: "i1",
    ruleId: "rate-home",
    ruleVersion: 1,
    selectorIdentity: "家宽",
    status: "not-evaluable",
    startedUtc: null,
    resolvedUtc: null,
    lastEvalUtc: 1,
    lastObserved: null,
    evidence: {
      ruleId: "rate-home",
      ruleVersion: 1,
      dataVersion: null,
      evaluatedAtUtc: 1,
      windowStartUtc: null,
      windowEndUtc: null,
      displayTimezone: "local",
      selector: "家宽",
      direction: "download",
      observedValue: null,
      triggerThreshold: 1,
      recoveryThreshold: null,
      coverageSummary: "gap",
      policyMetadata: null,
      reportQuery: null,
      notEvaluableReason: "coverage 不足"
    },
    ...over
  };
}

describe("CenterList", () => {
  it("空列表显示无告警，not-evaluable 与观测未知分开", () => {
    const empty = renderToStaticMarkup(
      <CenterList
        locale="zh"
        items={[]}
        filter="all"
        selectedId={null}
        hasMore={false}
        onFilter={() => undefined}
        onSelect={() => undefined}
        onMore={() => undefined}
      />
    );
    expect(empty).toContain("无告警");
    expect(empty).not.toContain("coverage 不足");

    const html = renderToStaticMarkup(
      <CenterList
        locale="zh"
        items={[instance()]}
        filter="all"
        selectedId={null}
        hasMore={false}
        onFilter={() => undefined}
        onSelect={() => undefined}
        onMore={() => undefined}
      />
    );
    expect(html).toContain("不可评估");
    expect(html).toContain("未知");
    expect(html).toContain("coverage 不足");
    expect(html).not.toContain("无告警");
  });
});

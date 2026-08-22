import { describe, expect, it } from "vitest";
import type { ResidentialShare } from "../dto";
import {
  beginShareRequest,
  finishShareRequest,
  shareReadout,
  type ResidentialShareViewState
} from "./use-residential-share";
import hookSource from "./use-residential-share.ts?raw";

function share(over: Partial<ResidentialShare> = {}): ResidentialShare {
  return {
    schemaVersion: 1,
    residentialUpload: 10,
    residentialDownload: 30,
    attributedUpload: 20,
    attributedDownload: 80,
    coverageStatus: "covered",
    namedSql: ["coverage_raw", "share_residential_raw"],
    generatedUtc: 1,
    targetCount: 1,
    policyVersion: 1,
    ...over
  };
}

function state(over: Partial<ResidentialShareViewState> = {}): ResidentialShareViewState {
  return { share: null, loading: false, errorZh: null, seq: 0, ...over };
}

describe("useResidentialShare 竞态", () => {
  it("过期响应不得覆盖最新结果，失败保留上次份额", () => {
    const first = beginShareRequest(state({ share: share({ residentialDownload: 1 }) }));
    const second = beginShareRequest(first);
    expect(second.seq).toBe(first.seq + 1);
    const stale = finishShareRequest(second, first.seq, {
      ok: true,
      share: share({ residentialDownload: 99 })
    });
    expect(stale.share?.residentialDownload).toBe(1);
    expect(stale.loading).toBe(true);
    const failed = finishShareRequest(second, second.seq, { ok: false, errorZh: "失败" });
    expect(failed.share?.residentialDownload).toBe(1);
    expect(failed.errorZh).toBe("失败");
    expect(failed.loading).toBe(false);
  });

  it("IPC 只在 hook 内", () => {
    expect(hookSource).toContain("residential_share");
    expect(hookSource).toContain("seqRef");
  });
});

describe("家宽占比三态", () => {
  it("任一字段为 None 时显示未知", () => {
    const readout = shareReadout(
      share({
        residentialUpload: null,
        residentialDownload: null,
        attributedUpload: null,
        attributedDownload: null,
        coverageStatus: "uncovered"
      })
    );
    expect(readout.kind).toBe("unknown");
    expect(readout.percent).toBeNull();
  });

  it("分母为零时显示未知", () => {
    const readout = shareReadout(
      share({
        residentialUpload: 0,
        residentialDownload: 0,
        attributedUpload: 0,
        attributedDownload: 0
      })
    );
    expect(readout.kind).toBe("zero-denominator");
    expect(readout.percent).toBeNull();
  });

  it("有覆盖且家宽为 0 时显示 0%", () => {
    const readout = shareReadout(
      share({
        residentialUpload: 0,
        residentialDownload: 0,
        attributedUpload: 10,
        attributedDownload: 10
      })
    );
    expect(readout.kind).toBe("zero-residential");
    expect(readout.percent).toBe(0);
  });

  it("分母大于 0 时显示百分比", () => {
    const readout = shareReadout(share());
    expect(readout.kind).toBe("percent");
    expect(readout.percent).toBe(40);
  });
});

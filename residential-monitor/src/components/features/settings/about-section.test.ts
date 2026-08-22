import { describe, expect, it } from "vitest";
import { decodeAbout } from "../../../dto";

describe("decodeAbout 关于页", () => {
  it("伪造 signed:true 载荷抛错，不渲染 signed", () => {
    expect(() =>
      decodeAbout({
        schemaVersion: 1,
        productName: "家宽流量监控",
        binaryName: "residential-monitor",
        identifier: "io.github.bahayonghang.residential-monitor",
        aumid: "io.github.bahayonghang.residential-monitor",
        version: "0.1.0",
        releasesUrl: "local-releases",
        signed: true,
        updaterPlugin: false,
        windowsService: false,
        signatureNoteZh: "伪造"
      })
    ).toThrow(/signed/);
  });
});

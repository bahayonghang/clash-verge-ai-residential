import { describe, expect, it } from "vitest";

const sources = import.meta.glob(["../**/*.ts", "../**/*.tsx"], {
  query: "?raw",
  eager: true,
  import: "default"
}) as Record<string, string>;

describe("IPC 边界", () => {
  it("components/** 不直接 invoke", () => {
    const hits = Object.entries(sources)
      .filter(([file]) => !file.includes(".test."))
      .filter(([, text]) => /\binvoke\s*[<(]/.test(text) || /from ["']@tauri-apps/.test(text))
      .map(([file]) => file);
    expect(hits).toEqual([]);
  });
});

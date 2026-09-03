import { describe, expect, it } from "vitest";
import {
  AutostartRequestController,
  type AutostartBackend,
  type AutostartRequestState
} from "./autostart-request";

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((next, fail) => {
    resolve = next;
    reject = fail;
  });
  return { promise, resolve, reject };
}

function controller(backend: AutostartBackend) {
  const states: AutostartRequestState[] = [];
  const request = new AutostartRequestController("zh", backend, (state) => states.push(state));
  return { request, states };
}

describe("AutostartRequestController", () => {
  it("loads OS truth without writing", async () => {
    let writes = 0;
    const { request } = controller({
      available: () => true,
      read: async () => ({ enabled: true }),
      write: async () => {
        writes += 1;
        return { enabled: false };
      }
    });
    await request.load();
    expect(request.snapshot()).toMatchObject({ enabled: true, loaded: true, loading: false });
    expect(writes).toBe(0);
  });

  it("commits the write readback and retains it after a later failure", async () => {
    let failRead = false;
    const { request } = controller({
      available: () => true,
      read: async () => {
        if (failRead) throw { messageZh: "系统读取失败" };
        return { enabled: false };
      },
      write: async () => ({ enabled: true })
    });
    await request.load();
    await request.setEnabled(true);
    expect(request.snapshot()).toMatchObject({ enabled: true, loaded: true, saving: false });
    failRead = true;
    await request.load();
    expect(request.snapshot()).toMatchObject({ enabled: true, loaded: true, loading: false });
    expect(request.snapshot().errorZh).toBe("系统读取失败");
  });

  it("ignores stale loads", async () => {
    const first = deferred<unknown>();
    const second = deferred<unknown>();
    let reads = 0;
    const { request } = controller({
      available: () => true,
      read: () => (++reads === 1 ? first.promise : second.promise),
      write: async () => ({ enabled: false })
    });
    const firstLoad = request.load();
    const secondLoad = request.load();
    second.resolve({ enabled: true });
    await secondLoad;
    first.resolve({ enabled: false });
    await firstLoad;
    expect(request.snapshot().enabled).toBe(true);
  });

  it("skips refresh and duplicate writes while saving", async () => {
    const pending = deferred<unknown>();
    let reads = 0;
    let writes = 0;
    const { request } = controller({
      available: () => true,
      read: async () => {
        reads += 1;
        return { enabled: false };
      },
      write: () => {
        writes += 1;
        return pending.promise;
      }
    });
    await request.load();
    const firstWrite = request.setEnabled(true);
    const duplicateWrite = request.setEnabled(true);
    await request.load();
    expect(reads).toBe(1);
    expect(writes).toBe(1);
    pending.resolve({ enabled: true });
    await Promise.all([firstWrite, duplicateWrite]);
    expect(request.snapshot()).toMatchObject({ enabled: true, saving: false });
  });

  it("keeps the switch unavailable outside Tauri", async () => {
    let calls = 0;
    const { request } = controller({
      available: () => false,
      read: async () => {
        calls += 1;
        return { enabled: true };
      },
      write: async () => {
        calls += 1;
        return { enabled: true };
      }
    });
    await request.load();
    await request.setEnabled(true);
    expect(calls).toBe(0);
    expect(request.snapshot()).toMatchObject({ enabled: false, loaded: false });
    expect(request.snapshot().errorZh).toContain("Tauri");
  });

  it("rejects an invalid readback without changing the confirmed state", async () => {
    let invalid = false;
    const { request } = controller({
      available: () => true,
      read: async () => ({ enabled: false }),
      write: async () => (invalid ? { enabled: "yes" } : { enabled: true })
    });
    await request.load();
    await request.setEnabled(true);
    invalid = true;
    await request.setEnabled(false);
    expect(request.snapshot().enabled).toBe(true);
    expect(request.snapshot().errorZh).not.toBeNull();
  });
});

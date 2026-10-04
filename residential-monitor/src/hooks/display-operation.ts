import { invoke } from "@tauri-apps/api/core";

/** 取消与释放只使用本次查询的 id，不能取消其它视图或后台任务。 */
export async function withDisplayOperation<T>(
  signal: AbortSignal,
  run: (operationId: string) => Promise<T>
): Promise<T> {
  const operationId = `display-${crypto.randomUUID()}`;
  let cancellation: Promise<unknown> | undefined;
  const cancel = (): void => {
    cancellation ??= invoke("cancel_operation", { operationId }).catch(() => undefined);
  };
  await invoke("start_operation", { operationId, kind: "display-query" });
  signal.addEventListener("abort", cancel, { once: true });
  try {
    if (signal.aborted) {
      cancel();
      throw new Error("展示查询已取消");
    }
    return await run(operationId);
  } finally {
    signal.removeEventListener("abort", cancel);
    await cancellation;
    // 清理失败不得覆盖原查询结果/错误。
    await invoke("finish_operation", { operationId }).catch(() => undefined);
  }
}

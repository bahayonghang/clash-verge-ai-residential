import type { CloseState } from "../../../dto";

export type CloseMark = CloseState["mark"];

export function setCloseMark(
  marks: ReadonlyMap<string, CloseMark>,
  identity: string,
  mark: CloseMark
): Map<string, CloseMark> {
  const next = new Map(marks);
  next.set(identity, mark);
  return next;
}

/** connectionDelta 的 remove 把 accepted 推进 closed。 */
export function promoteAcceptedToClosed(
  marks: ReadonlyMap<string, CloseMark>,
  disappeared: readonly string[]
): Map<string, CloseMark> {
  let changed = false;
  const next = new Map(marks);
  for (const identity of disappeared) {
    if (next.get(identity) === "accepted") {
      next.set(identity, "closed");
      changed = true;
    }
  }
  return changed ? next : new Map(marks);
}

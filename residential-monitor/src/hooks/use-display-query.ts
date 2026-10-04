import { useContext, useEffect, useState } from "react";
import { DisplayQuery } from "./display-query";
import { WindowVisibilityContext } from "./use-window-visibility";

export function useDisplayQuery(enabled = true): { queue: DisplayQuery; active: boolean } {
  const visible = useContext(WindowVisibilityContext);
  const active = visible && enabled;
  const [queue] = useState(() => new DisplayQuery());
  useEffect(() => {
    queue.setActive(active);
    return () => queue.setActive(false);
  }, [active, queue]);
  useEffect(() => () => queue.clear(), [queue]);
  return { queue, active };
}

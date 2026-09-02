import { useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  parseDimensionRankTableLayout,
  type DimensionRankTableLayout
} from "../dimension-rank-table-layout";
import { t, type UiLocale } from "../i18n";
import { isTauriRuntime } from "../ipc/live-session";
import { invokeErrorZh } from "../lib/utils";

/** 会话内共用，避免 DimensionPage `key={route}` 重挂载时回到过期 bootstrap。 */
let sessionLayout: DimensionRankTableLayout | null = null;

export function useDimensionRankTableLayout(
  initial: unknown,
  locale: UiLocale
): {
  layout: DimensionRankTableLayout;
  commitLayout: (next: DimensionRankTableLayout) => void;
  errorZh: string | null;
} {
  const seq = useRef(0);
  const [layout, setLayout] = useState(
    () => sessionLayout ?? parseDimensionRankTableLayout(initial)
  );
  const [errorZh, setErrorZh] = useState<string | null>(null);

  const commitLayout = useCallback(
    (next: DimensionRankTableLayout): void => {
      const token = ++seq.current;
      sessionLayout = next;
      setLayout(next);
      if (!isTauriRuntime()) {
        return;
      }
      void invoke<unknown>("save_dimension_rank_table_layout", { layout: next })
        .then((raw) => {
          if (token !== seq.current) {
            return;
          }
          const parsed = parseDimensionRankTableLayout(raw);
          sessionLayout = parsed;
          setLayout(parsed);
          setErrorZh(null);
        })
        .catch((caught: unknown) => {
          if (token !== seq.current) {
            return;
          }
          setErrorZh(invokeErrorZh(caught, t(locale, "live.layout_save_fail")));
        });
    },
    [locale]
  );

  return { layout, commitLayout, errorZh };
}

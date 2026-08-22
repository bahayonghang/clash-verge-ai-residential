import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { decodeResidentialShare, type ResidentialShare } from "../dto";
import { t } from "../i18n";
import { isTauriRuntime } from "../ipc/live-session";
import type { TimeRange } from "../lib/time-range";
import { invokeErrorZh } from "../lib/utils";
import { snapMsToMinute } from "./use-report";

export interface ResidentialShareViewState {
  share: ResidentialShare | null;
  loading: boolean;
  errorZh: string | null;
  seq: number;
}

export type ShareReadoutKind = "unknown" | "zero-denominator" | "zero-residential" | "percent";

export interface ShareReadout {
  kind: ShareReadoutKind;
  percent: number | null;
  residential: number | null;
  attributed: number | null;
}

export function beginShareRequest(state: ResidentialShareViewState): ResidentialShareViewState {
  return { ...state, seq: state.seq + 1, loading: true };
}

export function finishShareRequest(
  state: ResidentialShareViewState,
  seq: number,
  outcome: { ok: true; share: ResidentialShare } | { ok: false; errorZh: string }
): ResidentialShareViewState {
  if (seq !== state.seq) {
    return state;
  }
  if (outcome.ok) {
    return { seq: state.seq, share: outcome.share, loading: false, errorZh: null };
  }
  return { ...state, loading: false, errorZh: outcome.errorZh };
}

export function shareReadout(share: ResidentialShare | null): ShareReadout {
  if (
    share === null ||
    share.residentialUpload === null ||
    share.residentialDownload === null ||
    share.attributedUpload === null ||
    share.attributedDownload === null
  ) {
    return { kind: "unknown", percent: null, residential: null, attributed: null };
  }
  const residential = share.residentialUpload + share.residentialDownload;
  const attributed = share.attributedUpload + share.attributedDownload;
  if (attributed === 0) {
    return { kind: "zero-denominator", percent: null, residential, attributed };
  }
  const percent = (residential / attributed) * 100;
  if (residential === 0) {
    return { kind: "zero-residential", percent, residential, attributed };
  }
  return { kind: "percent", percent, residential, attributed };
}

export async function fetchResidentialShare(
  rangeStartUtc: number,
  rangeEndUtc: number,
  displayTimezone = "local"
): Promise<ResidentialShare> {
  const raw = await invoke<unknown>("residential_share", {
    rangeStartUtc,
    rangeEndUtc,
    displayTimezone
  });
  return decodeResidentialShare(raw);
}

export function useResidentialShare(timeRange: TimeRange, enabled = true): {
  share: ResidentialShare | null;
  loading: boolean;
  errorZh: string | null;
} {
  const [share, setShare] = useState<ResidentialShare | null>(null);
  const [loading, setLoading] = useState(false);
  const [errorZh, setErrorZh] = useState<string | null>(null);
  const seqRef = useRef(0);
  const startUtc = snapMsToMinute(timeRange.startUtc);
  const endUtc = snapMsToMinute(timeRange.endUtc);
  const query = useMemo(
    () => ({
      rangeStartUtc: Math.floor(startUtc / 1000),
      rangeEndUtc: Math.floor(endUtc / 1000)
    }),
    [endUtc, startUtc]
  );

  useEffect(() => {
    if (!enabled || !isTauriRuntime()) {
      setLoading(false);
      return;
    }
    const seq = ++seqRef.current;
    setLoading(true);
    let cancelled = false;
    void fetchResidentialShare(query.rangeStartUtc, query.rangeEndUtc)
      .then((next) => {
        if (cancelled || seq !== seqRef.current) {
          return;
        }
        setShare(next);
        setErrorZh(null);
        setLoading(false);
      })
      .catch((caught: unknown) => {
        if (cancelled || seq !== seqRef.current) {
          return;
        }
        setErrorZh(invokeErrorZh(caught, t("zh", "residential.fail")));
        setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [enabled, query]);

  return { share, loading, errorZh };
}

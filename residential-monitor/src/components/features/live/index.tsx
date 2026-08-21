import { useCallback, useEffect, useRef, useState } from "react";
import type { BootstrapDto, RouteId } from "../../../dto";
import { formatUtc } from "../../../format/units";
import { useLivePage } from "../../../hooks/use-live-page";
import { t, type UiLocale } from "../../../i18n";
import { liveEmptyKind } from "../../../ipc/live-empty";
import { defaultLiveQuery, LIST_PAGE_DEFAULT } from "../../../ipc/live-session";
import type { MonitorState } from "../../../ipc/reducer";
import { healthOf } from "../../../lib/health";
import { formatTemplate } from "../../../lib/utils";
import {
  appendLiveFilterClause,
  applyLiveFilterDraft,
  clearLiveFilterClauses,
  cloneLiveFilter,
  removeLiveFilterClause,
  type LiveFilterState
} from "../../../live-filter-workspace";
import { parseLiveTableLayout, type DataColumnId, type LiveTableLayout } from "../../../live-table-layout";
import { nextLiveSort, type LiveSortState } from "../../../live-table-sort";
import { Card, CardContent, CardHeader, CardTitle } from "../../ui/card";
import { ColumnMenu } from "./column-menu";
import { ConnectionTable } from "./connection-table";
import { promoteAcceptedToClosed, setCloseMark, type CloseMark } from "./close-marks";
import { LiveRecoveryActions } from "./empty-state";
import { FilterWorkspace, settleFilterStatus, type LiveFilterStatus } from "./filter-workspace";
import { HotspotCards } from "./hotspot-cards";

function initialFilter(): LiveFilterState {
  return cloneLiveFilter(defaultLiveQuery().filter);
}

export function LivePage({
  locale,
  boot,
  stream,
  autoRefresh,
  active = true,
  onRouteChange,
  onResubscribe
}: {
  locale: UiLocale;
  boot: BootstrapDto;
  stream: MonitorState;
  autoRefresh: boolean;
  active?: boolean;
  onRouteChange: (route: RouteId) => void;
  onResubscribe: () => void;
}) {
  const [applied, setApplied] = useState<LiveFilterState>(initialFilter);
  const [draft, setDraft] = useState<LiveFilterState>(initialFilter);
  const [editorIndex, setEditorIndex] = useState<number | null>(null);
  const [sort, setSort] = useState<LiveSortState>({ sortField: "identity", descending: false });
  const [layout, setLayout] = useState<LiveTableLayout>(() => parseLiveTableLayout(boot.liveTableLayout));
  const [closeMarks, setCloseMarks] = useState<Map<string, CloseMark>>(() => new Map());
  const [filterStatus, setFilterStatus] = useState<LiveFilterStatus>("idle");
  const filterLoadingRef = useRef(false);
  const live = useLivePage({
    applied,
    sort,
    cursor: null,
    refreshSignal: autoRefresh ? stream.lastSeq : null,
    locale,
    active
  });
  const { saveLayout, closeConnection } = live;

  const prevIdsRef = useRef<Set<string>>(new Set());
  const subRef = useRef(stream.subscriptionId);
  useEffect(() => {
    const current = new Set(stream.connections.keys());
    if (subRef.current !== stream.subscriptionId) {
      subRef.current = stream.subscriptionId;
      prevIdsRef.current = current;
      return;
    }
    const disappeared = [...prevIdsRef.current].filter((id) => !current.has(id));
    if (disappeared.length > 0) {
      setCloseMarks((marks) => promoteAcceptedToClosed(marks, disappeared));
    }
    prevIdsRef.current = current;
  }, [stream.connections, stream.lastSeq, stream.subscriptionId]);

  useEffect(() => {
    const next = settleFilterStatus(
      filterStatus,
      live.loading,
      live.queryFailed,
      filterLoadingRef.current
    );
    filterLoadingRef.current = next.sawLoading;
    if (next.status !== filterStatus) {
      setFilterStatus(next.status);
    }
  }, [filterStatus, live.loading, live.queryFailed]);

  const applyDraft = useCallback((): void => {
    const query = applyLiveFilterDraft(
      {
        filter: applied,
        sortField: sort.sortField,
        descending: sort.descending,
        cursor: null,
        limit: LIST_PAGE_DEFAULT
      },
      draft
    );
    const next = cloneLiveFilter(query.filter);
    setDraft(next);
    setEditorIndex(null);
    if (JSON.stringify(next) === JSON.stringify(applied)) {
      return;
    }
    setFilterStatus("applying");
    setApplied(next);
  }, [applied, draft, sort.descending, sort.sortField]);

  const cancelDraft = useCallback((): void => {
    setDraft(cloneLiveFilter(applied));
    setEditorIndex(null);
  }, [applied]);

  const commitLayout = useCallback(
    (next: LiveTableLayout): void => {
      setLayout(next);
      void saveLayout(next);
    },
    [saveLayout]
  );

  const address = boot.settings.address;
  const snapshot = stream.snapshot;
  const session = snapshot?.health.session ?? boot.overview.health.session;
  const health = healthOf(locale, session);
  const rows = live.page?.rows ?? [];
  const kind = liveEmptyKind({
    address,
    session,
    observationPhase: snapshot?.observationPhase ?? boot.overview.observationPhase,
    collectorRunning: live.collectorRunning,
    coverageKind: snapshot?.coverageKind ?? null,
    coverageReason: snapshot?.coverageReason ?? null,
    rowCount: rows.length,
    needResync: stream.needResync,
    frozen: stream.frozen,
    errorZh: live.errorZh ?? stream.errorZh
  });

  return (
    <div className="flex min-h-0 min-w-0 w-full flex-col gap-4">
      <div className="flex flex-wrap items-start justify-between gap-2">
        <div>
          <p className="text-sm" data-state={session}>
            {health.title}。{t(locale, "common.next")}：{health.action}
          </p>
          <p className="text-xs text-muted-foreground">
            {formatTemplate(t(locale, "live.last_sample"), {
              time: formatUtc(snapshot?.lastSampleUtc ?? null, t(locale, "common.no_sample"))
            })}
          </p>
          {live.collectorRunning === false ? (
            <p className="text-xs text-muted-foreground">{t(locale, "live.paused")}</p>
          ) : null}
        </div>
        <LiveRecoveryActions
          kind={kind}
          locale={locale}
          onGoSettings={() => onRouteChange("settings-data")}
          onResubscribe={onResubscribe}
        />
      </div>
      {live.errorZh ? (
        <p className="text-sm text-destructive" role="alert">
          {live.errorZh}
        </p>
      ) : null}
      <FilterWorkspace
        locale={locale}
        applied={applied}
        draft={draft}
        editorIndex={editorIndex}
        filterStatus={filterStatus}
        pageCount={rows.length}
        matchedCount={live.page?.matchedCount ?? null}
        emptyKind={kind}
        onDraftChange={setDraft}
        onApply={applyDraft}
        onCancel={cancelDraft}
        onAdd={() => {
          if (draft.clauses.length >= 8) {
            return;
          }
          const next = appendLiveFilterClause(draft, { field: "host", mode: "contains", value: "" });
          setDraft(next);
          setEditorIndex(next.clauses.length - 1);
        }}
        onClear={() => {
          const cleared = clearLiveFilterClauses(draft);
          const query = applyLiveFilterDraft(
            {
              filter: applied,
              sortField: sort.sortField,
              descending: sort.descending,
              cursor: null,
              limit: LIST_PAGE_DEFAULT
            },
            cleared
          );
          setFilterStatus("applying");
          setDraft(cloneLiveFilter(query.filter));
          setApplied(cloneLiveFilter(query.filter));
          setEditorIndex(null);
        }}
        onEdit={(index) => {
          setDraft(cloneLiveFilter(applied));
          setEditorIndex(index);
        }}
        onRemove={(index) => {
          const next = removeLiveFilterClause(applied, index);
          setFilterStatus("applying");
          setDraft(next);
          setApplied(next);
          setEditorIndex(null);
        }}
        onResidential={(value) => {
          const next = { ...draft, residentialOnly: value };
          setFilterStatus("applying");
          setDraft(next);
          setApplied(cloneLiveFilter(next));
          setEditorIndex(null);
        }}
      />
      <HotspotCards
        locale={locale}
        page={live.page}
        statusInput={{
          page: live.page,
          address,
          session,
          collectorRunning: live.collectorRunning,
          coverageKind: snapshot?.coverageKind ?? null,
          coverageReason: snapshot?.coverageReason ?? null,
          needResync: stream.needResync,
          frozen: stream.frozen
        }}
      />
      <Card className="min-h-0 min-w-0 overflow-hidden">
        <CardHeader className="flex flex-row items-center justify-between gap-2">
          <CardTitle className="text-sm">{t(locale, "live.table")}</CardTitle>
          <ColumnMenu locale={locale} layout={layout} onLayoutChange={commitLayout} />
        </CardHeader>
        <CardContent className="min-w-0">
          <ConnectionTable
            locale={locale}
            rows={rows}
            layout={layout}
            sort={sort}
            closeMarks={closeMarks}
            emptyKind={kind}
            healthTitle={health.title}
            healthAction={health.action}
            onSort={(column: DataColumnId) => setSort((current) => nextLiveSort(column, current))}
            onClose={(identity) => {
              void closeConnection(identity).then(
                (result) => {
                  setCloseMarks((marks) => setCloseMark(marks, identity, result.mark));
                },
                () => undefined
              );
            }}
            onLayoutCommit={commitLayout}
            onGoSettings={() => onRouteChange("settings-data")}
            onResubscribe={onResubscribe}
          />
        </CardContent>
      </Card>
    </div>
  );
}

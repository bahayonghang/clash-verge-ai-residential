"""Read only the approved generated corpus after the recorded first reads."""
from datetime import datetime, timezone
from pathlib import Path
import hashlib
import json
import re
import sqlite3
import time

ROOT = Path(__file__).resolve().parent
REPO = ROOT.parents[3]
DB = Path("C:/Users/lyh/AppData/Local/Temp/resiwatch-resource-measure-20260924-raw-fold/corpus-a250-30d/monitor.sqlite3")
OUT = ROOT / "corpus-a250-verification.json"
assert not OUT.exists()
marker = DB.parent / "production-corpus.json"
assert hashlib.sha256(marker.read_bytes()).hexdigest().upper() == "667A21E9FBE3FCF100974411D8E714E1589907E6458902AFD50683CEE68A3D42"
manifest = json.loads(marker.read_text())
assert manifest["kind"] == "production-corpus" and manifest["workload"]["average_active"] == 250
source = (REPO / "residential-monitor/src-tauri/src/c3/sql.rs").read_text(encoding="utf-8")
predicate = re.search(r'pub const RESIDENTIAL_RAW_MEMBERSHIP_SQL: &str = "(.*?)";', source).group(1)
start, end = manifest["start_utc"] // 60, manifest["end_utc"] // 60
report = {"started_utc": datetime.now(timezone.utc).isoformat(), "db": str(DB.resolve()), "marker_sha256": hashlib.sha256(marker.read_bytes()).hexdigest(), "cache": "after initial and corrected production/stage reads; not cold cache evidence", "sqlite_version": sqlite3.sqlite_version, "read_only_uri": True, "queries": []}


def save():
    OUT.write_text(json.dumps(report, ensure_ascii=False, indent=2) + chr(10), encoding="utf-8")


def query(connection, name, sql, params=()):
    item = {"name": name, "sql": sql, "params": params}
    report["queries"].append(item)
    save()
    print(json.dumps({"start": name}), flush=True)
    clock = time.perf_counter()
    try:
        result = connection.execute(sql, params).fetchall()
        item.update({"status": "ok", "result": result})
    except Exception as error:
        item.update({"status": "error", "error": str(error)})
        raise
    finally:
        item["wall_seconds"] = time.perf_counter() - clock
        save()
    print(json.dumps({"finished": name, "wall_seconds": item["wall_seconds"], "rows": len(result)}), flush=True)
    return result


with sqlite3.connect(DB.as_uri() + "?mode=ro", uri=True) as connection:
    connection.execute("pragma query_only = on")
    connection.execute("begin deferred")
    assert query(connection, "quick_check", "pragma quick_check") == [("ok",)]
    assert query(connection, "user_version", "pragma user_version") == [(5,)]
    assert query(connection, "journal_mode", "pragma journal_mode") == [("wal",)]
    versions = query(connection, "migration_identity", "select version, checksum from schema_migration order by version")
    assert versions[-1] == (5, "ledger-lifecycle-v5-layout3")
    for key, table in [("minutes", "connection_minute"), ("sessions", "connection_session"), ("chains", "connection_chain"), ("receipts", "committed_bundle")]:
        assert query(connection, key + "_count", "select count(*) from " + table) == [(manifest["expected"][key],)]
    join = " from connection_minute m join connection_session s on s.session_pk=m.session_pk left join connection_session_attr a on a.session_pk=s.session_pk"
    window = " where m.utc_minute>=? and m.utc_minute<?"
    sql = "select sum(m.upload),sum(m.download),count(distinct m.session_pk),count(distinct m.utc_minute),sum(case when " + predicate + " then m.upload else 0 end),sum(case when " + predicate + " then m.download else 0 end),count(distinct case when " + predicate + " then m.session_pk end),count(distinct case when " + predicate + " then m.utc_minute end)" + join + window
    totals = query(connection, "all_and_residential_totals_oracle", sql, (start, end))[0]
    assert totals == (151200035, 507599545, 2160000, 43200, 113399990, 380699625, 1620000, 43200), totals
    for kind, identity, suffix, top in [("host", "coalesce(nullif(s.host,''),'__unknown__')", "", 20), ("network", "coalesce(nullif(d.value,''),'__unknown__')", " left join dimension_dict d on d.dimension_kind='network' and d.dimension_id=a.network_id", 100)]:
        rank = "select " + identity + " as identity,sum(m.upload),sum(m.download),count(distinct m.session_pk),count(distinct m.utc_minute)*60" + join + suffix + window + " and " + predicate + " group by identity order by sum(m.download) desc,identity asc limit ?"
        query(connection, "residential_" + kind + "_rank_oracle", rank, (start, end, top))
    connection.execute("commit")
digest = hashlib.sha256()
with DB.open("rb") as stream:
    while data := stream.read(4 * 1024 * 1024):
        digest.update(data)
report.update({"database_sha256_after_probes": digest.hexdigest(), "database_bytes": DB.stat().st_size, "completed_utc": datetime.now(timezone.utc).isoformat(), "content_and_totals_pass": True})
save()
print(json.dumps({"completed": str(OUT), "content_and_totals_pass": True}), flush=True)

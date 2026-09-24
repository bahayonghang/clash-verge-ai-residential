"""Read only aggregate installed-ledger evidence; never print connection identities."""
import datetime
import json
import os
import pathlib
import sqlite3
import time

path = pathlib.Path(os.environ["LOCALAPPDATA"]) / "ResiWatch/data/monitor.sqlite3"
db = sqlite3.connect(path.as_uri() + "?mode=ro", uri=True, timeout=0.25)
db.execute("PRAGMA query_only=ON")
result = {"observed_at": datetime.datetime.now(datetime.timezone.utc).isoformat(), "mode": "ro; query_only; each statement has its own snapshot", "queries": {}}

def query(name, sql, seconds=3):
    start = time.monotonic()
    db.set_progress_handler(lambda: int(time.monotonic() - start > seconds), 1000)
    try:
        rows = db.execute(sql).fetchall()
        result["queries"][name] = {"rows": rows, "elapsed_ms": round((time.monotonic() - start) * 1000, 2)}
    except sqlite3.Error as error:
        result["queries"][name] = {"error": str(error), "elapsed_ms": round((time.monotonic() - start) * 1000, 2)}
    finally:
        db.set_progress_handler(None, 0)

for pragma in ["user_version", "page_size", "page_count", "freelist_count", "journal_mode"]:
    query(pragma, "PRAGMA " + pragma)
query("schema", "SELECT name, sql FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
query("object_bytes", "SELECT name, count(*) AS pages, sum(pgsize) AS bytes FROM dbstat GROUP BY name ORDER BY bytes DESC", 5)
for table in ["connection_minute", "connection_session", "connection_chain", "connection_session_attr", "traffic_hourly_dimension", "traffic_daily_dimension", "report_archive", "alert_event", "notification_outbox"]:
    query("count_" + table, 'SELECT count(*) FROM "' + table + '"')
query("retention_markers", "SELECT layer, watermark_utc, delete_watermark_utc FROM retention_watermark")
db.close()
print(json.dumps(result, ensure_ascii=False, indent=2))

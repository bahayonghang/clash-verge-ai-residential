"""只读采样隔离容量库的单行维护游标，不扫描明细或staging成员。"""

import json
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path


root = Path(sys.argv[1]).resolve()
manifest = json.loads((root / "production-corpus.json").read_text(encoding="utf-8"))
if manifest.get("kind") != "production-corpus":
    raise ValueError("只接受生成器标记的隔离容量库")
db = root / "monitor.sqlite3"
with sqlite3.connect(db.as_uri() + "?mode=ro", uri=True, timeout=1) as connection:
    connection.row_factory = sqlite3.Row
    job = connection.execute("select day_utc,phase,cursor_minute,cursor_session,raw_rows from retention_build where job_id=1").fetchone()
    result = {
        "observed_utc": datetime.now(timezone.utc).isoformat(),
        "job": dict(job) if job else None,
        "cleanup_cursors": dict(connection.execute("select key,value from machine_setting where key in ('session_cleanup_cursor','receipt_cleanup_cursor')")),
        "page_count": connection.execute("pragma page_count").fetchone()[0],
        "freelist_count": connection.execute("pragma freelist_count").fetchone()[0],
        "db_bytes": db.stat().st_size,
        "wal_bytes": Path(str(db) + "-wal").stat().st_size if Path(str(db) + "-wal").exists() else 0,
    }
print(json.dumps(result, ensure_ascii=False))

"""独立按生成公式核对完整日fixture的 raw + 已完整删日core；只读。"""

import json
import sqlite3
import sys
from pathlib import Path


root = Path(sys.argv[1]).resolve()
manifest = json.loads((root / "production-corpus.json").read_text(encoding="utf-8"))
if manifest["kind"] != "production-corpus":
    raise ValueError("需要生成器marker")
sessions = manifest["expected"]["sessions"]
if manifest["expected"]["minutes"] != sessions * 5:
    raise ValueError("此独立公式仅适用于完整5分钟会话fixture")
seed = manifest["workload"]["seed"] % 1_000_000_000


def modular_sum(modulus):
    cycles, tail = divmod(sessions, modulus)
    return cycles * modulus * (modulus - 1) // 2 + sum((i + seed) % modulus for i in range(1, tail + 1))


expected = {"upload": 5 * (8 * sessions + modular_sum(13)), "download": 5 * (32 * sessions + modular_sum(31))}
with sqlite3.connect((root / "monitor.sqlite3").as_uri() + "?mode=ro", uri=True, timeout=1) as connection:
    partial = connection.execute("select exists(select 1 from retention_state s where s.layer='day_exact_v1' and s.status='deleted' and exists(select 1 from connection_minute m where m.utc_minute>=s.chunk_utc/60 and m.utc_minute<(s.chunk_utc+86400)/60))").fetchone()[0]
    if partial:
        raise ValueError("已封存日期仍在分块删除；此时不能将完整core与残留raw直接相加")
    raw = connection.execute("select coalesce(sum(upload),0),coalesce(sum(download),0) from connection_minute").fetchone()
    core = connection.execute("select coalesce(sum(upload),0),coalesce(sum(download),0) from traffic_daily_core d where category_id=0 and exists(select 1 from retention_state s where s.layer='day_exact_v1' and s.chunk_utc=d.utc_day and s.status='deleted')").fetchone()
actual = {"upload": raw[0] + core[0], "download": raw[1] + core[1]}
result = {"fixture": str(root), "expected_by_generator_formula": expected, "raw_totals": list(raw), "fully_deleted_day_core_totals": list(core), "combined_totals": actual, "conserved": actual == expected}
print(json.dumps(result, ensure_ascii=False))
if actual != expected:
    raise SystemExit("字节不守恒")

"""仅查询明确绑定的新 A50/A250 合成库，不调用生成器或原生 rank。"""

import argparse
from contextlib import closing
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import sqlite3
import sys
import time

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[5]
ASSET = REPO / "bench-data/t05-rebuild-20261002-4f8a8c68"
START, DAY_START, END = 1787184000, 1789689600, 1789776000
TABLES = {"minutes": "connection_minute", "sessions": "connection_session",
          "chains": "connection_chain", "receipts": "committed_bundle"}
WORKLOAD = {"version": 1, "seed": 20260919, "average_active": None,
            "mean_session_minutes": 5.0, "mean_chain_nodes": 3.0,
            "nonzero_minute_ratio": 1.0, "days": 30, "sample_hz": 1,
            "domain_cardinality": 800, "process_cardinality": 120,
            "rule_cardinality": 40, "chain_cardinality": 60,
            "network_cardinality": 4, "frame_change_ratio": 1.0,
            "peak_active": 10000, "peak_minutes": 30,
            "long_process_path": True, "hostile_domain_mix": True}
DURABLE_SQL = "select max(watermark, coalesce((select max(data_version) from committed_bundle), 0)) from data_version where id=1"


def utc():
    return datetime.now(timezone.utc).isoformat()


def sha(path):
    digest = hashlib.sha256()
    with Path(path).open("rb") as stream:
        while data := stream.read(4 * 1024 * 1024):
            digest.update(data)
    return digest.hexdigest()


def wal_state(db):
    result = {}
    for suffix in ("-wal", "-shm"):
        path = db.with_name(db.name + suffix)
        result[suffix] = {"present": path.exists(), "bytes": path.stat().st_size if path.exists() else 0}
    return result


def require(condition, detail):
    if not condition:
        raise ValueError(detail)


def fixed_queries(contract):
    templates = contract["templates"]
    queries = []
    for window, start in (("30d", START), ("1d", DAY_START)):
        queries.append(("all_and_residential_totals_" + window,
                        templates["all_and_residential_totals_oracle"]["sql"], [start // 60, END // 60]))
    for kind, window, start, top in (("host", "30d", START, 20), ("host", "1d", DAY_START, 20),
                                   ("network", "30d", START, 100)):
        queries.append((kind + "-" + window, templates["residential_" + kind + "_rank_oracle"]["sql"],
                        [start // 60, END // 60, top]))
    return queries


def verify_corpus(active, db_arg, db_hash, marker_hash, contract, output):
    db = Path(db_arg)
    expected_path = ASSET / f"corpus-a{active}-30d/monitor.sqlite3"
    # 在任何输入库的内容读取或 SQLite 连接之前，限定到新资产根具名路径。
    require(db.is_absolute() and db.resolve() == expected_path.resolve(), "DB 路径不属于明确新合成语料")
    require(db.is_file(), "新合成 DB 不存在；真实 oracle 未执行")
    marker = db.parent / "production-corpus.json"
    require(marker.resolve().is_relative_to(ASSET.resolve()) and marker.is_file(), "新语料 marker 缺失或越界")
    for value in (db_hash, marker_hash):
        require(len(value) == 64 and all(c in "0123456789abcdefABCDEF" for c in value), "必须提供实际新 SHA256")
    record = {"active": active, "db": str(db.resolve()), "started_utc": utc(), "status": "RUNNING",
              "cache": "hash/integrity/oracle 将触页缓存；不提供物理冷页证据", "queries": [],
              "mode": "ro", "query_only": True, "transaction": "single deferred snapshot",
              "sqlite_version": sqlite3.sqlite_version}
    report_path = output / f"a{active}.json"

    def save():
        report_path.write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    def query(connection, name, sql, params=()):
        item = {"name": name, "sql": sql, "params": list(params), "status": "RUNNING"}
        record["queries"].append(item)
        save()
        started = time.perf_counter()
        try:
            rows = connection.execute(sql, params).fetchall()
            item.update({"status": "ok", "rows": rows})
            return rows
        except Exception as error:
            item.update({"status": "error", "error": repr(error)})
            raise
        finally:
            item["wall_seconds"] = time.perf_counter() - started
            save()

    try:
        record["marker_sha256_before"] = sha(marker)
        require(record["marker_sha256_before"] == marker_hash.lower(), "新 marker 身份不匹配")
        manifest = json.loads(marker.read_text(encoding="utf-8"))
        expected_workload = dict(WORKLOAD, average_active=active)
        require(manifest["kind"] == "production-corpus" and manifest["schema_version"] == 1,
                "语料 marker 类型或版本不匹配")
        require(manifest["workload"] == expected_workload, "固定 workload 不匹配")
        # WorkloadSpec::manifest_hash 按 Rust struct 声明顺序序列化，保留 f64 的 .0。
        payload = json.dumps(expected_workload, separators=(",", ":"), ensure_ascii=False).encode()
        require(manifest["spec_hash"] == hashlib.sha256(payload).hexdigest(), "spec_hash 不匹配")
        require(manifest["full_30_day_input"] is True and manifest["counts_match"] is True,
                "语料完整性标记不匹配")
        require((manifest["start_utc"], manifest["end_utc"], manifest["minutes"], manifest["generation_cache_kib"])
                == (START, END, 43200, 65536), "固定窗口或生成缓存字段不匹配")
        counts = {"minutes": active * 43200, "sessions": active * 8640,
                  "chains": active * 25920, "receipts": 2592000}
        require(manifest["expected"] == counts and manifest["actual"] == counts, "完整规模不匹配")
        require(manifest["sqlite_schema_version"] == 5, "marker schema 不匹配")
        record["marker"] = manifest
        record["sidecars_before"] = wal_state(db)
        require(record["sidecars_before"]["-wal"]["bytes"] == 0, "oracle 前有非零 WAL")
        record["database_sha256_before"] = sha(db)
        require(record["database_sha256_before"] == db_hash.lower(), "新 DB 身份不匹配")
        record["database_bytes"] = db.stat().st_size
        save()
        with closing(sqlite3.connect(db.as_uri() + "?mode=ro", uri=True)) as connection:
            connection.execute("pragma query_only=on")
            connection.execute("begin deferred")
            require(query(connection, "query_only", "pragma query_only") == [(1,)], "query_only 未生效")
            require(query(connection, "quick_check", "pragma quick_check") == [("ok",)], "quick_check 失败")
            require(query(connection, "user_version", "pragma user_version") == [(5,)], "schema 失败")
            require(query(connection, "journal_mode", "pragma journal_mode") == [("wal",)], "WAL 配置不匹配")
            versions = query(connection, "migration_identity", "select version, checksum from schema_migration order by version")
            require(versions[-1] == (5, "ledger-lifecycle-v5-layout3"), "migration identity 不匹配")
            for key, table in TABLES.items():
                require(query(connection, key + "_count", "select count(*) from " + table) == [(counts[key],)],
                        key + " count 不匹配")
            version = query(connection, "durable_data_version", DURABLE_SQL)
            require(version == [(2592000,)] and type(version[0][0]) is int, "durable dataVersion 不匹配")
            for name, sql, params in fixed_queries(contract):
                rows = query(connection, name, sql, params)
                require(bool(rows), "oracle 返回空结果：" + name)
                if name.startswith("all_and_residential"):
                    require(len(rows) == 1 and len(rows[0]) == 8 and all(type(v) is int for v in rows[0]),
                            "totals/distinct/minute coverage 类型不匹配")
                    if active == 250 and name.endswith("30d"):
                        require([list(row) for row in rows] == contract["templates"]["all_and_residential_totals_oracle"]["historical_rows"],
                                "A250 totals 与旧同输入语义结果不同")
                else:
                    require(all(len(row) == 5 and type(row[0]) is str and all(type(v) is int for v in row[1:]) for row in rows),
                            "rank 行形状或整数类型不匹配")
                    require(rows == sorted(rows, key=lambda row: (-row[2], row[0])), "download desc/identity asc 排序不匹配")
                    record["queries"][-1]["grouped_expectations"] = [
                        {"identity": row[0], "upload": row[1], "download": row[2], "connectionCount": row[3],
                         "distinct_minute_seconds": row[4], "unknown": row[0] == "__unknown__",
                         "zeroFlow": row[1] == 0 and row[2] == 0} for row in rows]
                    if active == 250 and name.endswith("30d"):
                        kind = name.split("-")[0]
                        require([list(row) for row in rows] == contract["templates"]["residential_" + kind + "_rank_oracle"]["historical_rows"],
                                "A250 rank 与旧同输入语义结果不同")
                    save()
            connection.execute("commit")
        record["status"] = "PASS_SQL_ORACLE"
    except Exception as error:
        record.update({"status": "FAIL", "error": repr(error)})
        raise
    finally:
        record["database_sha256_after"] = sha(db)
        record["marker_sha256_after"] = sha(marker) if marker.is_file() else None
        record["sidecars_after"] = wal_state(db)
        record["database_hash_unchanged"] = record.get("database_sha256_before") == record["database_sha256_after"]
        record["marker_hash_unchanged"] = record.get("marker_sha256_before") == record["marker_sha256_after"]
        record["completed_utc"] = utc()
        if not record["database_hash_unchanged"] or not record["marker_hash_unchanged"] or record["sidecars_after"]["-wal"]["bytes"]:
            record["status"] = "FAIL"
            record["identity_or_wal_failure"] = True
        save()
    require(record["status"] == "PASS_SQL_ORACLE", "oracle 后身份或 WAL 不匹配")
    return record


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for active in (50, 250):
        for suffix in ("db", "db-sha256", "marker-sha256"):
            parser.add_argument(f"--a{active}-{suffix}", required=True)
    parser.add_argument("--output-dir", required=True)
    args = parser.parse_args()
    output = Path(args.output_dir)
    require(output.is_absolute() and output.resolve().is_relative_to(HERE) and output.resolve() != HERE,
            "输出必须是本 oracle 根内的新子目录")
    require(not output.exists(), "输出已存在；不得覆盖或接续旧结果")
    output.mkdir(parents=True)
    contract_path = HERE / "sql-contract.json"
    contract = json.loads(contract_path.read_text(encoding="utf-8"))
    summary = {"started_utc": utc(), "status": "RUNNING", "argv": sys.argv,
               "script_sha256": sha(__file__), "sql_contract_sha256": sha(contract_path),
               "native_rank_comparison": "NOT_RUN；本脚本只准备完整独立 SQL 期望，不启动 monitor-db", "corpora": []}
    exit_code = 2
    try:
        for active in (50, 250):
            values = vars(args)
            record = verify_corpus(active, values[f"a{active}_db"], values[f"a{active}_db_sha256"],
                                   values[f"a{active}_marker_sha256"], contract, output)
            summary["corpora"].append({"active": active, "status": record["status"], "receipt": f"a{active}.json"})
        summary["status"] = "PASS_SQL_ORACLE_NATIVE_NOT_RUN"
        exit_code = 0
    except Exception as error:
        summary.update({"status": "FAIL_OR_BLOCKED", "error": repr(error)})
    finally:
        summary.update({"completed_utc": utc(), "driver_intended_exit_code": exit_code})
        (output / "summary.json").write_text(json.dumps(summary, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return exit_code


if __name__ == "__main__":
    sys.exit(main())

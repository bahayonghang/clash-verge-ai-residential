"""Exercise report validation without invoking any benchmark workload."""
import ast
import copy
import math
from pathlib import Path
import unittest

runner = Path(__file__).with_name("run-formal-replay.py")
tree = ast.parse(runner.read_text(encoding="utf-8"))
functions = ast.Module(body=[node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name in ["ratio", "number", "validate_report"]], type_ignores=[])
scope = {"math": math}
exec(compile(functions, str(runner), "exec"), scope)
ratio = scope["ratio"]
validate = scope["validate_report"]

class ValidationTests(unittest.TestCase):
    def setUp(self):
        self.expected = {"active": 250, "hz": 1, "duration_secs": 30, "warmup_secs": 5, "workload": "counters", "archive": "complete", "metadata_change_percent": 100, "query_every_frames": 5, "seed": 20260919, "start_utc": 1800001800, "period_rule": False, "virtual_time": False, "source_revision": "source-hash", "dir": "C:\\isolated\\case"}
        self.identity = {"sha256": "AB" * 32}
        self.data = {"options": dict(self.expected), "schema_version": 1, "kind": "isolated-real-facade", "platform": "windows", "process_id": 42, "frames": 30, "commits": 30, "synchronous": "FULL", "traffic": {"conserved": True}, "executable_sha256": "ab" * 32, "fixture_hash": "cd" * 32, "local_utc_offset_seconds": -18000, "initial_files_and_pages": {"journal_mode": "wal", "synchronous": 2}, "final_files_and_pages": {"journal_mode": "wal", "synchronous": 2}, "native_private_bytes": {"samples": 30, "p95": 1000, "max": 1001}, "native_working_set_bytes": {"samples": 30, "p95": 1200, "max": 1201}, "samples": [{} for _ in range(30)], "native_cpu_seconds": 0.5, "sqlite_application_file_write_bytes": 500, "measured_wall_secs": 30.1, "warmup_wall_secs": 5.0, "all_application_file_write_bytes": None, "spool_write_bytes": None, "latency": {}}
        for field, count in [("facade_ingest_including_durable_commit", 30), ("report_first_reader", 6), ("report_repeated_reader", 6)]:
            self.data["latency"][field] = {"count": count, "p50_ms": 1.0, "p95_ms": 2.0, "p99_ms": 3.0, "max_ms": 4.0}

    def check(self, data):
        return validate(data, self.expected, self.identity, 42)

    def test_complete_matrix_report_and_primary_counts(self):
        self.assertEqual(self.check(self.data), [])
        self.expected["duration_secs"] = 300
        self.expected["warmup_secs"] = 30
        self.expected["query_every_frames"] = 0
        self.data["options"] = dict(self.expected)
        for field in ["frames", "commits"]:
            self.data[field] = 300
        for field in ["native_private_bytes", "native_working_set_bytes"]:
            self.data[field]["samples"] = 300
        self.data["samples"] = [{} for _ in range(300)]
        self.data["latency"]["facade_ingest_including_durable_commit"]["count"] = 300
        for field in ["report_first_reader", "report_repeated_reader"]:
            self.data["latency"][field]["count"] = 0
        self.assertEqual(self.check(self.data), [])

    def test_invalid_report_shapes_and_required_fields(self):
        for value in [None, [], 1, True, "report", {}]:
            with self.subTest(value=value):
                self.assertTrue(self.check(value))
        for key in self.data:
            value = copy.deepcopy(self.data)
            del value[key]
            with self.subTest(missing=key):
                self.assertTrue(self.check(value))

    def test_zero_samples_and_wrong_identity_fail(self):
        for keys, bad in [(("latency", "report_first_reader", "count"), 0), (("latency", "report_repeated_reader", "count"), True), (("native_private_bytes", "samples"), 0), (("options", "source_revision"), "other"), (("options", "dir"), "elsewhere"), (("options", "virtual_time"), True), (("schema_version",), True), (("platform",), "linux"), (("initial_files_and_pages", "journal_mode"), "delete"), (("final_files_and_pages", "synchronous"), 1), (("process_id",), 0), (("traffic", "conserved"), 1)]:
            value = copy.deepcopy(self.data)
            target = value
            for key in keys[:-1]:
                target = target[key]
            target[keys[-1]] = bad
            with self.subTest(keys=keys, bad=bad):
                self.assertTrue(self.check(value))

    def test_invalid_native_values_do_not_enter_sums(self):
        for field in ["native_cpu_seconds", "sqlite_application_file_write_bytes", "all_application_file_write_bytes"]:
            for bad in [True, -1, float("inf"), float("nan"), "1"]:
                value = copy.deepcopy(self.data)
                value[field] = bad
                with self.subTest(field=field, bad=bad):
                    self.assertTrue(self.check(value))

    def test_ratios_reject_invalid_denominators_and_candidates(self):
        self.assertEqual(ratio(10, 7), 0.7)
        self.assertEqual(ratio(10, 0), 0.0)
        for before, after in [(0, 0), (True, 1), (1, False), (-1, 1), (1, -1), (float("inf"), 1), (1, float("inf")), (1, float("nan")), (None, 1), (1, None)]:
            with self.subTest(before=before, after=after):
                self.assertIsNone(ratio(before, after))

if __name__ == "__main__":
    unittest.main(verbosity=2)

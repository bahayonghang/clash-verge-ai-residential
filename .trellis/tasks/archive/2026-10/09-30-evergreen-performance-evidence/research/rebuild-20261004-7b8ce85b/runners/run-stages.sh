#!/usr/bin/env bash
# 顺序运行 matrix → primary → capacity → retention → heap-diagnostics。
# 每阶段保留 stdout/stderr 与 driver receipt；某阶段非零退出不阻止后续阶段（门失败为 2，UNVERIFIED 为 3）。
set -u
E="$(cd "$(dirname "$0")/.." && pwd)"
stage() {
  local name="$1"; shift
  local started; started="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  python -B "$@" > "$E/$name-driver.stdout.log" 2> "$E/$name-driver.stderr.log"
  local code=$?
  printf '{"set":"%s","started_utc":"%s","ended_utc":"%s","driver_exit":%d}\n' "$name" "$started" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$code" > "$E/$name-driver.receipt.json"
}
stage matrix "$E/runners/run-formal-replay.py" --set matrix
stage primary "$E/runners/run-formal-replay.py" --set primary
stage capacity "$E/runners/run-capacity.py"
stage retention "$E/runners/run-retention.py"
stage heap-diagnostics "$E/runners/run-heap-diagnostics.py"

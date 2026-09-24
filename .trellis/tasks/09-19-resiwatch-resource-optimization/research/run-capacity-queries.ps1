param(
    [Parameter(Mandatory)][string]$DbCli,
    [Parameter(Mandatory)][string]$DataRoot,
    [Parameter(Mandatory)][string]$EvidenceRoot
)

$ErrorActionPreference = 'Stop'
# 查询超时/不支持是要保留的实验结果，由下方 exit_code 显式记录。
$PSNativeCommandUseErrorActionPreference = $false
foreach ($active in 50, 250, 1000) {
    $fixture = Join-Path $DataRoot "corpus-a$active-30d"
    $manifest = Get-Content -Raw -LiteralPath (Join-Path $fixture 'production-corpus.json') | ConvertFrom-Json
    foreach ($window in '30d', '1d') {
        $startUtc = if ($window -eq '30d') { $manifest.start_utc } else { $manifest.end_utc - 86400 }
        $failures = 0
        foreach ($round in 0..20) {
            $label = "query-a$active-$window-$round"
            $output = Join-Path $EvidenceRoot "$label.json"
            if (Test-Path -LiteralPath $output) { throw "证据已存在：$output" }
            $timer = [System.Diagnostics.Stopwatch]::StartNew()
            & $DbCli --db (Join-Path $fixture 'monitor.sqlite3') --since $startUtc --until $manifest.end_utc --tz UTC rank --by host --top 20 1> $output 2> (Join-Path $EvidenceRoot "$label.stderr.txt")
            $queryExit = $LASTEXITCODE
            $timer.Stop()
            [pscustomobject]@{name=$label; exit_code=$queryExit; wall_ms=$timer.Elapsed.TotalMilliseconds; first_reader=($round -eq 0); start_utc=$startUtc; end_utc=$manifest.end_utc} |
                ConvertTo-Json -Compress | Tee-Object -FilePath (Join-Path $EvidenceRoot 'capacity-query-index.jsonl') -Append
            if ($queryExit -ne 0) { $failures++ }
            # 已确认失败时不重复20次同一超时；仍记录并继续其它规模/窗口。
            if ($failures -ge 2) { break }
        }
    }
    & $DbCli --db (Join-Path $fixture 'monitor.sqlite3') --since $manifest.start_utc --until $manifest.end_utc --tz UTC rank --by network --top 100 1> (Join-Path $EvidenceRoot "network-a$active-30d.json") 2> (Join-Path $EvidenceRoot "network-a$active-30d.stderr.txt")
    $queryExit = $LASTEXITCODE
    $count = if ($queryExit -eq 0) {
        $report = Get-Content -Raw -LiteralPath (Join-Path $EvidenceRoot "network-a$active-30d.json") | ConvertFrom-Json
        ($report.result.rankings | Measure-Object -Property connectionCount -Sum).Sum
    } else { $null }
    [pscustomobject]@{name="network-a$active-30d"; exit_code=$queryExit; connection_count=$count; expected_connection_count=$manifest.actual.sessions * 3 / 4} |
        ConvertTo-Json -Compress | Tee-Object -FilePath (Join-Path $EvidenceRoot 'capacity-query-index.jsonl') -Append
}

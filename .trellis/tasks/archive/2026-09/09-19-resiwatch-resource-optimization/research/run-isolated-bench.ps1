param(
    [Parameter(Mandatory)][ValidateSet('smoke', 'probe', 'primary', 'matrix', 'peak', 'capacity')][string]$RunSet,
    [Parameter(Mandatory)][string]$BaselineExe,
    [Parameter(Mandatory)][string]$CandidateExe,
    [Parameter(Mandatory)][string]$CandidateSource,
    [Parameter(Mandatory)][string]$DataRoot,
    [Parameter(Mandatory)][string]$EvidenceRoot
)

# 每组实验串行执行；运行前停止本任务的编译、测试和其它容量生成。
# 工具本身拒绝非空数据库目录。证据位于数据库目录之外，失败保留现场。
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $DataRoot, $EvidenceRoot | Out-Null
$baselineSource = 'c278bb7b56603001e32e353d2ee589dccef0bfe9'

function Invoke-Replay {
    param([string]$Name, [int]$Active, [string]$Workload, [string]$Archive,
          [int]$Duration, [int]$Warmup, [int]$Queries = 0, [switch]$Virtual,
          [switch]$PeriodRule)
    $variants = @('baseline', 'candidate')
    foreach ($variant in $variants) {
        $label = "$Name-$variant"
        $exe = if ($variant -eq 'baseline') { $BaselineExe } else { $CandidateExe }
        $source = if ($variant -eq 'baseline') { $baselineSource } else { $CandidateSource }
        $arguments = @('replay-facade', '--active', "$Active", '--hz', '1',
            '--duration-secs', "$Duration", '--warmup-secs', "$Warmup", '--workload', $Workload,
            '--metadata-change-percent', '100', '--archive', $Archive,
            '--query-every-frames', "$Queries", '--source-revision', $source,
            '--dir', (Join-Path $DataRoot $label))
        if ($Virtual) { $arguments += '--virtual-time' }
        if ($PeriodRule) { $arguments += '--period-rule' }
        $output = Join-Path $EvidenceRoot "$label.json"
        if (Test-Path -LiteralPath $output) { throw "证据已存在：$output" }
        Write-Output "$([DateTime]::UtcNow.ToString('o')) START $label"
        & $exe @arguments 1> $output 2> (Join-Path $EvidenceRoot "$label.stderr.txt")
        if ($LASTEXITCODE -ne 0) { throw "$label 失败，exit=$LASTEXITCODE" }
        $result = Get-Content -Raw -LiteralPath $output | ConvertFrom-Json
        [pscustomobject]@{
            name = $label; finished_utc = [DateTime]::UtcNow.ToString('o')
            fixture_hash = $result.fixture_hash; executable_sha256 = $result.executable_sha256
            wall_seconds = $result.measured_wall_secs; cpu_seconds = $result.native_cpu_seconds
            sqlite_xwrite_bytes = $result.sqlite_application_file_write_bytes
            private_p95 = $result.native_private_bytes.p95
            ingest_p95_ms = $result.latency.facade_ingest_including_durable_commit.p95_ms
            ingest_max_ms = $result.latency.facade_ingest_including_durable_commit.max_ms
            report_p95_ms = $result.latency.report_first_reader.p95_ms
            conserved = $result.traffic.conserved
        } | ConvertTo-Json -Compress | Tee-Object -FilePath (Join-Path $EvidenceRoot "$RunSet-index.jsonl") -Append
    }
}

switch ($RunSet) {
    'smoke' {
        Invoke-Replay -Name 'smoke' -Active 8 -Workload counters -Archive complete -Duration 3 -Warmup 0 -Virtual
    }
    'probe' {
        Invoke-Replay -Name 'probe' -Active 250 -Workload counters -Archive complete -Duration 300 -Warmup 30 -Virtual
    }
    'primary' {
        foreach ($round in 1..3) {
            Invoke-Replay -Name "primary-r$round" -Active 250 -Workload counters -Archive complete -Duration 300 -Warmup 30
        }
    }
    'matrix' {
        foreach ($active in 50, 250, 1000) {
            foreach ($workload in 'unchanged', 'counters', 'metadata') {
                Invoke-Replay -Name "matrix-a$active-$workload" -Active $active -Workload $workload -Archive complete -Duration 30 -Warmup 5 -Queries 5
            }
        }
        foreach ($archive in 'backlog', 'failed') {
            Invoke-Replay -Name "matrix-a250-$archive" -Active 250 -Workload metadata -Archive $archive -Duration 30 -Warmup 5 -Queries 5 -PeriodRule
        }
    }
    'peak' {
        Invoke-Replay -Name 'peak-a10000' -Active 10000 -Workload counters -Archive complete -Duration 1800 -Warmup 30
    }
    'capacity' {
        $endUtc = [long][Math]::Floor([DateTimeOffset]::UtcNow.ToUnixTimeSeconds() / 86400) * 86400
        $startUtc = $endUtc - 30 * 86400
        foreach ($active in 50, 250, 1000) {
            $label = "corpus-a$active-30d"
            $output = Join-Path $EvidenceRoot "$label.json"
            if (Test-Path -LiteralPath $output) { throw "证据已存在：$output" }
            Write-Output "$([DateTime]::UtcNow.ToString('o')) START $label"
            & $CandidateExe generate-corpus --average-active $active --days 30 --start-utc $startUtc --dir (Join-Path $DataRoot $label) 1> $output 2> (Join-Path $EvidenceRoot "$label.stderr.txt")
            if ($LASTEXITCODE -ne 0) { throw "$label 失败，exit=$LASTEXITCODE" }
            $result = Get-Content -Raw -LiteralPath $output | ConvertFrom-Json
            [pscustomobject]@{name=$label; wall_seconds=$result.generation_wall_secs; counts_match=$result.counts_match; rows=$result.actual; pages=$result.files_and_pages} |
                ConvertTo-Json -Depth 8 -Compress | Tee-Object -FilePath (Join-Path $EvidenceRoot 'capacity-index.jsonl') -Append
        }
    }
}

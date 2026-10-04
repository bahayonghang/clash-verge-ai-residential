param(
    [Parameter(Mandatory)][string]$Fixture,
    [Parameter(Mandatory)][string]$Output,
    [switch]$SeriesOnly
)

$ErrorActionPreference = 'Stop'
$manifest = Get-Content -LiteralPath (Join-Path $Fixture 'production-corpus.json') -Raw | ConvertFrom-Json
if ($manifest.kind -ne 'production-corpus') { throw '只允许显式标记的生成库' }
if (Test-Path -LiteralPath $Output) { throw "证据已存在：$Output" }
$outputParent = Split-Path -Parent $Output
New-Item -ItemType Directory -Path $outputParent -Force | Out-Null
$env:RESIWATCH_SQL_STAGE_DB = Join-Path $Fixture 'monitor.sqlite3'
$env:RESIWATCH_SQL_STAGE_OUT = [System.IO.Path]::GetFullPath($Output)
$env:RESIWATCH_SQL_STAGE_START = [string]($manifest.start_utc + 86400)
$env:RESIWATCH_SQL_STAGE_END = [string]$manifest.end_utc
$env:RESIWATCH_SQL_STAGE_SERIES_ONLY = if ($SeriesOnly) { '1' } else { '0' }
$hashes = @('residential-monitor/src-tauri/src/c3/service.rs', 'residential-monitor/src-tauri/src/c3/sql.rs') | ForEach-Object {
    [pscustomobject]@{ path = $_; sha256 = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash }
}
$env:RESIWATCH_SQL_STAGE_SOURCE = $hashes | ConvertTo-Json -Compress
& rtk proxy cargo test --release --manifest-path residential-monitor/src-tauri/Cargo.toml --lib isolated_raw_stage_probe -- --ignored --nocapture
if ($LASTEXITCODE -ne 0) { throw "SQL阶段探针失败：exit=$LASTEXITCODE" }

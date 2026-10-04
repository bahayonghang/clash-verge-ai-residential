$ErrorActionPreference = "Stop"
$PSNativeCommandUseErrorActionPreference = $false
$researchRoot = $PSScriptRoot
$prefix = Join-Path $researchRoot "formal-primary-recovery-20261001"
$preflight = Get-Content -LiteralPath ($prefix + ".preflight-final.json") -Raw | ConvertFrom-Json
if ($preflight.status -ne "PASS") { throw "启动前检查未通过" }
foreach ($path in $preflight.fresh_paths) {
  if (Test-Path -LiteralPath $path) { throw ("证据路径已存在：" + $path) }
}
$scriptHash = (Get-FileHash -LiteralPath $PSCommandPath -Algorithm SHA256).Hash
if ($scriptHash -ne $preflight.outer_script_sha256) { throw "外层脚本身份不匹配" }
$started = [DateTime]::UtcNow
$outerReceipt = [ordered]@{
  status = "running"
  powershell_pid = $PID
  started_utc = $started.ToString("o")
  command = $preflight.outer_command
  wrapper_argv = $preflight.wrapper_argv
  observed_outer_exit_code = $null
}
$outerReceipt | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath ($prefix + ".outer.json") -Encoding utf8
$pythonExe = $preflight.python_executable
$wrapperArgs = @($preflight.wrapper_argv | Select-Object -Skip 1)
& $pythonExe @wrapperArgs 2> ($prefix + ".wrapper.stderr.log") | Tee-Object -FilePath ($prefix + ".wrapper.stdout.log")
$wrapperExit = $LASTEXITCODE
$finished = [DateTime]::UtcNow
$outerReceipt.status = "wrapper_finished_outer_completion_pending"
$outerReceipt.finished_utc = $finished.ToString("o")
$outerReceipt.wall_seconds = ($finished - $started).TotalSeconds
$outerReceipt.wrapper_exit_code = $wrapperExit
$outerReceipt.intended_outer_exit_code = $wrapperExit
$outerReceipt | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath ($prefix + ".outer.json") -Encoding utf8
exit $wrapperExit

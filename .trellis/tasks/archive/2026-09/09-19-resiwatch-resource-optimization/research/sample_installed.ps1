param([int]$ProcessId = 25188, [int]$DurationSeconds = 30)
$ErrorActionPreference = 'Stop'
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class ResiWatchReadOnlyIo {
    [StructLayout(LayoutKind.Sequential)] public struct Counters {
        public ulong ReadOperations, WriteOperations, OtherOperations;
        public ulong ReadBytes, WriteBytes, OtherBytes;
    }
    [DllImport("kernel32.dll", SetLastError=true)] public static extern IntPtr OpenProcess(uint access, bool inherit, uint pid);
    [DllImport("kernel32.dll", SetLastError=true)] public static extern bool GetProcessIoCounters(IntPtr handle, out Counters counters);
    [DllImport("kernel32.dll")] public static extern bool CloseHandle(IntPtr handle);
}
'@
$observedProcess = Get-Process -Id $ProcessId
$dataDirectory = Join-Path (Split-Path -Parent $observedProcess.Path) 'data'
$processHandle = [ResiWatchReadOnlyIo]::OpenProcess(0x1000, $false, $ProcessId)
$samples = @()
try {
    for ($index = 0; $index -le $DurationSeconds; $index += 2) {
        $observedProcess.Refresh()
        $io = New-Object ResiWatchReadOnlyIo+Counters
        $ioOk = [ResiWatchReadOnlyIo]::GetProcessIoCounters($processHandle, [ref]$io)
        $spool = @(Get-ChildItem -LiteralPath (Join-Path $dataDirectory 'report-spool') -File)
        $newestSpool = $spool | Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1
        $samples += [pscustomobject]@{
            at_utc = (Get-Date).ToUniversalTime().ToString('o')
            cpu_seconds = $observedProcess.CPU
            working_set_bytes = $observedProcess.WorkingSet64
            private_bytes = $observedProcess.PrivateMemorySize64
            handles = $observedProcess.HandleCount
            read_operations = $(if ($ioOk) { $io.ReadOperations } else { $null })
            write_operations = $(if ($ioOk) { $io.WriteOperations } else { $null })
            io_read_bytes = $(if ($ioOk) { $io.ReadBytes } else { $null })
            io_write_bytes = $(if ($ioOk) { $io.WriteBytes } else { $null })
            db_bytes = (Get-Item -LiteralPath (Join-Path $dataDirectory 'monitor.sqlite3')).Length
            wal_bytes = (Get-Item -LiteralPath (Join-Path $dataDirectory 'monitor.sqlite3-wal')).Length
            spool_file_count = $spool.Count
            spool_bytes = ($spool | Measure-Object -Property Length -Sum).Sum
            spool_newest_write_utc = $(if ($newestSpool) { $newestSpool.LastWriteTimeUtc.ToString('o') } else { $null })
        }
        if ($index -lt $DurationSeconds) { Start-Sleep -Seconds 2 }
    }
} finally {
    if ($processHandle -ne [IntPtr]::Zero) { [void][ResiWatchReadOnlyIo]::CloseHandle($processHandle) }
}
$first = $samples[0]
$last = $samples[-1]
$elapsed = ([datetime]$last.at_utc - [datetime]$first.at_utc).TotalSeconds
[pscustomobject]@{
    pid = $ProcessId
    elapsed_seconds = $elapsed
    cpu_delta_seconds = $last.cpu_seconds - $first.cpu_seconds
    one_core_cpu_percent = 100 * ($last.cpu_seconds - $first.cpu_seconds) / $elapsed
    io_write_bytes_delta = $last.io_write_bytes - $first.io_write_bytes
    io_write_operations_delta = $last.write_operations - $first.write_operations
    scope = 'Native parent process only. I/O counters include file/device/network I/O; not physical disk bytes. Spool metadata polling may miss intermediate writes. Window state and workload uncontrolled.'
    samples = $samples
} | ConvertTo-Json -Depth 4

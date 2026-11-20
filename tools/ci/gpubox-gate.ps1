# Runs before each heavy step of CI on our own runner (gpubox, decision 0012):
# the brush checks yield to everything else on that PC.
#
#   -Lower  once per job, first: this job's Runner.Worker drops to the Idle
#           priority class, so every later step (cargo, rustc, the tests)
#           starts at Idle and gets only the CPU nobody else wants - below
#           gpubox's GPU queue (below normal), the desktop's CI (normal) and
#           the owner's own programs.
#
# Every call then waits while any of these holds, for up to -Minutes:
#   - gpubox is booked: <Tests>\.reserved says "NNN until HH:mm" and that time
#     hasn't passed (a timing run has the whole PC);
#   - another Actions job runs on this PC (another Runner.Worker.exe; on
#     gpubox, the desktop's runner, whose Checks time the app against budgets);
#   - an agent's test run holds gpubox's test lock (<Tests>\.run.lock, kept
#     open without sharing while it runs);
#   - less than -FreeMB of memory is free.
# Off gpubox the two files don't exist, so only the last two apply.
# Nothing here, or anywhere in CI, uses the GPU.
#
#   -Check  print what would make it wait, then exit 0 (nothing lowered or waited)

param(
    [switch]$Lower,
    [switch]$Check,
    [int]$Minutes = 60,
    [int]$FreeMB = 2048,
    [string]$Tests = 'D:\polyglotic\tests'
)
$ErrorActionPreference = 'Stop'

function Get-Worker {
    # This job's Runner.Worker.exe: the nearest one among this process's parents.
    $id = $PID
    for ($i = 0; $i -lt 12 -and $id; $i++) {
        $p = Get-CimInstance Win32_Process -Filter "ProcessId=$id"
        if (-not $p) { return $null }
        if ($p.Name -eq 'Runner.Worker.exe') { return [int]$p.ProcessId }
        $id = $p.ParentProcessId
    }
    return $null
}

function Get-Busy([object]$worker, [datetime]$now) {
    $busy = @()

    $reserved = Join-Path $Tests '.reserved'
    if (Test-Path $reserved) {
        $booked = Get-Content $reserved -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($booked -match '^\s*(\d{3,4}) until (\d{1,2}):(\d{2})\s*$') {
            $until = $now.Date.AddHours([int]$Matches[2]).AddMinutes([int]$Matches[3])
            if ($now -lt $until) { $busy += "gpubox is booked ($($booked.Trim()))" }
        }
    }

    $others = @(Get-CimInstance Win32_Process -Filter "Name='Runner.Worker.exe'" |
        Where-Object { [int]$_.ProcessId -ne $worker })
    if ($others.Count) { $busy += "another Actions job is running here ($($others.Count))" }

    $lock = Join-Path $Tests '.run.lock'
    if (Test-Path $lock) {
        try { [IO.File]::Open($lock, 'Open', 'Read', 'ReadWrite').Dispose() }
        catch [IO.IOException] {
            $who = Get-Content (Join-Path $Tests '.run.who') -ErrorAction SilentlyContinue | Select-Object -First 1
            $busy += "an agent's test run holds gpubox's test lock ($who)"
        }
        catch { } # not readable by this account: nothing to learn from it
    }

    $free = [int]((Get-CimInstance Win32_OperatingSystem).FreePhysicalMemory / 1KB)
    if ($free -lt $FreeMB) { $busy += "only $free MB of memory free (waits for $FreeMB)" }
    return $busy
}

$worker = Get-Worker
if ($Check) {
    $busy = @(Get-Busy $worker (Get-Date))
    if ($busy.Count) { $busy | ForEach-Object { Write-Host "would wait: $_" } } else { Write-Host 'clear' }
    exit 0
}

if ($Lower) {
    if ($worker) {
        (Get-Process -Id $worker).PriorityClass = 'Idle'
        Write-Host "This job's steps run at Idle priority (Runner.Worker $worker)."
    } else {
        Write-Host '::warning::No Runner.Worker among this step''s parents: priority left as it is.'
    }
}

$start = Get-Date
$said = ''
while ($true) {
    $busy = @(Get-Busy $worker (Get-Date))
    if (-not $busy.Count) { break }
    $now = ($busy -join '; ')
    if ($now -ne $said) { Write-Host "Waiting: $now"; $said = $now }
    if (((Get-Date) - $start).TotalMinutes -ge $Minutes) {
        Write-Host "::error::gpubox stayed busy for $Minutes minutes ($now). Re-run the job when it is free."
        exit 1
    }
    Start-Sleep -Seconds 20
}
$waited = [int]((Get-Date) - $start).TotalSeconds
if ($waited -ge 20) { Write-Host "::notice::Waited $waited s for gpubox ($said)." } else { Write-Host 'gpubox is clear.' }

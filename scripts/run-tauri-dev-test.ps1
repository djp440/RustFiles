$ErrorActionPreference = "Stop"

Write-Host "Starting Tauri dev smoke test..." -ForegroundColor Cyan

$logFile = Join-Path -Path $PSScriptRoot -ChildPath "..\tauri-dev.log"
$errLogFile = Join-Path -Path $PSScriptRoot -ChildPath "..\tauri-dev-err.log"

$projectDir = Resolve-Path (Join-Path -Path $PSScriptRoot -ChildPath "..")

$process = Start-Process -FilePath "npx.cmd" -ArgumentList "tauri", "dev" -NoNewWindow -PassThru -RedirectStandardOutput $logFile -RedirectStandardError $errLogFile -WorkingDirectory $projectDir

Write-Host "Tauri dev process started with PID: $($process.Id)" -ForegroundColor Cyan

Start-Sleep -Seconds 15

if ($process.HasExited) {
    $exitCode = $process.ExitCode
    Write-Host "Tauri dev process exited with code $exitCode" -ForegroundColor Red
    Write-Host "--- stdout ---" -ForegroundColor Yellow
    Get-Content $logFile -ErrorAction SilentlyContinue
    Write-Host "--- stderr ---" -ForegroundColor Yellow
    Get-Content $errLogFile -ErrorAction SilentlyContinue
    exit 1
}

taskkill /PID $process.Id /T /F | Out-Null
Start-Sleep -Seconds 2

$lingeringProcesses = Get-CimInstance Win32_Process | Where-Object {
    $_.CommandLine -like '*RustFiles*' -and (
        $_.Name -eq 'rustfiles.exe' -or
        $_.Name -eq 'node.exe' -or
        $_.Name -eq 'npx.cmd'
    )
}

if ($lingeringProcesses) {
    Write-Host "Smoke test left lingering processes:" -ForegroundColor Red
    $lingeringProcesses | Select-Object ProcessId, Name, CommandLine | Format-List | Out-String | Write-Host
    exit 1
}

Write-Host "Tauri dev smoke test completed successfully" -ForegroundColor Green
exit 0

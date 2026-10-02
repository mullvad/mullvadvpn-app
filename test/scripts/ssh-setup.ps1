# Windows counterpart of ssh-setup.sh. Installs the test runner from the directory containing this
# script and registers it as a scheduled task.

param(
    [Parameter(Mandatory)] [string]$RunnerDir,
    [Parameter(Mandatory)] [string]$AppPackage,
    [string]$PreviousApp,
    [string]$UiRunner
)

$ErrorActionPreference = 'Stop'

$TaskName = 'Mullvad Test Runner'
# How long to wait for the test runner task to start
$StartTimeoutSeconds = 60

# Stop any runner that is already running so that its files can be replaced
Stop-ScheduledTask -TaskName $TaskName -ErrorAction SilentlyContinue

# Copy over test runner to correct place

Write-Output "Copying test-runner to $RunnerDir"

New-Item -ItemType Directory -Force -Path $RunnerDir | Out-Null

foreach ($file in @('test-runner.exe', 'connection-checker.exe', $AppPackage, $PreviousApp, $UiRunner)) {
    if (-not $file) {
        continue
    }
    Write-Output "Moving $PSScriptRoot\$file to $RunnerDir"
    Copy-Item -Force -Path (Join-Path $PSScriptRoot $file) -Destination $RunnerDir
}

# Windows Defender occasionally kills the test runner because it believes it to be a trojan
try {
    Add-MpPreference -ExclusionPath $RunnerDir
} catch {
    Write-Warning "Failed to add Defender exclusion: $_"
}

# Create task
#
# The runner is started through a scheduled task rather than directly over SSH. A process started
# from the SSH session would run in session 0, without access to the desktop that the GUI tests
# need, and would be killed when the SSH session ends.

Write-Output "Creating test runner task '$TaskName'"

$action = New-ScheduledTaskAction `
    -Execute (Join-Path $RunnerDir 'test-runner.exe') `
    -Argument '\\.\COM1 serve'
$trigger = New-ScheduledTaskTrigger -AtLogOn -User $env:USERNAME
$principal = New-ScheduledTaskPrincipal `
    -UserId $env:USERNAME `
    -LogonType Interactive `
    -RunLevel Highest
$settings = New-ScheduledTaskSettingsSet `
    -AllowStartIfOnBatteries `
    -DontStopIfGoingOnBatteries `
    -ExecutionTimeLimit ([TimeSpan]::Zero) `
    -StartWhenAvailable `
    -RestartInterval (New-TimeSpan -Minutes 1) `
    -RestartCount 999

Register-ScheduledTask `
    -TaskName $TaskName `
    -Action $action `
    -Trigger $trigger `
    -Principal $principal `
    -Settings $settings `
    -Force | Out-Null

Write-Output "Starting test runner task"

# This does nothing if the user is not logged on yet. In that case, the logon trigger starts it.
Start-ScheduledTask -TaskName $TaskName

$deadline = (Get-Date).AddSeconds($StartTimeoutSeconds)
while ((Get-ScheduledTask -TaskName $TaskName).State -ne 'Running') {
    if ((Get-Date) -gt $deadline) {
        throw "Test runner task did not start within $StartTimeoutSeconds seconds"
    }
    Start-Sleep -Seconds 1
}

Write-Output "Test runner is running"

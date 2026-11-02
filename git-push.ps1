# Git Push with Telegram Notification
# Usage: .\git-push.ps1 [git push arguments]

param(
    [Parameter(ValueFromRemainingArguments=$true)]
    [string[]]$GitArgs
)

# Update PATH
$env:PATH = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

Write-Host "Pushing to GitHub..." -ForegroundColor Cyan

# Execute git push
$pushCommand = "git push $($GitArgs -join ' ')"
Invoke-Expression $pushCommand

if ($LASTEXITCODE -eq 0) {
    Write-Host "Push successful!" -ForegroundColor Green
    Write-Host "Sending notification to Telegram..." -ForegroundColor Cyan

    # Run notification script
    & "$PSScriptRoot\notify-telegram.ps1"
} else {
    Write-Host "Push failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}

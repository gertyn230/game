# Telegram notification script for git push
# Usage: Run this after git push

$token = "9113723998:AAEHW2xz6WXxDFj9HzbIodGy01-4Xq4SAvg"
$chatId = "@pixel_village"

# Update PATH
$env:PATH = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

# Get last commit info
$commitHash = git log -1 --pretty=%h
$commitMessage = git log -1 --pretty=%B | Select-Object -First 1
$commitAuthor = git log -1 --pretty=%an
$commitUrl = "https://github.com/gertyn230/game/commit/$commitHash"

# Create message
$message = @"
**New Update / Нове оновлення**

**EN:**
Commit: $commitMessage
Author: $commitAuthor
View changes: $commitUrl

**UA:**
Коміт: $commitMessage
Автор: $commitAuthor
Переглянути зміни: $commitUrl

Commit hash: $commitHash
"@

# Send to Telegram
$body = @{
    chat_id = $chatId
    text = $message
    parse_mode = "Markdown"
    disable_web_page_preview = $false
} | ConvertTo-Json -Compress

try {
    $response = Invoke-RestMethod -Uri "https://api.telegram.org/bot$token/sendMessage" -Method Post -ContentType "application/json; charset=utf-8" -Body $body
    Write-Host "Message sent to Telegram successfully!" -ForegroundColor Green
} catch {
    Write-Host "Failed to send message: $_" -ForegroundColor Red
}

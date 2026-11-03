# Telegram notification script
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$PSDefaultParameterValues['*:Encoding'] = 'utf8'

$token = "9113723998:AAEHW2xz6WXxDFj9HzbIodGy01-4Xq4SAvg"
$chatId = "@pixel_village"

$env:PATH = [System.Environment]::GetEnvironmentVariable("Path","Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path","User")

$commitMessage = git log -1 --pretty=%B | Select-Object -First 1
$commitAuthor = git log -1 --pretty=%an

# Розпарсюємо формат "EN text | UA текст"
if ($commitMessage -match '(.+?)\s*\|\s*(.+)') {
    $enMessage = $matches[1].Trim()
    $uaMessage = $matches[2].Trim()

    $messageText = "🔄 New Update / Нове оновлення`n`n" +
    "EN: $enMessage`n" +
    "UA: $uaMessage`n`n" +
    "👤 Author / Автор: $commitAuthor"
} else {
    # Якщо формат не знайдено, показуємо як є
    $messageText = "🔄 New Update / Нове оновлення`n`n" +
    "📝 $commitMessage`n`n" +
    "👤 Author / Автор: $commitAuthor"
}

$payload = @{
    chat_id = $chatId
    text = $messageText
} | ConvertTo-Json -Depth 10

$utf8Bytes = [System.Text.Encoding]::UTF8.GetBytes($payload)
Invoke-RestMethod -Uri "https://api.telegram.org/bot$token/sendMessage" -Method Post -ContentType "application/json; charset=utf-8" -Body $utf8Bytes
Write-Host "Message sent!" -ForegroundColor Green
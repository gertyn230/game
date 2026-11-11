#!/usr/bin/env pwsh
# Автоматичний деплой: зміни → GitHub → release → автооновлення гри

param(
    [string]$Message = "Автоматичне оновлення",
    [switch]$Patch,
    [switch]$Minor,
    [switch]$Major
)

Write-Host "🚀 Автоматичний деплой Pixel Village" -ForegroundColor Cyan
Write-Host ""

# Перевірка на незбережені зміни
$status = git status --porcelain
if (-not $status) {
    Write-Host "✅ Немає змін для деплою" -ForegroundColor Green
    exit 0
}

# Читання поточної версії
$cargoToml = Get-Content "client/Cargo.toml" -Raw
if ($cargoToml -match 'version = "(\d+)\.(\d+)\.(\d+)"') {
    $major_v = [int]$matches[1]
    $minor_v = [int]$matches[2]
    $patch_v = [int]$matches[3]

    $currentVersion = "$major_v.$minor_v.$patch_v"
    Write-Host "📦 Поточна версія: v$currentVersion" -ForegroundColor Yellow

    # Автоматичне збільшення patch версії (або вибір користувача)
    if ($Major) {
        $major_v++
        $minor_v = 0
        $patch_v = 0
    } elseif ($Minor) {
        $minor_v++
        $patch_v = 0
    } else {
        $patch_v++
    }

    $newVersion = "$major_v.$minor_v.$patch_v"
    Write-Host "📦 Нова версія: v$newVersion" -ForegroundColor Green

    # Оновлення версії в Cargo.toml
    $cargoToml = $cargoToml -replace 'version = "\d+\.\d+\.\d+"', "version = `"$newVersion`""
    Set-Content "client/Cargo.toml" $cargoToml -NoNewline

    Write-Host ""
    Write-Host "📝 Коміт змін..." -ForegroundColor Cyan

    # Додавання всіх змін (крім ігнорованих)
    git add .

    # Створення коміту
    $commitMessage = "$Message`n`nCo-Authored-By: Claude <noreply@anthropic.com>"
    git commit -m $commitMessage

    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Помилка при створенні коміту" -ForegroundColor Red
        exit 1
    }

    Write-Host "✅ Коміт створено" -ForegroundColor Green
    Write-Host ""

    # Створення тегу
    Write-Host "🏷️  Створення тегу v$newVersion..." -ForegroundColor Cyan
    git tag -a "v$newVersion" -m "Release v$newVersion"

    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Помилка при створенні тегу" -ForegroundColor Red
        exit 1
    }

    Write-Host "✅ Тег створено" -ForegroundColor Green
    Write-Host ""

    # Пуш в GitHub
    Write-Host "🌐 Пуш в GitHub..." -ForegroundColor Cyan
    git push origin main
    git push origin "v$newVersion"

    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Помилка при пуші в GitHub" -ForegroundColor Red
        exit 1
    }

    Write-Host ""
    Write-Host "✅ Деплой успішний!" -ForegroundColor Green
    Write-Host "🎮 GitHub Actions збирає гру..." -ForegroundColor Cyan
    Write-Host "🔗 https://github.com/gertyn230/game/actions" -ForegroundColor Blue
    Write-Host "📦 Release: https://github.com/gertyn230/game-releases/releases/tag/v$newVersion" -ForegroundColor Blue
    Write-Host ""
    Write-Host "⏳ Зачекай 3-5 хвилин поки збереться, потім запусти гру - вона оновиться автоматично!" -ForegroundColor Yellow

} else {
    Write-Host "❌ Не можу знайти версію в client/Cargo.toml" -ForegroundColor Red
    exit 1
}

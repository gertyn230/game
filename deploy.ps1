#!/usr/bin/env pwsh
# Швидкий деплой - просто запусти ./deploy.ps1

param([string]$msg = "Оновлення гри")

.\auto-deploy.ps1 -Message $msg

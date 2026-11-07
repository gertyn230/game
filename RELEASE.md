# 🚀 Інструкція з випуску нових версій

## 📋 Передумови

1. Всі зміни закомічені в git
2. Код перевірений локально (`cargo build --release -p client`)
3. Гра протестована

## 🔢 Випуск нової версії

### Крок 1: Оновити версію в Cargo.toml

Відредагуйте `client/Cargo.toml`:
```toml
[package]
version = "1.3.5"  # Змініть на нову версію
```

**Правила версіонування (Semantic Versioning):**
- Патч (1.3.4 → 1.3.5): Виправлення багів, малі покращення
- Мінор (1.3.5 → 1.4.0): Нові функції без breaking changes
- Мажор (1.4.0 → 2.0.0): Breaking changes в API/функціональності

### Крок 2: Створити коміт і тег

```powershell
git add client/Cargo.toml
git commit -m "Bump version to v1.3.5"
git tag v1.3.5
git push origin main
git push origin v1.3.5
```

### Крок 3: GitHub Actions автоматично створить реліз

GitHub Actions:
1. ✅ Зберe Windows клієнт (`PixelVillage.exe`)
2. ✅ Згенерує SHA256 контрольну суму (`PixelVillage.exe.sha256`)
3. ✅ Зберe Android APK (`PixelVillage-android.apk`)
4. ✅ Створить GitHub Release на https://github.com/gertyn230/game/releases

**Час збірки:** ~5-10 хвилин

## 🧪 Тестування оновлення локально

### Симуляція оновлення з версії 1.3.4 → 1.3.5

1. **Зберіть версію 1.3.4:**
   ```powershell
   git checkout v1.3.4
   cargo build --release -p client
   Copy-Item target/release/client.exe C:\Temp\PixelVillage-old.exe
   ```

2. **Створіть реліз 1.3.5:**
   ```powershell
   git checkout main
   # Змініть версію в Cargo.toml на 1.3.5
   git commit -am "Bump to v1.3.5"
   git tag v1.3.5
   git push --tags
   ```

3. **Зачекайте поки GitHub Actions зберe 1.3.5**

4. **Запустіть стару версію:**
   ```powershell
   C:\Temp\PixelVillage-old.exe
   ```

5. **Перевірте що:**
   - ✅ З'являється вікно "Доступна версія 1.3.5"
   - ✅ Показується changelog
   - ✅ Кнопка "Встановити" працює
   - ✅ Завантаження показує прогрес
   - ✅ Після встановлення гра перезапускається
   - ✅ Нова версія працює коректно

## 🔧 Як працює система оновлення

### Windows

1. **Перевірка оновлень:**
   - При запуску гра звертається до GitHub API: `https://api.github.com/repos/gertyn230/game/releases/latest`
   - Порівнює версії через `semver` (1.3.4 < 1.3.5)

2. **Завантаження:**
   - Завантажує `PixelVillage.exe` у фоновому потоці
   - Показує прогрес-бар
   - Завантажує `PixelVillage.exe.sha256`
   - Перевіряє контрольну суму

3. **Встановлення (self-replace):**
   ```
   PixelVillage.exe           → PixelVillage.exe.old
   PixelVillage.exe.new       → PixelVillage.exe
   Запустити PixelVillage.exe
   Завершити поточний процес
   ```

4. **Cleanup:**
   - При наступному запуску видаляється `PixelVillage.exe.old`

### Android

На даний момент APK завантажується вручну з GitHub Releases.

**Майбутнє покращення:** Автоматичне завантаження APK + системний встановлювач через JNI.

## ⚠️ Troubleshooting

### Помилка: "SHA256 не співпадає"
- Файл пошкоджений при завантаженні
- Гра автоматично відкине оновлення
- Користувач може спробувати ще раз

### Помилка: "Помилка перейменування поточного exe"
- Windows не дозволив перейменувати файл (антивірус?)
- Виконується rollback
- Користувач може спробувати:
  - Закрити антивірус
  - Запустити від адміністратора
  - Завантажити вручну з GitHub

### GitHub Actions не створює реліз
1. Перевірте що тег має формат `v*.*.*` (наприклад `v1.3.5`)
2. Перевірте permissions в workflow (`contents: write`)
3. Подивіться логи в Actions: https://github.com/gertyn230/game/actions

### Версія в грі не змінилась після оновлення
- `CLIENT_VERSION` тепер береться з `Cargo.toml` через `env!("CARGO_PKG_VERSION")`
- Переконайтесь що ви оновили `client/Cargo.toml`, а не `Cargo.toml` в корені

## 📁 Структура файлів релізу

```
GitHub Release v1.3.5
├── PixelVillage.exe           (один файл з усіма ресурсами)
├── PixelVillage.exe.sha256    (контрольна сума)
└── PixelVillage-android.apk   (Android APK)
```

**Розмір:**
- Windows .exe: ~15-20 МБ (з вбудованими ресурсами)
- Android .apk: ~10-15 МБ

## 🎯 Що було покращено

### До (v1.3.3 і раніше):
- ❌ Popup вікна "Нажміть U"
- ❌ Кнопки для оновлення
- ❌ Папка `assets` поряд з exe
- ❌ Батник-скрипти (антивіруси блокують)
- ❌ Окремий launcher

### Після (v1.3.4+):
- ✅ Автоматична перевірка оновлень при запуску
- ✅ Вікно з кнопками "Встановити" / "Пізніше"
- ✅ Один .exe файл з усіма ресурсами
- ✅ Self-update через перейменування (безпечно)
- ✅ SHA256 перевірка цілісності
- ✅ Прогрес-бар завантаження
- ✅ GitHub Releases автоматично

## 📱 Telegram анонси

Після створення релізу опублікуйте в каналі @pixel_village:

**Українська:**
```
🎮 Pixel Village v1.3.5

✨ Що нового:
- [Опис змін]

📥 Завантажити Windows або Android можна як завжди!
```

**English:**
```
🎮 Pixel Village v1.3.5

✨ What's new:
- [Description of changes]

📥 Download Windows or Android as usual!
```

**❗ НЕ ПОСТИТИ:** Посилання на GitHub у Telegram (згідно з інструкціями)

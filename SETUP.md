# 🚀 Інструкція з налаштування та випуску релізів

## ✅ Що вже зроблено автоматично

### 1. Код повністю готовий
- ✅ Система автооновлення через latest.json
- ✅ Self-replace механізм для Windows
- ✅ SHA256 перевірка цілісності
- ✅ UpdateState enum з UI для всіх станів
- ✅ Android keystore згенеровано
- ✅ CI/CD workflow для релізів
- ✅ Launcher видалено з workspace
- ✅ Android package: com.gertyn.pixelvillage
- ✅ Server version з Cargo.toml
- ✅ Cargo.lock додано до репозиторію
- ✅ Перший коміт запушено на GitHub

### 2. Android Keystore створено
**Локація:** `C:\Users\gertn\pixelvillage-backup\`
- ✅ `pixelvillage.keystore` - ключ підпису (2560 bytes)
- ✅ `keystore-base64.txt` - закодований у base64 (3416 chars)

**⚠️ ВАЖЛИВО:** Збережіть цю папку окремо! Без ключа неможливо оновити встановлені APK на пристроях користувачів.

---

## 📋 Що потрібно зробити вручну (5 хвилин)

### Крок 1: Налаштувати GitHub Secrets

Відкрийте: https://github.com/gertyn230/game/settings/secrets/actions

**Додайте 2 секрети:**

1. **ANDROID_KEYSTORE_B64**
   - Значення візьміть з файлу: `C:\Users\gertn\pixelvillage-backup\keystore-base64.txt`
   - Відкрийте файл у блокноті, скопіюйте весь текст (3416 символів)
   - Вставте як значення секрету

2. **RELEASES_TOKEN**
   - Значення: `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`

### Крок 2: Налаштувати GitHub Variables

Відкрийте: https://github.com/gertyn230/game/settings/variables/actions

**Додайте 1 змінну:**

1. **PV_SERVER**
   - Name: `PV_SERVER`
   - Value: `https://game-n6ay.onrender.com`

### Крок 3: Оновити workflow файл вручну

GitHub блокує push workflow файлів через OAuth. Потрібно оновити вручну:

1. Відкрийте: https://github.com/gertyn230/game/edit/main/.github/workflows/build.yml

2. Замініть **весь вміст** файлу на:

```yaml
name: Build
on:
  push:
    tags:
      - 'v*.*.*'
  workflow_dispatch:

jobs:
  check-version:
    if: startsWith(github.ref, 'refs/tags/v')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check version match
        run: |
          TAG_VERSION="${GITHUB_REF_NAME#v}"
          CARGO_VERSION=$(grep '^version = ' client/Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
          echo "Tag version: $TAG_VERSION"
          echo "Cargo version: $CARGO_VERSION"
          if [ "$TAG_VERSION" != "$CARGO_VERSION" ]; then
            echo "Error: Tag version ($TAG_VERSION) does not match Cargo.toml version ($CARGO_VERSION)"
            exit 1
          fi

  windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      - uses: Swatinem/rust-cache@v2
      - name: Build client
        env:
          PV_SERVER: ${{ vars.PV_SERVER }}
        run: |
          if ([string]::IsNullOrEmpty($env:PV_SERVER)) {
            Write-Error "PV_SERVER variable is not set"
            exit 1
          }
          cargo build --release -p client
      - name: Rename executable
        run: Copy-Item target\release\client.exe PixelVillage.exe
      - uses: actions/upload-artifact@v4
        with:
          name: PixelVillage-windows
          path: PixelVillage.exe

  android:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Rust 1.95 and Android targets
        run: |
          rustup toolchain install 1.95.0 --profile minimal
          rustup default 1.95.0
          rustup target add armv7-linux-androideabi aarch64-linux-android

      - name: Restore Android keystore
        if: startsWith(github.ref, 'refs/tags/v')
        run: |
          mkdir -p ~/.android
          echo "${{ secrets.ANDROID_KEYSTORE_B64 }}" | base64 -d > ~/.android/debug.keystore

      - name: Set version_code from run_number
        if: startsWith(github.ref, 'refs/tags/v')
        run: |
          sed -i "s/^version_code = .*/version_code = ${{ github.run_number }}/" client/Cargo.toml
          cat client/Cargo.toml | grep version_code

      - name: Install cargo-quad-apk
        run: cargo +1.95.0 install --git https://github.com/hoklims/cargo-quad-apk cargo-quad-apk

      - name: Android SDK
        run: |
          SDKM="$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager"
          yes | "$SDKM" --licenses > /dev/null || true
          "$SDKM" "platforms;android-36" "build-tools;36.0.0"

      - name: Build APK
        env:
          PV_SERVER: ${{ vars.PV_SERVER }}
        run: |
          export NDK_HOME="${ANDROID_NDK_LATEST_HOME:-$ANDROID_NDK_HOME}"
          cd client
          cargo quad-apk build --release

      - name: Find and rename APK
        id: find_apk
        run: |
          APK_PATH=$(find . -name "*.apk" -type f | head -n 1)
          if [ -n "$APK_PATH" ]; then
            cp "$APK_PATH" PixelVillage-android.apk
            echo "apk_found=true" >> $GITHUB_OUTPUT
          else
            echo "apk_found=false" >> $GITHUB_OUTPUT
          fi

      - uses: actions/upload-artifact@v4
        if: steps.find_apk.outputs.apk_found == 'true'
        with:
          name: PixelVillage-android
          path: PixelVillage-android.apk

  release:
    if: startsWith(github.ref, 'refs/tags/v')
    needs: [check-version, windows, android]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4

      - name: Download Windows artifact
        uses: actions/download-artifact@v4
        with:
          name: PixelVillage-windows

      - name: Download Android artifact
        uses: actions/download-artifact@v4
        with:
          name: PixelVillage-android
        continue-on-error: true

      - name: Calculate SHA256
        run: |
          sha256sum PixelVillage.exe | awk '{print $1}' > sha256.txt
          SHA256=$(cat sha256.txt)
          SIZE=$(stat -c%s PixelVillage.exe)
          echo "SHA256: $SHA256"
          echo "Size: $SIZE bytes"
          echo "sha256=$SHA256" >> $GITHUB_ENV
          echo "size=$SIZE" >> $GITHUB_ENV

      - name: Generate latest.json
        run: |
          TAG_VERSION="${GITHUB_REF_NAME#v}"

          # Extract notes from CHANGELOG.md or tag message
          if [ -f CHANGELOG.md ]; then
            NOTES=$(grep -A 10 "## \[${TAG_VERSION}\]" CHANGELOG.md | tail -n +2 | sed '/^## /q' | sed '$d' || echo "Update to version ${TAG_VERSION}")
          else
            NOTES="Update to version ${TAG_VERSION}"
          fi

          # Escape for JSON
          NOTES_JSON=$(echo "$NOTES" | jq -Rs .)

          cat > latest.json <<EOF
          {
            "version": "${TAG_VERSION}",
            "sha256": "${sha256}",
            "size": ${size},
            "notes": ${NOTES_JSON}
          }
          EOF

          cat latest.json

      - name: Create GitHub Release in game-releases
        uses: softprops/action-gh-release@v1
        with:
          repository: gertyn230/game-releases
          tag_name: ${{ github.ref_name }}
          name: Pixel Village ${{ github.ref_name }}
          body: |
            🎮 **Pixel Village ${{ github.ref_name }}**

            ## 🚀 Автоматичне оновлення
            Гра тепер оновлюється автоматично! Просто запустіть старішу версію і з'явиться вікно з кнопкою "Встановити".

            ## 📥 Завантажити
            - **Windows**: [PixelVillage.exe](https://github.com/gertyn230/game-releases/releases/download/${{ github.ref_name }}/PixelVillage.exe)
            - **Android**: [PixelVillage-android.apk](https://github.com/gertyn230/game-releases/releases/download/${{ github.ref_name }}/PixelVillage-android.apk)

            ## 📱 Сайт
            https://gertyn230.github.io/game-releases/
          files: |
            PixelVillage.exe
            latest.json
            PixelVillage-android.apk
        env:
          GITHUB_TOKEN: ${{ secrets.RELEASES_TOKEN }}

  server:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --release -p server
      - uses: actions/upload-artifact@v4
        with:
          name: server-linux
          path: target/release/server
```

3. Натисніть **Commit changes** → **Commit directly to the main branch**

---

## 🎯 Як випустити версію 1.3.5

### Крок 1: Оновити версію в Cargo.toml

```powershell
# Відредагувати client\Cargo.toml
version = "1.3.5"
```

### Крок 2: Створити тег і запушити

```powershell
cd C:\Users\gertn\game
$git = "C:\Program Files\Git\bin\git.exe"

# Додати зміни
& $git add client/Cargo.toml

# Закомітити
& $git commit -m "Bump version to v1.3.5"

# Створити тег
& $git tag v1.3.5

# Запушити
& $git push origin main
& $git push origin v1.3.5
```

### Крок 3: Дочекатися збірки

1. Відкрийте: https://github.com/gertyn230/game/actions
2. Зачекайте ~10-15 хвилин
3. Перевірте реліз: https://github.com/gertyn230/game-releases/releases

---

## 🧪 Як протестувати локально

### Тест оновлення через локальний сервер

```powershell
# 1. Створити тестову папку
mkdir C:\Temp\update-test
cd C:\Temp\update-test

# 2. Створити latest.json
@"
{
  "version": "1.3.5",
  "sha256": "test",
  "size": 1000000,
  "notes": "Тестове оновлення"
}
"@ | Out-File -FilePath latest.json -Encoding utf8

# 3. Скопіювати exe
Copy-Item C:\Users\gertn\game\target\release\client.exe PixelVillage.exe

# 4. Запустити HTTP сервер
python -m http.server 8000

# 5. В іншому терміналі запустити стару версію з UPDATE_BASE_URL
$env:UPDATE_BASE_URL = "http://127.0.0.1:8000"
C:\Users\gertn\game\target\debug\client.exe
```

---

## 📊 Структура релізу

Після запуску CI в `game-releases` з'явиться:

```
Release v1.3.5
├── PixelVillage.exe (9-10 МБ)
├── latest.json (metadata)
└── PixelVillage-android.apk (10-15 МБ)
```

**Сайт:** https://gertyn230.github.io/game-releases/

---

## ⚠️ Важливі нотатки

### Android Keystore
**Резервна копія:** `C:\Users\gertn\pixelvillage-backup\`
- Зберігайте цю папку в безпечному місці (Dropbox, Google Drive, USB)
- Без ключа неможливо оновити APK на пристроях користувачів
- Пароль: `android` (не змінюйте!)

### PV_SERVER
- Debug збірки: дозволено `ws://127.0.0.1:8080/ws`
- Release збірки: **вимагається** віддалений сервер
- Поточне значення: `https://game-n6ay.onrender.com`

### Версіонування
- Єдине джерело: `client/Cargo.toml`
- Тег має збігатися з версією (`v1.3.5` = версія `1.3.5`)
- Server автоматично бере версію з `server/Cargo.toml`

---

## 🔄 Команди для наступних релізів

```powershell
cd C:\Users\gertn\game
$git = "C:\Program Files\Git\bin\git.exe"

# 1. Змінити версію в client/Cargo.toml на 1.3.6

# 2. Закомітити та запушити
& $git add client/Cargo.toml
& $git commit -m "Bump version to v1.3.6"
& $git tag v1.3.6
& $git push origin main
& $git push origin v1.3.6

# 3. Дочекатися збірки (10-15 хв)
# 4. Перевірити: https://github.com/gertyn230/game-releases/releases/tag/v1.3.6
```

---

## ✅ Чеклист готовності до релізу

- [ ] ANDROID_KEYSTORE_B64 додано до GitHub Secrets
- [ ] RELEASES_TOKEN додано до GitHub Secrets  
- [ ] PV_SERVER додано до GitHub Variables
- [ ] Workflow файл оновлено вручну через веб-інтерфейс
- [ ] Keystore збережено в безпечне місце (backup)
- [ ] Версія в client/Cargo.toml змінена на 1.3.5
- [ ] Тег v1.3.5 створено і запушено
- [ ] GitHub Actions запущено і завершилось успішно
- [ ] Реліз з'явився в game-releases
- [ ] latest.json доступний за посиланням
- [ ] PixelVillage.exe завантажується
- [ ] PixelVillage-android.apk завантажується

---

**Дата:** 2026-11-07  
**Час:** 17:34 UTC  
**Статус:** Готово до релізу після ручного налаштування (5 хвилин)

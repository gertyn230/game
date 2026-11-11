# Оновлення для macOS та iOS збірок

## Проблема
GitHub токен не має прав на оновлення `.github/workflows/build.yml` (потрібен scope `workflow`).

## Рішення

### Варіант 1: Оновити через GitHub Web UI (найпростіше)

1. Відкрий: https://github.com/gertyn230/game/blob/main/.github/workflows/build.yml
2. Натисни "Edit" (олівець)
3. Застосуй зміни з файлу нижче
4. Commit directly to main

### Варіант 2: Оновити токен

1. Йди на: https://github.com/settings/tokens
2. Знайди токен `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`
3. Додай scope `workflow`
4. Потім запуш зміни: `git push origin main`

---

## Зміни в .github/workflows/build.yml

Файл вже змінений локально в `~/game/.github/workflows/build.yml`

Щоб побачити зміни:
```bash
cd ~/game
git diff .github/workflows/build.yml
```

Або застосувати вручну:

### Додати після секції `linux:` (рядок ~103):

```yaml
  macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: Swatinem/rust-cache@v2
      - name: Build client
        env:
          PV_SERVER: ${{ vars.PV_SERVER }}
        run: |
          if [ -z "$PV_SERVER" ]; then
            echo "Error: PV_SERVER variable is not set"
            exit 1
          fi
          cargo build --release -p client
      - name: Create macOS app bundle
        run: |
          mkdir -p PixelVillage.app/Contents/MacOS
          mkdir -p PixelVillage.app/Contents/Resources
          cp target/release/client PixelVillage.app/Contents/MacOS/PixelVillage
          chmod +x PixelVillage.app/Contents/MacOS/PixelVillage

          # Create Info.plist
          cat > PixelVillage.app/Contents/Info.plist <<EOF
          <?xml version="1.0" encoding="UTF-8"?>
          <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
          <plist version="1.0">
          <dict>
              <key>CFBundleName</key>
              <string>Pixel Village</string>
              <key>CFBundleDisplayName</key>
              <string>Pixel Village</string>
              <key>CFBundleIdentifier</key>
              <string>com.gertyn.pixelvillage</string>
              <key>CFBundleVersion</key>
              <string>${GITHUB_REF_NAME#v}</string>
              <key>CFBundleShortVersionString</key>
              <string>${GITHUB_REF_NAME#v}</string>
              <key>CFBundleExecutable</key>
              <string>PixelVillage</string>
              <key>CFBundlePackageType</key>
              <string>APPL</string>
              <key>NSHighResolutionCapable</key>
              <true/>
          </dict>
          </plist>
          EOF

          # Create DMG
          hdiutil create -volname "Pixel Village" -srcfolder PixelVillage.app -ov -format UDZO PixelVillage-macOS.dmg
      - uses: actions/upload-artifact@v4
        with:
          name: PixelVillage-macos
          path: PixelVillage-macOS.dmg

  ios:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install iOS targets
        run: |
          rustup target add aarch64-apple-ios x86_64-apple-ios

      - name: Install cargo-mobile2
        run: cargo install --git https://github.com/tauri-apps/cargo-mobile2

      - name: Check if cargo-mobile2 config exists
        id: check_mobile
        run: |
          if [ -f ".cargo-mobile/config.toml" ]; then
            echo "has_config=true" >> $GITHUB_OUTPUT
          else
            echo "has_config=false" >> $GITHUB_OUTPUT
            echo "⚠️ iOS build not configured yet. Run 'cargo mobile init' locally first."
          fi

      - name: Build iOS
        if: steps.check_mobile.outputs.has_config == 'true'
        env:
          PV_SERVER: ${{ vars.PV_SERVER }}
        run: |
          cd client
          cargo mobile build --release --platform ios

      - name: Package IPA
        if: steps.check_mobile.outputs.has_config == 'true'
        run: |
          # Find the .app bundle
          APP_PATH=$(find client/target -name "*.app" -type d | head -n 1)
          if [ -n "$APP_PATH" ]; then
            mkdir -p Payload
            cp -r "$APP_PATH" Payload/
            zip -r PixelVillage-iOS.ipa Payload
            echo "ipa_found=true" >> $GITHUB_OUTPUT
          else
            echo "ipa_found=false" >> $GITHUB_OUTPUT
          fi
        id: package_ipa

      - uses: actions/upload-artifact@v4
        if: steps.check_mobile.outputs.has_config == 'true' && steps.package_ipa.outputs.ipa_found == 'true'
        with:
          name: PixelVillage-ios
          path: PixelVillage-iOS.ipa
        continue-on-error: true
```

### Змінити `needs:` в секції `release:` (рядок ~132):

```yaml
needs: [check-version, windows, android, linux, macos, ios]
```

### Додати download artifacts в секції `release:`:

Після `Download Linux artifact` додати:

```yaml
      - name: Download macOS artifact
        uses: actions/download-artifact@v4
        with:
          name: PixelVillage-macos
        continue-on-error: true
```

Після `Download Android artifact` додати:

```yaml
      - name: Download iOS artifact
        uses: actions/download-artifact@v4
        with:
          name: PixelVillage-ios
        continue-on-error: true
```

### Оновити release body (рядок ~202):

```yaml
            ## 📥 Завантажити
            - **Windows**: [PixelVillage.exe](https://github.com/gertyn230/game-releases/releases/download/${{ github.ref_name }}/PixelVillage.exe)
            - **macOS**: [PixelVillage-macOS.dmg](https://github.com/gertyn230/game-releases/releases/download/${{ github.ref_name }}/PixelVillage-macOS.dmg)
            - **Linux**: [PixelVillage](https://github.com/gertyn230/game-releases/releases/download/${{ github.ref_name }}/PixelVillage)
            - **Android**: [PixelVillage-android.apk](https://github.com/gertyn230/game-releases/releases/download/${{ github.ref_name }}/PixelVillage-android.apk)
            - **iOS**: [PixelVillage-iOS.ipa](https://github.com/gertyn230/game-releases/releases/download/${{ github.ref_name }}/PixelVillage-iOS.ipa) (потрібен джейлбрейк або developer account)
```

### Оновити files в release (рядок ~209):

```yaml
          files: |
            PixelVillage.exe
            PixelVillage
            PixelVillage-macOS.dmg
            latest.json
            PixelVillage-android.apk
            PixelVillage-iOS.ipa
```

---

## Після оновлення

Коли workflow оновиться, наступний деплой (`./deploy.ps1`) автоматично створить збірки для всіх платформ!

🎮 macOS - DMG для MacBook  
📱 iOS - IPA для iPhone/iPad

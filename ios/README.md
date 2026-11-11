# ios

SwiftUI uygulaması (iPhone, iPad; iOS 18+). Xcode projesi XcodeGen ile `project.yml`'dan üretilir; `.xcodeproj` depoya girmez ve elle düzenlenmez (ADR-0001). Derleme ve testler macOS CI'da, cihaz testi ürün sahibinde. İskelet: iş T0.6.

## Yapı

- `project.yml`: hedefler `Demleme` (uygulama), `DemlemeTests` (birim, Swift Testing), `DemlemeUITests` (arayüz, XCTest).
- `Demleme/App`: giriş noktası ve ilk ekran. `Demleme/Design/Tokens.swift`: üretilmiş renk sabitleri (elle düzenlenmez).
- `Demleme/Resources/Assets.xcassets/Colors`: üretilmiş renk asset'leri.
- `scripts/`: `gen_colors.py`, `validate.py`.
- `.swiftlint.yml`, `.swiftformat`: biçim ve denetim ayarları.

Bundle id öneki `app.demleme` yer tutucudur, **değiştirilecek**. İmzalama ayarları (`DEVELOPMENT_TEAM`) bilerek boştur; CI ve ürün sahibi ayarlar.

## Token üretimi

Renkler `tasarim-sistemi/tokens.json`'dan üretilir; üretilen dosyalar depoda tutulur.

    python3 -I ios/scripts/gen_colors.py          # üret
    python3 -I ios/scripts/gen_colors.py --check  # güncel değilse 1 ile çıkar

Açık ve koyu görünüm her renkte vardır. Yüksek karşıtlık değeri olan renklerde `contrast: high` varyantı üretilir (açık + yüksek karşıtlık için tokens.json'daki değer; koyu + yüksek karşıtlık için koyu değer, çünkü tokens.json tek karşıtlık değeri tanımlar). Tokens.json değişince betiği çalıştırıp çıktıyı commit'le.

## Linux'ta doğrulama

    python3 -I ios/scripts/validate.py

`project.yml` okunabilirliği, colorset JSON geçerliliği ve `gen_colors.py --check` denetlenir. Swift derlenmez.

## macOS'ta

    brew install xcodegen swiftlint swiftformat
    cd ios && xcodegen generate
    xcodebuild test -project Demleme.xcodeproj -scheme Demleme \
      -destination 'platform=iOS Simulator,name=iPhone 16'

İlk derlemede yapılacaklar: imza ve bundle id'yi ayarla, derleme ve testleri çalıştır, `swiftlint` ve `swiftformat --lint .` çıktısını düzelt, simülatör adını CI'daki Xcode sürümüne göre güncelle (Xcode sürümü CI'da sabitlenir).

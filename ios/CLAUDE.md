# ios

## Ne yapar
- SwiftUI uygulaması (iPhone, iPad; iOS 18+, Swift 6.0).
- Xcode projesi XcodeGen ile `project.yml`'dan üretilir; hedefler `Demleme`, `DemlemeTests`, `DemlemeUITests`.
- Giriş noktası ve ilk ekran `Demleme/App` altında.

## Neye dokunmaz
- `.xcodeproj` elle düzenlenmez ve depoya girmez; değişiklik `project.yml`'da yapılır.
- İş kuralı ve hesap yok; hesap sunucudadır, iOS yalnız arayüzü çizer.
- Bundle id ve imzalama (`DEVELOPMENT_TEAM`) kararı ürün sahibinde ve CI'da; burada doldurulmaz.

## Nasıl test edilir
- Linux: `python3 -I ios/scripts/validate.py` (Swift derlenmez).
- Renk üretimi güncel mi: `python3 -I ios/scripts/gen_colors.py --check`.
- Derleme ve testler macOS CI'da; cihaz testi ürün sahibinde.

## Kurallar
- Üretilen dosyalar elle düzenlenmez: `Demleme/Resources/Assets.xcassets/Colors` ve `Demleme/Design/Tokens.swift`; `python3 -I ios/scripts/gen_colors.py` ile yeniden üretilir.
- Biçim: SwiftFormat ve SwiftLint (`.swiftformat`, `.swiftlint.yml`).
- Kod İngilizce; belgeler Türkçe. Kullanıcıya görünen metin kataloğundan gelir.
- Commit mesajı iş numarasıyla başlar: `T0.x: ...`.

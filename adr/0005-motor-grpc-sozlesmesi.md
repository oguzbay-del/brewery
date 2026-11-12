# ADR-0005: Hesap motoru gRPC sözleşmesi

- **Durum:** kabul edildi
- **Tarih:** 12.11.2026
- **İş:** T1.11 (tasarım parçası)

## Bağlam

Motorun yaklaşık 400 açık fonksiyonu var (ADR-0004 biçiminde: girdi yapısı, adlı çıktı, tek `engine::Error`). API servisi bunları `engine-service` üzerinden gRPC ile çağırır; istemciye hesap kodu gitmez.

## Karar

1. **Paket ve sürüm:** `demleme.engine.v1`; geriye uyumsuz değişiklik `v2` paketiyle gelir. Alan numarası yeniden kullanılmaz.
2. **Hizmetler, ekran ihtiyacına göre:** her fonksiyon için uç açılmaz.
   - `RecipeService.Calculate(RecipeInput) → RecipeSummary`: OG, FG, ABV, IBU, SRM/EBC, verim, hacim zinciri, kalori; tarif ekranının tek çağrısı.
   - `WaterService` (profil, tuz, asit, mash pH), `MashService` (strike, infüzyon, sparge planı), `YeastService` (pitch, starter, mead besini, karbonasyon), `FermentationService` (tahmin, tetikleyici, kalibrasyon), `BreweryService` (maliyet, harman, vergi, spunding).
   - `ToolService`: küçük araçlar (dönüştürücüler, refraktometre, hidrometre, seyreltme) tek tek RPC.
3. **Birimler:** proto metriktir (L, kg, g, °C, dk, SG, SRM, %). BeerSmith iç birimlerine dönüşüm yalnız `engine-service`'teki tek dönüştürücü katmanında yapılır (ADR-0004 madde 4). Alan adları birim eki taşır (`volume_l`, `temp_c`).
4. **Kodlar:** her enum proto `enum` olur, 0 değeri `*_UNSPECIFIED`; belirtilmemiş değer `INVALID_ARGUMENT` döner.
5. **Hatalar:** `engine::Error` → gRPC durumu: `InvalidInput`, `InvalidBands`, `EmptyInput` → `INVALID_ARGUMENT` (alan adı `google.rpc.BadRequest` ayrıntısında); `NoConvergence`, `Unreachable` → `FAILED_PRECONDITION`; `DivisionByZero`, `Overflow` → `INVALID_ARGUMENT`. Her istekte önce `RecipeInput::validate()`.
6. **Para ve vergi:** ondalık değerler proto'da metin (`string`, ör. `"12.34"`) taşınır; `double` kullanılmaz.
7. **Sınama:** her RPC için parite vektöründen türetilmiş en az bir uç testi; proto'dan üretilen OpenAPI ve istemciler CI fark denetiminde (T0.4).

## Sonuçlar

- T1.11 uygulaması hizmet hizmet ilerler; ilk parça `ToolService` (küçük, bağımsız), sonra `RecipeService`.

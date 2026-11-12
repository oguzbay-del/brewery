# ADR-0004: Motor girdi modeli, hata tipi, birimler ve kodlar

- **Durum:** kabul edildi
- **Tarih:** 12.11.2026
- **İş:** T1.16 (dayanak: T1.12 incelemeleri; `gelistirme/raporlar/T1.12-*.md`)

## Bağlam

T1.1–T1.14 bölüm bölüm yazıldı. İnceleme dört yapısal sorun buldu: 01 ve 02 ayrı `Recipe`/`Fermentable` tanımlıyor ve geç ekstrakt profili iki kez yazılmış (01 D1, D2); Python'un istisna attığı girdilerde Rust sessizce NaN ya da sonsuz döndürüyor (01 C2, 02–05 B3); birimler modüller arasında tutarsız ve adlarda görünmüyor (B5); kodlar `i64`, `u8` ve enum karışık, geçersiz kod sessizce varsayılana düşüyor, birçok fonksiyon 8–12 konumsal `f64` alıyor (B6, B11). gRPC katmanı (T1.11) bunların üstüne kurulursa sorunlar sözleşmeye taşınır.

## Karar

1. **Ortak tarif modeli.** Yeni modül `engine::recipe` yalnız veri tipleri ve doğrulama içerir, formül içermez: `RecipeInput`, `Equipment`, `Fermentable`, `Hop`, `Yeast`, `Misc`, `Mash`, `MashStep`, `Options` ve ortak enum'lar (`RecipeType`, `GrainType`, `GrainUse`, `HopForm`, `HopUse`, `StepKind`, `MiscUse`). Alanlar T1.12 01 raporundaki öneriyi izler. `density`, `hops`, `yeast` kendi tarif yapılarını bırakır; `hops` fermentable puanını `density::grain_og_points`'ten, geç ekstrakt profilini tek uygulamadan alır.
2. **Tek uygulama.** Aynı hesap motorda bir kez yazılır (ADR-0003 madde 4'ün uygulaması). Çiftler (geç ekstrakt profili, `brix_to_sg`, ortalama attenuasyon, kaynama sıcaklığı, su ekleme hacmi, mash sonrası OG, iki `RecipeType`, iki oz→g sabiti) tek yere iner; saf dönüşümler ve birim sabitleri `units`'te durur. Referans betiklerde de aynı fonksiyonu iki bölüm farklı yazmışsa önce spesifikasyon tek tanıma bağlanır.
3. **Hata tipi.** `engine::Error` (ör. `InvalidInput { field, reason }`, `NoConvergence`, `Unreachable`). Python referansının istisna attığı her girdide Rust `Err` döndürür; parite vektöründeki `{"hata": ...}` vakaları `Err` ile eşleşir. Sonlu olmayan girdi motorun kapısında (`RecipeInput::validate` ve tek tek araç fonksiyonlarının başında) reddedilir; motor NaN ya da sonsuz döndürmez. Yakınsamayan yineleme sınırlıdır ve bunu bildirir.
4. **Birimler.** Motor fonksiyonu spesifikasyonun birimini alır (ADR-0003 madde 5 geçerli: 01–04 BeerSmith iç birimleri, 05 ve 07–11 metrik). Yeni: her açık alan ve parametre adı birim ekini taşır (`_oz`, `_floz`, `_gal`, `_l`, `_kg`, `_g`, `_f`, `_c`, `_min`, `_h`, `_pct`, `_sg`, `_srm`); oran ve SG eksiz kalabilir. Dönüşüm yalnız `engine-service`'te, tek dönüştürücü katmanında yapılır; API ve proto metriktir.
5. **Kodlar.** Açık API'de ham sayı kodu yoktur; enum kullanılır. Koddan enum'a çeviri tek yerde (`from_code`) yapılır ve geçersiz kod `Error::InvalidInput` döndürür. BeerSmith kod değerleri enum'da belgelenir.
6. **İmza biçimi.** Dörtten fazla girdisi olan açık fonksiyon girdi yapısı alır; birden fazla çıktısı olan fonksiyon adlı alanlı yapı döndürür (demet yok). Böylece proto mesajları Rust yapılarıyla bire bir eşlenir.
7. **Gösterim motorda değildir.** Metin etiketleri ve gösterim yuvarlaması motorda yapılmaz; motor kod ve tam değer döndürür. İstisna: referansın kendi hesabının parçası olan yuvarlama (ADR-0003 madde 6).

## Sonuçlar

- 01 ve 02 parite vektörleri aynı reçete fikstürlerini paylaşır; iki bölümün aynı reçetede tutarlılığı da sınanır.
- Açık API değişir; T1.11 bu biçimin üstüne kurulur. Değişiklik bir kez, gRPC'den önce yapılır.
- `validate()` ve hata tipi API'nin 400 yanıtlarının kaynağıdır.

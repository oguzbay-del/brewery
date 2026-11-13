# sozlesme

API sözleşmesi. Elle düzenlenmez; koddan üretilir.

| Klasör | İçerik |
|---|---|
| `openapi/openapi.json` | Tek OpenAPI dosyası; `sunucu/crates/api` kodundan (`utoipa`) üretilir. Yollar `/v1/...` altındadır; `/healthz` sürümsüz kalır |
| `istemci/typescript/` | `openapi-typescript` ile `src/schema.d.ts` üretir (web istemcisi bu tipleri kullanır) |
| `istemci/swift/` | Apple swift-openapi-generator ayarı; Swift kodu depoya konmaz, macOS'ta derlemede üretilir |
| `katalog/` | Alan tanımları (SI birim, sınır, adım, zorunluluk), birim dönüşüm katsayıları ve TR/EN metin kataloğu: tek veri dosyası `katalog.json` (şema: `katalog.schema.json`). `uret.py` TypeScript (`istemci/typescript/src/katalog.ts`), Swift (`istemci/swift/Katalog.swift`) ve Rust'ın gömdüğü kopyayı (`sunucu/crates/common/src/catalog/`) üretir; `test.py` denetler (T2.17) |
| `proto/` | Hesap motoru gRPC tanımları |
| `scripts/` | `gen.sh` (yalnız üretir), `check.sh` (üretir, fark varsa 1 döner) |

## Komutlar

```sh
bash sozlesme/scripts/gen.sh     # (ya da `make gen`) openapi.json, schema.d.ts ve katalog çıktılarını üretir (ilk çalıştırmada npm ci yapar)
bash sozlesme/scripts/check.sh   # yeniden üretir; commit edilmiş dosyalardan fark varsa 1 döner (CI'da çalışır)
```

Yalnız OpenAPI için: `cd sunucu && cargo run -p api --bin gen-openapi`.

## Kurallar

- API koddaki `#[utoipa::path]` açıklamalarından üretilir. Uç nokta veya şema değişince `gen.sh` çalıştırılır, üretilen dosyalar aynı commit'e girer.
- Üretilen dosyalar (`openapi.json`, `schema.d.ts`) elle düzenlenmez. Üretim deterministiktir (anahtarlar sıralı, sonda satır sonu).
- Hatalar her yerde `{ "error": { "code", "message" } }` zarfıyla döner (`ErrorResponse`).
- `GET /v1/version` yanıtında `engine` alanı hesap motoru bağlanana kadar `null`.

## Swift istemcisi (macOS)

Swift araç zinciri Linux ortamında yok; kod burada üretilmez. iOS projesinde (`ios/`) [swift-openapi-generator](https://github.com/apple/swift-openapi-generator) SwiftPM eklentisi olarak kullanılır:

1. Paket bağımlılıkları: `swift-openapi-generator` (eklenti), `swift-openapi-runtime`, `swift-openapi-urlsession`.
2. Hedefin kaynak klasörüne `openapi-generator-config.yaml` (bu klasördeki dosya) ve `openapi.json` (`sozlesme/openapi/openapi.json`; sembolik bağlantı veya derleme öncesi kopya) konur.
3. Hedefte `OpenAPIGenerator` eklentisi etkinleştirilir; `types` ve `client` kodu derlemede üretilir, `public` erişimlidir.

Elle üretmek için: `swift run swift-openapi-generator generate --config sozlesme/istemci/swift/openapi-generator-config.yaml --output-directory <hedef> sozlesme/openapi/openapi.json`.

## Alan, birim ve metin kataloğu (T2.17)

- Tek kaynak `katalog/katalog.json`. Alan anahtarı `kapsam.alan` (ör. `hop.alpha_pct`), alan adı ve birim eki API'deki JSON alanıyla aynıdır. Değerler SI takımındadır (kg, L, °C, dk, SG, SRM, ppm, bar, %; ADR-0007); kullanıcı birimi yalnız gösterimdir.
- Sınırlar tüm modlarda kabul edilen en geniş aralıktır, `modes` yalnız daha dar Ev/İşletme sınırı verir. `verify: true` ürün kararıdır, kaynakla doğrulanmadı (doğrulanacak).
- Dönüşüm: `SI = (değer + pre) × mul / div + post`. SG↔°P ve SG↔°Bx doğrusal değildir; `nonlinear` adı formülü gösterir (`formuller/01` §1.1-§1.3), formül üreticide yazılıdır.
- Katsayılar tanımı kesin olan değerlerdir (1 lb = 0.45359237 kg, 1 US gal = 3.785411784 L, 1 imp gal = 4.54609 L, 1 psi = 6894.757293168361 Pa). Motor BeerSmith'in yuvarlak sabitlerini kullanır (`LB_KG` = 0.4535924); fark gösterimde görünmez ve `common` testinde sınırlıdır.
- Rust: `common::catalog` (`validate`, `limits`, `Unit::to_si`). Testi motorun `RecipeInput::validate` kapısıyla karşılaştırır: katalog aralığı motorun kabul ettiğinin içinde kalır, motorun alt sınırı 0 ise katalogunki de 0'dır.
- Yeni alan: `katalog.json`'a alan ve her iki dilde etiket eklenir, `make gen` çalışır, çıktılar aynı commit'e girer.

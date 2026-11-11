# sozlesme

API sözleşmesi. Elle düzenlenmez; koddan üretilir.

| Klasör | İçerik |
|---|---|
| `openapi/openapi.json` | Tek OpenAPI dosyası; `sunucu/crates/api` kodundan (`utoipa`) üretilir. Yollar `/v1/...` altındadır; `/healthz` sürümsüz kalır |
| `istemci/typescript/` | `openapi-typescript` ile `src/schema.d.ts` üretir (web istemcisi bu tipleri kullanır) |
| `istemci/swift/` | Apple swift-openapi-generator ayarı; Swift kodu depoya konmaz, macOS'ta derlemede üretilir |
| `proto/` | Hesap motoru gRPC tanımları |
| `scripts/` | `gen.sh` (yalnız üretir), `check.sh` (üretir, fark varsa 1 döner) |

## Komutlar

```sh
bash sozlesme/scripts/gen.sh     # openapi.json ve schema.d.ts üretir (ilk çalıştırmada npm ci yapar)
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

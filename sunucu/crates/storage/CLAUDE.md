# storage

## Ne yapar
- Nesne deposu arayüzü `ObjectStore` ve S3 uygulaması `S3Store` (S3, GCS birlikte çalışabilirliği, SeaweedFS): imzalı `PUT`/`GET` adresi (SigV4, sorgu imzası), `head`, `digest` (akıtarak SHA-256), `put`, `delete`, `ensure_bucket`.
- `S3Config`: `S3_*` ortam değişkenleri (`S3_ENDPOINT` yoksa depo kapalı).
- Ayrıntı: `README.md`, karar: `adr/0008-api-kurallari.md` (T2.18 notları).

## Neye dokunmaz
- İş kuralı yoktur (boyut, tür, yetki `api` crate'indedir).
- Azure Blob uygulaması yok; eklenirse `ObjectStore`'u uygular.

## Nasıl test edilir
- `sunucu/` içinde: `cargo test -p storage`. `tests/s3_live.rs` compose'taki `objectstore`'a (127.0.0.1:8333) bağlanır: `docker compose -f altyapi/compose/compose.yaml up -d --wait objectstore`. `S3_*` değişkenleri adresi ve anahtarı değiştirir.

## Kurallar
- İmza koduna dokunan değişiklik `sigv4.rs` içindeki AWS örnek testlerini geçmelidir.
- Sırlar günlüğe ve `Debug` çıktısına girmez.
- Commit mesajı iş numarasıyla başlar: `T2.x: ...`.

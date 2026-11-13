# storage

Nesne deposu için tek arayüz (T2.18). Dosyalar API'den geçmez: istemci kısa süreli imzalı adresle doğrudan depoya yükler ve oradan indirir.

| Parça | Ne yapar |
|---|---|
| `ObjectStore` | `presign_put` (boyut, tür ve SHA-256 imzalı başlık), `presign_get`, `head`, `digest`, `put`, `delete`, `ensure_bucket` |
| `S3Store` | S3 ve S3 uyumlu depolar. GCS için S3 birlikte çalışabilirliği ve HMAC anahtarı; yerelde SeaweedFS. Yol biçimi (`S3_PATH_STYLE=true`, varsayılan) ya da sanal ana makine biçimi |
| `sigv4` | AWS SigV4 (sorgu ve başlık imzası); AWS belgelerindeki örnek imzalarla sınanır |
| `S3Config` | Ortamdan okunur |

## Yapılandırma

`S3_ENDPOINT` (yoksa dosya uçları `503` verir), `S3_PUBLIC_ENDPOINT` (istemcinin eriştiği adres; imzalı adresler bununla imzalanır; varsayılan `S3_ENDPOINT`), `S3_BUCKET`, `S3_REGION` (`us-east-1`), `S3_ACCESS_KEY` ve `S3_SECRET_KEY` (`*_FILE` ile de), `S3_PATH_STYLE` (`true`), `S3_CREATE_BUCKET` (yalnız yerel yığın), `S3_PRESIGN_TTL_SECS` (600, en çok 3600).

Yerel yığında `objectstore` servisi (SeaweedFS) `127.0.0.1:8333`'te açılır; anahtar `demleme-yerel`.

## Sınırlar

- Azure Blob uygulaması yok. SAS imzası boyutu bağlayamaz; tamamlama adımındaki nesne denetimi (`api` crate'i) bu yüzden her zaman yapılır.
- İstemci `Content-Length` ve `Host` başlıklarını kendisi koyar; imzaya girerler ama `PresignedRequest.headers` içinde verilmezler.

## Test

`cargo test -p storage`. Birim testleri imzayı AWS örnekleriyle sınar; `tests/s3_live.rs` gerçek depoya karşı yükleme, indirme, süresi geçmiş bağlantı (`403`) ve imzalı sınırın zorlanmasını sınar.

# common

Tüm sunucu crate'lerinin paylaştığı parçalar: ortam değişkeninden yapılandırma (`PORT`, `SERVICE_NAME`), JSON log kurulumu (`tracing`), ortak hata tipi, temiz kapanma sinyali.
`http` özelliği açılınca axum için `/healthz` yönlendiricisi ve sunucu başlatıcısı gelir.
İş mantığına, veritabanına ve hesap kodlarına dokunmaz.
Test: `cargo test -p common` (`sunucu/` içinde).

- `config::secret_from_lookup`: `NAME` ya da `NAME_FILE` (bağlı sır dosyası) okur, ikisi birden hata (ADR-0009 madde 11.1).
- `otel` (özellik `otel`): `init_tracing_otel` OpenTelemetry izini kurar; `OTEL_EXPORTER_OTLP_ENDPOINT` varsa OTLP/HTTP ile dışa aktarır, yoksa dışa aktarım kapalıdır (iz kimlikleri günlükte yine olur). Dönen koruyucu kapanışta bekleyen izleri boşaltır.

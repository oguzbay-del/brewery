# common

Tüm sunucu crate'lerinin paylaştığı parçalar: ortam değişkeninden yapılandırma (`PORT`, `SERVICE_NAME`), JSON log kurulumu (`tracing`), ortak hata tipi, temiz kapanma sinyali.
`http` özelliği açılınca axum için `/healthz` yönlendiricisi ve sunucu başlatıcısı gelir.
İş mantığına, veritabanına ve hesap kodlarına dokunmaz.
Test: `cargo test -p common` (`sunucu/` içinde).

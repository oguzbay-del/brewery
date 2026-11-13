# common

## Ne yapar
- Tüm sunucu crate'lerinin ortak parçaları: `PORT` ve `SERVICE_NAME` ortam değişkeninden yapılandırma, JSON log (`tracing`), ortak hata tipi, temiz kapanma sinyali.
- `http` özelliği açılınca axum için `/healthz` ve sunucu başlatıcısı gelir.
- `catalog`: alan tanımı, birim ve metin kataloğu (`sozlesme/katalog` verisinden; `validate`, `limits`). Alan sınırlarının tek kaynağı; sunucu doğrulaması buradan okur (T2.17).

## Neye dokunmaz
- İş mantığı, veritabanı ve hesap kodu yok.
- Tek bir crate'e özgü davranış buraya konmaz.

## Nasıl test edilir
- `sunucu/` içinde: `cargo test -p common`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Kurallar
- Ortak tip veya özellik değişince tüm workspace clippy'den geçmelidir.
- Kod İngilizce; belgeler Türkçe.
- Commit mesajı iş numarasıyla başlar: `T0.x: ...`.

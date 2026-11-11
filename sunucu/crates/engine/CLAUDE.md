# engine

## Ne yapar
- Hesap motoru kütüphanesi; formüller saf fonksiyon olarak burada yazılır.
- Formüller `formuller/` spesifikasyonlarına ve test vektörlerine göre gelir (Aşama 1).
- Şimdilik yalnız `version()` var.

## Neye dokunmaz
- IO yok: veritabanı, ağ, saat ve async bağımlılığı eklenmez; `[dependencies]` boş kalır.
- gRPC ve sunucu kodu `engine-service`'e aittir.

## Nasıl test edilir
- `sunucu/` içinde: `cargo test -p engine`
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Kurallar
- Kod İngilizce; belgeler Türkçe.
- Formül değişikliği önce `formuller/` spesifikasyonunda yapılır, sonra kod ve test vektörü gelir.
- Commit mesajı iş numarasıyla başlar: `T0.x: ...`.

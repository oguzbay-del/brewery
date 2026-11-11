# engine

## Ne yapar
- Hesap motoru kütüphanesi; formüller saf fonksiyon olarak burada yazılır.
- Formüller `formuller/` spesifikasyonlarına ve test vektörlerine göre gelir (Aşama 1).
- Bölüm başına bir modül (`units`, `density`, `hops`, …); parite yöntemi ADR-0003.

## Neye dokunmaz
- IO yok: veritabanı, ağ, saat ve async bağımlılığı eklenmez. Yalnız saf hesap crate'i (ör. `rust_decimal`) eklenebilir; test bağımlılığı `serde_json`.
- gRPC ve sunucu kodu `engine-service`'e aittir.

## Nasıl test edilir
- `sunucu/` içinde: `cargo test -p engine` (parite: `tests/parite_NN.rs`, vektörler `make vektor` ile üretilir)
- Lint: `cargo clippy --workspace --all-targets -- -D warnings`

## Kurallar
- Kod İngilizce; belgeler Türkçe.
- Formül değişikliği önce `formuller/` spesifikasyonunda yapılır, sonra kod ve test vektörü gelir.
- Commit mesajı iş numarasıyla başlar: `T0.x: ...`.

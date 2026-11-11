# engine

Hesap motoru kütüphanesi. Kural: saf fonksiyonlar, IO yok; veritabanı, ağ, saat ve async bağımlılığı eklenmez.
Şimdilik yalnız `version()` var; formüller Aşama 1'de `formuller/` spesifikasyonlarına göre gelecek.
Ağ ve gRPC ile ilgilenmez (bu `engine-service` işidir).
Test: `cargo test -p engine` (`sunucu/` içinde).

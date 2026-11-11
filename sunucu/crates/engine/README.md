# engine

Hesap motoru kütüphanesi. Kural: saf fonksiyonlar, IO yok; veritabanı, ağ, saat ve async bağımlılığı eklenmez.
Formüller `formuller/` spesifikasyonlarına göre bölüm başına bir modülde; Python referansıyla parite ADR-0003'te.
Ağ ve gRPC ile ilgilenmez (bu `engine-service` işidir).
Test: `cargo test -p engine` (`sunucu/` içinde).

# db

## Ne yapar
- PostgreSQL erişiminin tek kapısı: sqlx havuzu (`DATABASE_URL` ya da `DATABASE_URL_FILE`), göçler, istek bağlamı yardımcıları.
- Göçler `sunucu/migrations/` altındadır ve `migrate()` ile (ya da `migrate` ikilisiyle) uygulanır; imaja gömülüdür.
- `migrate` ikilisi compose'da ve Kubernetes'te tek seferlik çalışır; uygulama servisleri göç çalıştırmaz.

## Neye dokunmaz
- Tablo ve sütun tanımları T2.5–T2.7'nin işidir; bu iş yalnız altyapıyı kurar.
- Uygulama servislerinin iş mantığı burada yazılmaz.

## Nasıl test edilir
- Gerçek PostgreSQL gerekir: `make up` (ya da yalnız `docker compose -f altyapi/compose/compose.yaml up -d postgres`), sonra kökte `make test`. Elle: `DATABASE_URL=postgres://demleme:demleme-yerel@127.0.0.1:5432/demleme cargo test -p db`.
- `#[sqlx::test]` her test için geçici veritabanı açar, göçleri uygular ve sonda siler.
- Sorgu önbelleği: kökte `make sqlx-prepare`; `sunucu/.sqlx/` commit edilir, derleme `SQLX_OFFLINE=true` ile çalışır.

## Kurallar
- Göç dosyaları `NNNN_ad.sql` (dört haneli sıra, küçük harf, alt çizgi); uygulanmış göç değiştirilmez, yeni göç eklenir.
- Kod İngilizce; belgeler Türkçe.
- Commit mesajı iş numarasıyla başlar: `T2.x: ...`.

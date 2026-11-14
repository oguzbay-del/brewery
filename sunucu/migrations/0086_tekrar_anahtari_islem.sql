-- T2.9 (F2.1 bulgu 4): tekrar anahtarı iş işlemiyle aynı işlemde işaretlenir. İş commit
-- edilir edilmez `work_committed_at` da kalıcıdır; süreç yanıtı yazmadan düşerse anahtar
-- devralınıp iş ikinci kez çalıştırılmaz (adr/0008 › 6.3).
SET LOCAL ROLE app_owner;
ALTER TABLE idempotency_keys ADD COLUMN work_committed_at timestamptz;
RESET ROLE;

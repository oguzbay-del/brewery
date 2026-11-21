# gcp-storage

Üç Cloud Storage kovası, ikisi de tek bölgede, tekdüze erişim ve herkese açık erişim engeli ile:

- `<önek>-objects`: kağıt taramaları, fotoğraf, PDF, dışa aktarma. Sürümleme açık; eski sürümler `noncurrent_version_retention_days` sonra silinir; imzalı bağlantı yüklemesi için isteğe bağlı CORS.
- `<önek>-audit`: İşletme denetim arşivi. Sürümleme ve `audit_retention_days` saklama politikası. `audit_retention_locked = true` politikayı kalıcı kilitler (WORM); geri alınamaz, yalnız canlıda ve süre kararı verildikten sonra aç.
- `<önek>-digests`: worker'ın haftalık denetim özetleri (`S3_DIGEST_BUCKET`). Sürümleme ve `digest_retention_days` (400) saklama politikası; `digest_retention_locked = true` kilitler (yalnız canlıda). GCS S3 Object Lock API'sini sunmaz; kümede `S3_OBJECT_LOCK_MODE` verilmez (doğrulanacak).

IAM bağlamaları (hangi hizmet hesabı hangi kovaya erişir) uygulama katmanında, kova düzeyinde verilir; bu modül geniş rol vermez.

# gcp-storage

İki Cloud Storage kovası, ikisi de tek bölgede, tekdüze erişim ve herkese açık erişim engeli ile:

- `<önek>-objects`: kağıt taramaları, fotoğraf, PDF, dışa aktarma. Sürümleme açık; eski sürümler `noncurrent_version_retention_days` sonra silinir; imzalı bağlantı yüklemesi için isteğe bağlı CORS.
- `<önek>-audit`: İşletme denetim arşivi. Sürümleme ve `audit_retention_days` saklama politikası. `audit_retention_locked = true` politikayı kalıcı kilitler (WORM); geri alınamaz, yalnız canlıda ve süre kararı verildikten sonra aç.

IAM bağlamaları (hangi hizmet hesabı hangi kovaya erişir) uygulama katmanında, kova düzeyinde verilir; bu modül geniş rol vermez.

# gcp-secrets

Cloud KMS anahtar halkası ve şifreleme anahtarları (90 günde bir döner, silinmeye karşı korumalı) ile Secret Manager sır kapları.

- Sır değerleri Terraform'a girmez: yalnız kap açılır, değer Secret Manager'a elle veya iş akışıyla eklenir.
- Sırlar seçilen tek bölgede çoğaltılır (veri yeri).
- Okuma yetkisi sır başına (`secret_accessors`), anahtar yetkisi anahtar başına (`key_users`) verilir; proje geneli rol yok.
- Anahtarlar şimdilik üretilir; hangi hizmetin CMEK kullanacağı (SQL, Storage, Redis, Registry) ayrı karardır.
- Kubernetes'e sır aktarımı External Secrets ile (altyapi/k8s).
- Gerekli API'ler: `cloudkms`, `secretmanager`.

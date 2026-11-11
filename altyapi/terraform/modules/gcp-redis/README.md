# gcp-redis

Memorystore for Redis: özel hizmet erişimiyle özel IP, AUTH ve aktarım şifrelemesi (TLS) açık.

- Canlı ve testte `STANDARD_HA`, geliştirmede `BASIC`.
- `auth_string` duyarlı çıktıdır ve Terraform durumunda durur; durum dosyası şifreli GCS kovasında tutulmalı. Sırrı uygulamaya Secret Manager ve External Secrets ile ver.
- Kökte `depends_on = [module.network]` ile özel hizmet erişiminin hazır olması beklenir.
- Gerekli API: `redis`.

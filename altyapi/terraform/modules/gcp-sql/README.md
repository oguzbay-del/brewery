# gcp-sql

Cloud SQL for PostgreSQL: yalnız özel IP, şifreli bağlantı zorunlu, otomatik yedek ve noktaya geri dönüş (PITR), isteğe bağlı okuma kopyası ve CMEK.

- Kimlik doğrulama IAM ile (parola yok): `iam_service_accounts` listesindeki hesaplara veritabanı kullanıcısı açılır. Parola veya gizli bilgi Terraform durumuna girmez.
- `availability_type = "REGIONAL"` canlı ve testte; geliştirmede `ZONAL`.
- Silme koruması varsayılan açık.
- Başka bölgeye yedek kopyası (plan: veri yeri) bu modülde yok; bölge içi veri yeri kuralı nedeniyle ayrı karar gerekir.
- `network_id` çıktısının bağlı olduğu özel hizmet erişimi hazır olmalı: kökte `depends_on = [module.network]`.
- Gerekli API'ler: `sqladmin`, `servicenetworking`.

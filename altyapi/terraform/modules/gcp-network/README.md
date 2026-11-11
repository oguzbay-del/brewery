# gcp-network

VPC (özel alt ağ, ikincil Pod ve Service aralıkları), Cloud NAT ve özel hizmet erişimi (Cloud SQL ve Memorystore için özel IP) kurar.

- Yalnız özel IP; herkese açık düğüm yok. Dışarı çıkış Cloud NAT ile.
- Bölge başına bir çağrı. Aralıklar ortamlar arasında çakışmamalı (ileride eşleme ihtiyacı için).
- Gerekli API'ler: `compute`, `servicenetworking` (ortam kökünde açılır).
- Çıktıdaki `psa_connection_id`, SQL ve Redis modüllerinin `depends_on` bağlantısı için kullanılır.

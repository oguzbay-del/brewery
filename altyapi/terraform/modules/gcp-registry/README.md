# gcp-registry

Artifact Registry Docker deposu: değiştirilemez etiketler, eski ve etiketsiz imajlar için temizlik politikası.

- Okuma ve yazma rolleri yalnız bu depoya, adı verilen üyelere verilir (proje geneli rol yok).
- `reader_members`: düğüm hizmet hesabı (`gcp-gke` çıktısı). `writer_members`: CI hizmet hesabı (GitHub Actions için Workload Identity Federation, ayrı iş).
- Gerekli API: `artifactregistry`.

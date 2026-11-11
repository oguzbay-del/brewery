# gcp-gke

GKE Autopilot kümesi: özel düğümler, yetkili ağlarla kısıtlı kontrol düzlemi, Workload Identity, sürüm kanalı.

- Düğümler varsayılan Compute hesabı yerine en az yetkili ayrı hizmet hesabıyla çalışır (günlük ve ölçüm yazma rolleri; her biri `main.tf` içinde gerekçeli).
- `master_authorized_networks` zorunlu girdidir; boş liste dış erişimi kapatır.
- Pod'lar için Google API erişimi Workload Identity ile verilir; hizmet hesabı bağlamaları uygulama katmanında (k8s) yapılır.
- NATS, Redis istemcileri, hesap motoru gibi iş yükleri bu modülde değil `altyapi/k8s/` altındadır.
- Gerekli API: `container`.

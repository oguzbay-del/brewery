# altyapi/terraform

Bulut altyapısı (ilk bulut Google Cloud). Terraform sürümü `.terraform-version` dosyasında sabit.

- `modules/gcp-*`: bulut bağımlı tek yer. Başka bulut için yalnız yeni `modules/aws-*` veya `modules/azure-*` yazılır.
- `envs/dev`, `envs/test`, `envs/canli`: modülleri çağıran kök yapılandırmalar. Her kök tek bölge kurar (`region` değişkeni: `us-east4` veya `europe-west2`); iki bölge için aynı kök ayrı tfvars ve ayrı durum öneki ile çalıştırılır.
- NATS Terraform'da değil, Kubernetes içindedir (`altyapi/k8s/`).
- Durum kovası (`backend "gcs"`) henüz yok; bloklar yorum satırı. Hesap kurulunca açılır.
- Gerçek proje kimliği ve sırlar depoya girmez: `terraform.tfvars` gitignore'dadır, örnek `terraform.tfvars.example`'dır.

Kontrol (hesap gerekmez):

```
cd envs/dev && terraform init -backend=false && terraform validate
terraform fmt -check -recursive   # altyapi/terraform altında
```

`.terraform.lock.hcl` dosyaları depoda kalır; sağlayıcı yükseltmesi ayrı iştir.

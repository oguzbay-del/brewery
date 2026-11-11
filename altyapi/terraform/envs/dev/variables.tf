variable "project_id" {
  description = "Bu ortamın Google Cloud proje kimliği (örnek dosyada yer tutucu)."
  type        = string
}

variable "region" {
  description = "Dağıtım bölgesi. ABD çalışma alanları us-east4, İngiltere çalışma alanları europe-west2; her bölge ayrı durum ve ayrı tfvars ile kurulur."
  type        = string

  validation {
    condition     = contains(["us-east4", "europe-west2"], var.region)
    error_message = "region us-east4 veya europe-west2 olmalı."
  }
}

variable "name_prefix" {
  description = "Kaynak adı öneki."
  type        = string
  default     = "demleme"
}

variable "subnet_cidr" {
  description = "Düğüm alt ağı aralığı."
  type        = string
  default     = "10.10.0.0/20"
}

variable "pods_cidr" {
  description = "GKE Pod aralığı."
  type        = string
  default     = "10.11.0.0/16"
}

variable "services_cidr" {
  description = "GKE Service aralığı."
  type        = string
  default     = "10.12.0.0/20"
}

variable "master_authorized_networks" {
  description = "Kubernetes kontrol düzlemine erişebilecek CIDR'ler (CI ve yönetici ağı)."
  type = list(object({
    cidr_block   = string
    display_name = string
  }))
}

variable "gke_release_channel" {
  description = "GKE sürüm kanalı."
  type        = string
  default     = "RAPID"
}

variable "deletion_protection" {
  description = "Küme ve veritabanı için silme koruması."
  type        = bool
  default     = false
}

variable "sql_tier" {
  description = "Cloud SQL makine sınıfı."
  type        = string
  default     = "db-custom-1-3840"
}

variable "sql_availability_type" {
  description = "Cloud SQL erişilebilirlik türü (REGIONAL yüksek erişilebilir)."
  type        = string
  default     = "ZONAL"
}

variable "sql_disk_size_gb" {
  description = "Cloud SQL başlangıç disk boyutu (GB)."
  type        = number
  default     = 20
}

variable "sql_read_replica_count" {
  description = "Cloud SQL okuma kopyası sayısı."
  type        = number
  default     = 0
}

variable "sql_iam_service_accounts" {
  description = "Veritabanına IAM ile bağlanacak iş yükü hizmet hesapları (e-posta)."
  type        = list(string)
  default     = []
}

variable "redis_tier" {
  description = "Memorystore katmanı."
  type        = string
  default     = "BASIC"
}

variable "redis_memory_size_gb" {
  description = "Memorystore bellek boyutu (GB)."
  type        = number
  default     = 1
}

variable "storage_cors_origins" {
  description = "İmzalı bağlantıyla yükleme yapacak web kökenleri."
  type        = list(string)
  default     = []
}

variable "storage_force_destroy" {
  description = "Dolu nesne kovasının silinmesine izin ver."
  type        = bool
  default     = true
}

variable "audit_retention_days" {
  description = "Denetim arşivi saklama süresi (gün). Canlı değeri hukuki süreye göre [doğrulanacak]."
  type        = number
  default     = 1
}

variable "audit_retention_locked" {
  description = "Saklama politikasını kalıcı kilitle (WORM, geri alınamaz)."
  type        = bool
  default     = false
}

variable "registry_writer_members" {
  description = "İmaj gönderebilecek üyeler (CI hizmet hesabı), IAM biçiminde."
  type        = list(string)
  default     = []
}

variable "secret_ids" {
  description = "Boş kap olarak açılacak sırlar; değerler Secret Manager'a Terraform dışında girilir."
  type        = list(string)
  default = [
    "api-session-signing-key",
    "oidc-apple-client-secret",
    "oidc-google-client-secret",
    "email-provider-api-key",
    "github-app-private-key",
    "redis-auth",
  ]
}

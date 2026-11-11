variable "project_id" {
  description = "Google Cloud proje kimliği."
  type        = string
}

variable "name" {
  description = "Örnek adı. Silinen adlar bir süre yeniden kullanılamaz; değiştirmeden önce düşün."
  type        = string
}

variable "region" {
  description = "Örnek bölgesi."
  type        = string
}

variable "network_id" {
  description = "Özel IP için VPC kaynak kimliği."
  type        = string
}

variable "database_version" {
  description = "PostgreSQL sürümü."
  type        = string
  default     = "POSTGRES_16"
}

variable "tier" {
  description = "Makine sınıfı (ör. db-custom-2-7680)."
  type        = string
}

variable "availability_type" {
  description = "REGIONAL (yüksek erişilebilir) veya ZONAL (yalnız geliştirme)."
  type        = string
  default     = "REGIONAL"

  validation {
    condition     = contains(["REGIONAL", "ZONAL"], var.availability_type)
    error_message = "availability_type REGIONAL veya ZONAL olmalı."
  }
}

variable "disk_size_gb" {
  description = "Başlangıç disk boyutu; disk otomatik büyür."
  type        = number
  default     = 50
}

variable "backup_start_time" {
  description = "Günlük yedek başlangıcı (UTC, SS:DD)."
  type        = string
  default     = "03:00"
}

variable "backup_retention_count" {
  description = "Tutulacak otomatik yedek sayısı."
  type        = number
  default     = 14
}

variable "pitr_retention_days" {
  description = "Noktaya geri dönüş (PITR) için işlem günlüğü saklama süresi (1-35 gün)."
  type        = number
  default     = 7
}

variable "database_name" {
  description = "Oluşturulacak uygulama veritabanı."
  type        = string
  default     = "demleme"
}

variable "iam_service_accounts" {
  description = "Parolasız (IAM) girişle bağlanacak hizmet hesabı e-postaları; her biri için veritabanı kullanıcısı ve cloudsql.instanceUser rolü verilir."
  type        = list(string)
  default     = []
}

variable "read_replica_count" {
  description = "Okuma kopyası sayısı (aynı bölgede)."
  type        = number
  default     = 0
}

variable "kms_key_name" {
  description = "Müşteri yönetimli şifreleme anahtarı (CMEK); boşsa Google yönetimli anahtar. Cloud SQL hizmet hesabına anahtar izni ayrıca verilmelidir."
  type        = string
  default     = null
}

variable "deletion_protection" {
  description = "Terraform ve API düzeyinde silme koruması (canlıda true)."
  type        = bool
  default     = true
}

variable "labels" {
  description = "Kaynak etiketleri."
  type        = map(string)
  default     = {}
}

variable "project_id" {
  description = "Google Cloud proje kimliği."
  type        = string
}

variable "name" {
  description = "KMS anahtar halkası adı."
  type        = string
}

variable "location" {
  description = "KMS ve Secret Manager konumu (veri yeri için tek bölge)."
  type        = string
}

variable "key_names" {
  description = "Oluşturulacak şifreleme anahtarları (ör. sql, storage, secrets)."
  type        = list(string)
  default     = ["sql", "storage", "redis", "registry", "secrets"]
}

variable "key_rotation_period" {
  description = "Anahtar döndürme sıklığı (saniye, 's' ekli)."
  type        = string
  default     = "7776000s"
}

variable "key_users" {
  description = "Anahtar adı -> şifreleme/çözme yetkisi verilecek üyeler (IAM biçimi). Örn. Cloud SQL hizmet hesabı."
  type        = map(list(string))
  default     = {}
}

variable "secret_ids" {
  description = "Boş kap olarak açılacak sırlar (değerler Terraform dışında, Secret Manager'a girilir)."
  type        = list(string)
  default     = []
}

variable "secret_accessors" {
  description = "Sır kimliği -> okuma yetkisi verilecek üyeler (IAM biçimi). Yetki yalnız o sır üzerinde."
  type        = map(list(string))
  default     = {}
}

variable "labels" {
  description = "Sırlara eklenecek etiketler."
  type        = map(string)
  default     = {}
}

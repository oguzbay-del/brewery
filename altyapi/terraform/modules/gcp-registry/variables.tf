variable "project_id" {
  description = "Google Cloud proje kimliği."
  type        = string
}

variable "repository_id" {
  description = "Depo adı."
  type        = string
}

variable "location" {
  description = "Depo bölgesi."
  type        = string
}

variable "immutable_tags" {
  description = "true ise bir etiket başka imaja yeniden yazılamaz."
  type        = bool
  default     = true
}

variable "keep_recent_versions" {
  description = "Her paket için tutulacak son sürüm sayısı."
  type        = number
  default     = 30
}

variable "untagged_retention_days" {
  description = "Etiketsiz imajların silinmeden önce tutulacağı gün sayısı."
  type        = number
  default     = 14
}

variable "reader_members" {
  description = "Depodan imaj çekebilecek üyeler (ör. düğüm hizmet hesabı), IAM biçiminde: serviceAccount:..."
  type        = list(string)
  default     = []
}

variable "writer_members" {
  description = "Depoya imaj gönderebilecek üyeler (ör. CI hizmet hesabı), IAM biçiminde."
  type        = list(string)
  default     = []
}

variable "kms_key_name" {
  description = "CMEK anahtarı; boşsa Google yönetimli."
  type        = string
  default     = null
}

variable "labels" {
  description = "Kaynak etiketleri."
  type        = map(string)
  default     = {}
}

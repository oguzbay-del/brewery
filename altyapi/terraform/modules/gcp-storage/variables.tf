variable "project_id" {
  description = "Google Cloud proje kimliği."
  type        = string
}

variable "name_prefix" {
  description = "Kova adı öneki; küresel olarak benzersiz olmalı (ör. demleme-dev-us)."
  type        = string
}

variable "location" {
  description = "Kova konumu: veri yeri için tek bölge (us-east4, europe-west2)."
  type        = string
}

variable "kms_key_name" {
  description = "CMEK anahtarı; boşsa Google yönetimli. Cloud Storage hizmet hesabına izin ayrıca verilmelidir."
  type        = string
  default     = null
}

variable "noncurrent_version_retention_days" {
  description = "Eski nesne sürümlerinin silinmeden önce tutulacağı gün sayısı."
  type        = number
  default     = 30
}

variable "cors_origins" {
  description = "İmzalı bağlantıyla doğrudan yükleme/indirme yapacak web kökenleri."
  type        = list(string)
  default     = []
}

variable "audit_retention_days" {
  description = "Denetim arşivi kovasında nesnelerin silinemez kaldığı gün sayısı (saklama politikası)."
  type        = number
}

variable "audit_retention_locked" {
  description = "true ise saklama politikası kilitlenir (WORM) ve geri alınamaz. Yalnız canlıda, hukuki karar sonrası true yap."
  type        = bool
  default     = false
}

variable "force_destroy" {
  description = "true ise dolu kova da silinebilir (yalnız geliştirme)."
  type        = bool
  default     = false
}

variable "labels" {
  description = "Kaynak etiketleri."
  type        = map(string)
  default     = {}
}

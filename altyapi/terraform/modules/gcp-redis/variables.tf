variable "project_id" {
  description = "Google Cloud proje kimliği."
  type        = string
}

variable "name" {
  description = "Örnek adı."
  type        = string
}

variable "region" {
  description = "Örnek bölgesi."
  type        = string
}

variable "network_id" {
  description = "Bağlanılacak VPC kaynak kimliği."
  type        = string
}

variable "tier" {
  description = "BASIC (geliştirme) veya STANDARD_HA (yüksek erişilebilir)."
  type        = string
  default     = "STANDARD_HA"

  validation {
    condition     = contains(["BASIC", "STANDARD_HA"], var.tier)
    error_message = "tier BASIC veya STANDARD_HA olmalı."
  }
}

variable "memory_size_gb" {
  description = "Bellek boyutu (GB)."
  type        = number
  default     = 1
}

variable "redis_version" {
  description = "Redis sürümü."
  type        = string
  default     = "REDIS_7_2"
}

variable "kms_key_name" {
  description = "Müşteri yönetimli şifreleme anahtarı; boşsa Google yönetimli."
  type        = string
  default     = null
}

variable "labels" {
  description = "Kaynak etiketleri."
  type        = map(string)
  default     = {}
}

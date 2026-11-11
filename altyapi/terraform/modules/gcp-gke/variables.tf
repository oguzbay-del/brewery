variable "project_id" {
  description = "Google Cloud proje kimliği."
  type        = string
}

variable "name" {
  description = "Küme adı."
  type        = string
}

variable "region" {
  description = "Kümenin bölgesi (Autopilot bölgesel kümedir)."
  type        = string
}

variable "network_id" {
  description = "VPC kaynak kimliği."
  type        = string
}

variable "subnet_id" {
  description = "Alt ağ kaynak kimliği."
  type        = string
}

variable "pods_range_name" {
  description = "Alt ağdaki Pod ikincil aralığının adı."
  type        = string
}

variable "services_range_name" {
  description = "Alt ağdaki Service ikincil aralığının adı."
  type        = string
}

variable "master_ipv4_cidr_block" {
  description = "Kontrol düzlemi için /28 aralık; VPC'deki diğer aralıklarla çakışmamalı."
  type        = string
  default     = "172.16.0.0/28"
}

variable "master_authorized_networks" {
  description = "Kontrol düzlemine erişebilecek CIDR'ler (CI, yönetici ağı). Boş liste: dış erişim kapalı."
  type = list(object({
    cidr_block   = string
    display_name = string
  }))
}

variable "release_channel" {
  description = "GKE sürüm kanalı: RAPID, REGULAR veya STABLE."
  type        = string
  default     = "REGULAR"

  validation {
    condition     = contains(["RAPID", "REGULAR", "STABLE"], var.release_channel)
    error_message = "release_channel RAPID, REGULAR veya STABLE olmalı."
  }
}

variable "deletion_protection" {
  description = "Küme silmeye karşı korumalı mı (canlıda true)."
  type        = bool
  default     = true
}

variable "labels" {
  description = "Kümeye eklenecek etiketler."
  type        = map(string)
  default     = {}
}

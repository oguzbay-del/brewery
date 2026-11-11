variable "project_id" {
  description = "Kaynakların açılacağı Google Cloud proje kimliği."
  type        = string
}

variable "name" {
  description = "Kaynak adı öneki (ör. demleme-dev-us)."
  type        = string
}

variable "region" {
  description = "Alt ağın ve NAT'ın bölgesi (ör. us-east4, europe-west2)."
  type        = string
}

variable "subnet_cidr" {
  description = "Düğüm alt ağının birincil CIDR aralığı."
  type        = string
}

variable "pods_cidr" {
  description = "GKE Pod'ları için ikincil aralık."
  type        = string
}

variable "services_cidr" {
  description = "GKE Service'leri için ikincil aralık."
  type        = string
}

variable "psa_prefix_length" {
  description = "Özel hizmet erişimi (Cloud SQL, Memorystore) için ayrılan aralığın önek uzunluğu."
  type        = number
  default     = 20
}

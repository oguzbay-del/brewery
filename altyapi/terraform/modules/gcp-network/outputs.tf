output "network_id" {
  description = "VPC kaynak kimliği."
  value       = google_compute_network.this.id
}

output "network_name" {
  description = "VPC adı."
  value       = google_compute_network.this.name
}

output "subnet_id" {
  description = "Düğüm alt ağının kaynak kimliği."
  value       = google_compute_subnetwork.nodes.id
}

output "subnet_name" {
  description = "Düğüm alt ağının adı."
  value       = google_compute_subnetwork.nodes.name
}

output "pods_range_name" {
  description = "GKE Pod'ları için ikincil aralık adı."
  value       = "pods"
}

output "services_range_name" {
  description = "GKE Service'leri için ikincil aralık adı."
  value       = "services"
}

output "psa_connection_id" {
  description = "Özel hizmet erişimi bağlantısı; Cloud SQL ve Redis bunun bitmesini beklemeli."
  value       = google_service_networking_connection.psa.id
}

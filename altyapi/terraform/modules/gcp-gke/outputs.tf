output "cluster_name" {
  description = "Küme adı."
  value       = google_container_cluster.this.name
}

output "cluster_id" {
  description = "Küme kaynak kimliği."
  value       = google_container_cluster.this.id
}

output "endpoint" {
  description = "Kontrol düzlemi adresi."
  value       = google_container_cluster.this.endpoint
}

output "ca_certificate" {
  description = "Küme CA sertifikası (base64)."
  value       = google_container_cluster.this.master_auth[0].cluster_ca_certificate
  sensitive   = true
}

output "workload_pool" {
  description = "Workload Identity havuzu; Kubernetes hizmet hesapları IAM'e bu havuzla bağlanır."
  value       = "${var.project_id}.svc.id.goog"
}

output "node_service_account_email" {
  description = "Düğüm hizmet hesabı (Artifact Registry okuma izni için)."
  value       = google_service_account.nodes.email
}

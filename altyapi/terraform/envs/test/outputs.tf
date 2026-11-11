output "gke_cluster_name" {
  description = "GKE Autopilot küme adı."
  value       = module.gke.cluster_name
}

output "gke_workload_pool" {
  description = "Workload Identity havuzu."
  value       = module.gke.workload_pool
}

output "sql_connection_name" {
  description = "Cloud SQL bağlantı adı."
  value       = module.sql.connection_name
}

output "sql_private_ip" {
  description = "Cloud SQL özel IP."
  value       = module.sql.private_ip
}

output "redis_host" {
  description = "Redis özel IP."
  value       = module.redis.host
}

output "redis_port" {
  description = "Redis bağlantı noktası."
  value       = module.redis.port
}

output "objects_bucket" {
  description = "Nesne kovası."
  value       = module.storage.objects_bucket_name
}

output "audit_bucket" {
  description = "Denetim arşivi kovası."
  value       = module.storage.audit_bucket_name
}

output "image_prefix" {
  description = "Konteyner imajı öneki."
  value       = module.registry.image_prefix
}

output "kms_key_ids" {
  description = "KMS anahtarları."
  value       = module.secrets.key_ids
}

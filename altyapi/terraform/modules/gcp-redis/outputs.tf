output "host" {
  description = "Redis özel IP adresi."
  value       = google_redis_instance.this.host
}

output "port" {
  description = "Redis bağlantı noktası."
  value       = google_redis_instance.this.port
}

output "auth_string" {
  description = "AUTH parolası; Secret Manager'a aktarılır, çıktı olarak yazdırılmaz."
  value       = google_redis_instance.this.auth_string
  sensitive   = true
}

output "server_ca_certs" {
  description = "TLS bağlantısı için sunucu CA sertifikaları."
  value       = google_redis_instance.this.server_ca_certs
}

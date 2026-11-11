output "instance_name" {
  description = "Birincil örnek adı."
  value       = google_sql_database_instance.primary.name
}

output "connection_name" {
  description = "Bağlantı adı (proje:bölge:örnek)."
  value       = google_sql_database_instance.primary.connection_name
}

output "private_ip" {
  description = "Birincil örneğin özel IP adresi."
  value       = google_sql_database_instance.primary.private_ip_address
}

output "replica_private_ips" {
  description = "Okuma kopyalarının özel IP adresleri."
  value       = google_sql_database_instance.replica[*].private_ip_address
}

output "database_name" {
  description = "Uygulama veritabanı adı."
  value       = google_sql_database.app.name
}

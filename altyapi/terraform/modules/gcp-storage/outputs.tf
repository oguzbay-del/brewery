output "objects_bucket_name" {
  description = "Nesne kovasının adı."
  value       = google_storage_bucket.objects.name
}

output "audit_bucket_name" {
  description = "Denetim arşivi kovasının adı."
  value       = google_storage_bucket.audit.name
}

output "digest_bucket_name" {
  description = "Denetim zinciri özetleri kovasının adı (S3_DIGEST_BUCKET)."
  value       = google_storage_bucket.digest.name
}

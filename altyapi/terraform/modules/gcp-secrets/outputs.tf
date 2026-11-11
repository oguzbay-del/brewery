output "key_ring_id" {
  description = "KMS anahtar halkası kimliği."
  value       = google_kms_key_ring.this.id
}

output "key_ids" {
  description = "Anahtar adı -> anahtar kaynak kimliği (CMEK için kms_key_name)."
  value       = { for k, v in google_kms_crypto_key.this : k => v.id }
}

output "secret_ids" {
  description = "Oluşturulan sırların kimlikleri."
  value       = [for s in google_secret_manager_secret.this : s.secret_id]
}

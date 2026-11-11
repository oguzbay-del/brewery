locals {
  key_user_pairs = flatten([
    for key, members in var.key_users : [
      for member in members : { key = key, member = member }
    ]
  ])

  secret_accessor_pairs = flatten([
    for secret, members in var.secret_accessors : [
      for member in members : { secret = secret, member = member }
    ]
  ])
}

resource "google_kms_key_ring" "this" {
  project  = var.project_id
  name     = var.name
  location = var.location
}

resource "google_kms_crypto_key" "this" {
  for_each = toset(var.key_names)

  name            = each.value
  key_ring        = google_kms_key_ring.this.id
  rotation_period = var.key_rotation_period
  purpose         = "ENCRYPT_DECRYPT"

  version_template {
    algorithm        = "GOOGLE_SYMMETRIC_ENCRYPTION"
    protection_level = "SOFTWARE"
  }

  lifecycle {
    prevent_destroy = true
  }
}

# Gerekçe: şifreleme/çözme yetkisi anahtar düzeyinde, yalnız adı verilen üyelere.
resource "google_kms_crypto_key_iam_member" "users" {
  for_each = { for p in local.key_user_pairs : "${p.key}/${p.member}" => p }

  crypto_key_id = google_kms_crypto_key.this[each.value.key].id
  role          = "roles/cloudkms.cryptoKeyEncrypterDecrypter"
  member        = each.value.member
}

# Yalnız kap: sır değeri durum dosyasına girmesin diye sürüm Terraform'da açılmaz.
resource "google_secret_manager_secret" "this" {
  for_each = toset(var.secret_ids)

  project   = var.project_id
  secret_id = each.value
  labels    = var.labels

  replication {
    user_managed {
      replicas {
        location = var.location
      }
    }
  }
}

# Gerekçe: okuma yetkisi sır düzeyinde; proje geneli secretAccessor verilmez.
resource "google_secret_manager_secret_iam_member" "accessors" {
  for_each = { for p in local.secret_accessor_pairs : "${p.secret}/${p.member}" => p }

  project   = var.project_id
  secret_id = google_secret_manager_secret.this[each.value.secret].secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = each.value.member
}

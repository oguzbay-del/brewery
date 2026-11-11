# Taranan kağıtlar, fotoğraflar, PDF ve dışa aktarmalar. Erişim kısa süreli imzalı bağlantıyla.
resource "google_storage_bucket" "objects" {
  project                     = var.project_id
  name                        = "${var.name_prefix}-objects"
  location                    = var.location
  storage_class               = "STANDARD"
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"
  force_destroy               = var.force_destroy
  labels                      = var.labels

  versioning {
    enabled = true
  }

  dynamic "encryption" {
    for_each = var.kms_key_name == null ? [] : [var.kms_key_name]
    content {
      default_kms_key_name = encryption.value
    }
  }

  lifecycle_rule {
    condition {
      num_newer_versions         = 1
      days_since_noncurrent_time = var.noncurrent_version_retention_days
    }
    action {
      type = "Delete"
    }
  }

  lifecycle_rule {
    condition {
      age = 1
    }
    action {
      type = "AbortIncompleteMultipartUpload"
    }
  }

  dynamic "cors" {
    for_each = length(var.cors_origins) > 0 ? [1] : []
    content {
      origin          = var.cors_origins
      method          = ["GET", "PUT", "HEAD"]
      response_header = ["Content-Type", "Content-MD5", "x-goog-content-length-range"]
      max_age_seconds = 3600
    }
  }
}

# İşletme modu denetim arşivi: saklama politikasıyla silinmez; kilit isteğe bağlı (WORM).
resource "google_storage_bucket" "audit" {
  project                     = var.project_id
  name                        = "${var.name_prefix}-audit"
  location                    = var.location
  storage_class               = "STANDARD"
  uniform_bucket_level_access = true
  public_access_prevention    = "enforced"
  force_destroy               = false
  labels                      = var.labels

  versioning {
    enabled = true
  }

  retention_policy {
    retention_period = var.audit_retention_days * 86400
    is_locked        = var.audit_retention_locked
  }

  dynamic "encryption" {
    for_each = var.kms_key_name == null ? [] : [var.kms_key_name]
    content {
      default_kms_key_name = encryption.value
    }
  }
}

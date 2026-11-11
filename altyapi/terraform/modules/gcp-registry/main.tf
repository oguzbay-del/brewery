resource "google_artifact_registry_repository" "this" {
  project       = var.project_id
  repository_id = var.repository_id
  location      = var.location
  format        = "DOCKER"
  description   = "OCI konteyner imajları"
  kms_key_name  = var.kms_key_name
  labels        = var.labels

  docker_config {
    immutable_tags = var.immutable_tags
  }

  cleanup_policy_dry_run = false

  cleanup_policies {
    id     = "etiketsiz-sil"
    action = "DELETE"
    condition {
      tag_state  = "UNTAGGED"
      older_than = "${var.untagged_retention_days * 86400}s"
    }
  }

  cleanup_policies {
    id     = "son-surumleri-tut"
    action = "KEEP"
    most_recent_versions {
      keep_count = var.keep_recent_versions
    }
  }
}

# Gerekçe: roller depo düzeyinde; proje genelinde imaj erişimi verilmez.
resource "google_artifact_registry_repository_iam_member" "reader" {
  for_each = { for i, m in var.reader_members : tostring(i) => m }

  project    = var.project_id
  location   = google_artifact_registry_repository.this.location
  repository = google_artifact_registry_repository.this.name
  role       = "roles/artifactregistry.reader"
  member     = each.value
}

resource "google_artifact_registry_repository_iam_member" "writer" {
  for_each = { for i, m in var.writer_members : tostring(i) => m }

  project    = var.project_id
  location   = google_artifact_registry_repository.this.location
  repository = google_artifact_registry_repository.this.name
  role       = "roles/artifactregistry.writer"
  member     = each.value
}

# Redis yalnız önbellek, oturum ve hız sınırı sayaçları tutar; kaybolursa veri kaybı olmaz (plan › Veri katmanı).
resource "google_redis_instance" "this" {
  project                 = var.project_id
  name                    = var.name
  region                  = var.region
  tier                    = var.tier
  memory_size_gb          = var.memory_size_gb
  redis_version           = var.redis_version
  authorized_network      = var.network_id
  connect_mode            = "PRIVATE_SERVICE_ACCESS"
  auth_enabled            = true
  transit_encryption_mode = "SERVER_AUTHENTICATION"
  customer_managed_key    = var.kms_key_name
  labels                  = var.labels

  maintenance_policy {
    weekly_maintenance_window {
      day = "SUNDAY"
      start_time {
        hours   = 4
        minutes = 0
      }
    }
  }
}

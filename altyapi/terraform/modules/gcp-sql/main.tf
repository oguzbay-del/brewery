locals {
  # Cloud SQL, hizmet hesabı kullanıcı adını ".gserviceaccount.com" soneki olmadan ister.
  iam_users = { for email in var.iam_service_accounts : email => trimsuffix(email, ".gserviceaccount.com") }
}

resource "google_sql_database_instance" "primary" {
  project             = var.project_id
  name                = var.name
  region              = var.region
  database_version    = var.database_version
  deletion_protection = var.deletion_protection
  encryption_key_name = var.kms_key_name

  settings {
    tier                        = var.tier
    edition                     = "ENTERPRISE"
    availability_type           = var.availability_type
    disk_type                   = "PD_SSD"
    disk_size                   = var.disk_size_gb
    disk_autoresize             = true
    deletion_protection_enabled = var.deletion_protection
    user_labels                 = var.labels

    ip_configuration {
      ipv4_enabled    = false
      private_network = var.network_id
      ssl_mode        = "ENCRYPTED_ONLY"
    }

    backup_configuration {
      enabled                        = true
      start_time                     = var.backup_start_time
      point_in_time_recovery_enabled = true
      transaction_log_retention_days = var.pitr_retention_days

      backup_retention_settings {
        retained_backups = var.backup_retention_count
        retention_unit   = "COUNT"
      }
    }

    maintenance_window {
      day          = 7
      hour         = 4
      update_track = "stable"
    }

    database_flags {
      name  = "cloudsql.iam_authentication"
      value = "on"
    }

    database_flags {
      name  = "log_min_duration_statement"
      value = "1000"
    }

    insights_config {
      query_insights_enabled = true
    }
  }
}

resource "google_sql_database_instance" "replica" {
  count = var.read_replica_count

  project              = var.project_id
  name                 = "${var.name}-replica-${count.index + 1}"
  region               = var.region
  database_version     = var.database_version
  master_instance_name = google_sql_database_instance.primary.name
  deletion_protection  = var.deletion_protection
  encryption_key_name  = var.kms_key_name

  settings {
    tier                        = var.tier
    edition                     = "ENTERPRISE"
    availability_type           = "ZONAL"
    disk_type                   = "PD_SSD"
    disk_autoresize             = true
    deletion_protection_enabled = var.deletion_protection
    user_labels                 = var.labels

    ip_configuration {
      ipv4_enabled    = false
      private_network = var.network_id
      ssl_mode        = "ENCRYPTED_ONLY"
    }

    database_flags {
      name  = "cloudsql.iam_authentication"
      value = "on"
    }
  }
}

resource "google_sql_database" "app" {
  project  = var.project_id
  instance = google_sql_database_instance.primary.name
  name     = var.database_name
}

resource "google_sql_user" "iam" {
  for_each = local.iam_users

  project  = var.project_id
  instance = google_sql_database_instance.primary.name
  name     = each.value
  type     = "CLOUD_IAM_SERVICE_ACCOUNT"
}

# Gerekçe: IAM ile veritabanına girişin tek koşulu; yalnız adı verilen hizmet hesaplarına.
resource "google_project_iam_member" "instance_user" {
  for_each = local.iam_users

  project = var.project_id
  role    = "roles/cloudsql.instanceUser"
  member  = "serviceAccount:${each.key}"
}

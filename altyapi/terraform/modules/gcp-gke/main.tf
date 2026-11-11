# Düğümlerin kullandığı hizmet hesabı: varsayılan Compute hesabı (Editor) yerine en az yetki.
resource "google_service_account" "nodes" {
  project      = var.project_id
  account_id   = "${substr(var.name, 0, 20)}-nodes"
  display_name = "GKE düğümleri (${var.name})"
}

# Gerekçe: düğüm günlüğü ve ölçümü yazabilmek için gereken en küçük roller.
resource "google_project_iam_member" "nodes" {
  for_each = toset([
    "roles/logging.logWriter",
    "roles/monitoring.metricWriter",
    "roles/monitoring.viewer",
    "roles/stackdriver.resourceMetadata.writer",
  ])

  project = var.project_id
  role    = each.value
  member  = "serviceAccount:${google_service_account.nodes.email}"
}

resource "google_container_cluster" "this" {
  project  = var.project_id
  name     = var.name
  location = var.region

  enable_autopilot    = true
  deletion_protection = var.deletion_protection
  network             = var.network_id
  subnetwork          = var.subnet_id
  resource_labels     = var.labels

  ip_allocation_policy {
    cluster_secondary_range_name  = var.pods_range_name
    services_secondary_range_name = var.services_range_name
  }

  private_cluster_config {
    enable_private_nodes    = true
    enable_private_endpoint = false
    master_ipv4_cidr_block  = var.master_ipv4_cidr_block
  }

  master_authorized_networks_config {
    dynamic "cidr_blocks" {
      for_each = var.master_authorized_networks
      content {
        cidr_block   = cidr_blocks.value.cidr_block
        display_name = cidr_blocks.value.display_name
      }
    }
  }

  cluster_autoscaling {
    auto_provisioning_defaults {
      service_account = google_service_account.nodes.email
      oauth_scopes    = ["https://www.googleapis.com/auth/cloud-platform"]
    }
  }

  release_channel {
    channel = var.release_channel
  }

  # Workload Identity Autopilot'ta açıktır; Pod'lar hizmet hesabına buradan bağlanır.
  workload_identity_config {
    workload_pool = "${var.project_id}.svc.id.goog"
  }

  security_posture_config {
    mode               = "BASIC"
    vulnerability_mode = "VULNERABILITY_DISABLED"
  }

  depends_on = [google_project_iam_member.nodes]
}
